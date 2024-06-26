use std::{borrow::Cow, collections::HashSet, hash::Hash, mem, ops::Range};

use anyhow::{anyhow, Ok, Result};
use brainrot::{path, root, rooted_path};
use hashlink::{LinkedHashMap, LinkedHashSet};
use rand::seq::IteratorRandom;
use regex::Regex;
use replace_with::replace_with_or_abort;
use typed_path::{
	TypedPath, TypedPathBuf, UnixPath, UnixPathBuf, Utf8TypedPath, Utf8TypedPathBuf, Utf8UnixPath, Utf8UnixPathBuf,
	Utf8WindowsPath, Utf8WindowsPathBuf, WindowsPath, WindowsPathBuf,
};
use velcro::iter;
use wgpu::{Device, ShaderModule, ShaderModuleDescriptor};

use super::{
	buffer::{Bufferable, UniformBuffer, UniformBufferArc},
	embed::Assets,
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(Clone, Debug, Default, Hash, Eq, PartialEq)]
pub struct ShaderBuilder {
	include_directives: LinkedHashSet<Shader>,
	define_directives: LinkedHashMap<String, String>,
}

impl ShaderBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn include(&mut self, shader: impl Into<Shader>) -> &mut Self {
		self.include_directives.insert(shader.into());
		self
	}

	pub fn include_path(&mut self, path: impl Into<Utf8UnixPathBuf>) -> &mut Self {
		self.include(path.into())
	}

	pub fn include_uniform(&mut self, name: impl Bufferable) -> &mut Self {
		let uniform = &name as &dyn UniformBuffer;
		self
	}

	pub fn define<K, V>(&mut self, key: K, value: V) -> &mut Self
	where
		K: Into<String>,
		V: Into<String>,
	{
		self.define_directives.insert(key.into(), value.into());
		self
	}

	pub fn build<T: Assets>(&mut self, shader_map: &T, device: &Device) -> Result<CompiledShader> {
		let mut state = ShaderBuilderState::new(self.include_directives.clone(), shader_map);

		let shader_source = self.build_source(&mut state)?;

		let compiled_shader = shader_source.build(device);
		Ok(compiled_shader)
	}

	fn build_source(&mut self, state: &mut ShaderBuilderState) -> Result<ShaderSource> {
		let mut builder = mem::take(self);

		let mut shader_source = ShaderSource::new();

		for shader in builder.include_directives.drain() {
			let included_source = shader.build_recursively(state)?;
			shader_source.extend(included_source);
		}

		builder
			.define_directives
			.extend(Self::process_define_directives(&mut shader_source));
		shader_source = builder.apply_define_directives(shader_source);

		Ok(shader_source)
	}

	fn process_define_directives(shader_source: &mut ShaderSource) -> LinkedHashMap<String, String> {
		let mut define_directives = LinkedHashMap::<String, String>::new();

		// Find all `#define KEY value` in the source
		let re = Regex::new(r#"(?m)^#define (.+?) (.+?)$"#).unwrap();

		let mut ranges = Vec::<Range<usize>>::new();
		for caps in re.captures_iter(&shader_source.source) {
			// The bytes that the `#define ...` statement occupies
			let range = caps.get(0).unwrap().range();
			ranges.push(range);

			let key = caps.get(1).unwrap().as_str().to_owned();
			let value = caps.get(2).unwrap().as_str().to_owned();
			define_directives.insert(key, value);
		}

		// Delete the directives from the source string
		let mut offset: isize = 0;
		for range in ranges {
			let range = (range.start as isize + offset) as usize..(range.end as isize + offset) as usize;

			// Decrease offset since we're deleting sections of text
			offset -= range.len() as isize;

			shader_source.source.replace_range(range, "");
		}

		define_directives
	}

	fn apply_define_directives(&mut self, mut shader_source: ShaderSource) -> ShaderSource {
		let mut directives = self.define_directives.iter().collect::<Vec<_>>();
		// Sort by reverse size, so from biggest key to smallest key
		directives.sort_by(|(key1, _), (key2, _)| key2.cmp(key1));

		for (key, value) in directives {
			shader_source.source = shader_source.source.replace(key, value);
		}
		shader_source
	}
}

struct ShaderBuilderState<'a> {
	pub blacklist: HashSet<Shader>,
	pub binding_group_offset: u32,
	pub shader_map: &'a dyn Assets,
}

impl<'a> ShaderBuilderState<'a> {
	pub fn new<T: Assets>(include_directives: LinkedHashSet<Shader>, shader_map: &'a T) -> Self {
		Self {
			blacklist: HashSet::from_iter(include_directives),
			binding_group_offset: 0,
			shader_map: shader_map as &'a dyn Assets,
		}
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(Hash, Debug, Clone, PartialEq, Eq)]
pub enum Shader {
	Source(String),
	Path(Utf8UnixPathBuf),
	Builder(ShaderBuilder),
	Uniform { name: String, arc: UniformBufferArc },
}

impl Shader {
	pub fn get_parent(&self) -> Utf8UnixPathBuf {
		match self {
			Shader::Source(_) => root!(),
			Shader::Path(path) => path.parent().map(|x| x.to_owned()).unwrap_or(root!()),
			Shader::Builder(_) => root!(),
			Shader::Uniform { .. } => root!(),
		}
	}

	pub fn obfuscate_fn(&mut self, func_name: &str) -> String {
		// Generate the obfuscated function name
		let obfuscated = iter![..('a'..='z'), ..('A'..='Z')]
			.choose_multiple(&mut rand::thread_rng(), 16)
			.into_iter()
			.collect::<String>();

		let from = format!("{}(", func_name);
		let to = format!("{}(", obfuscated);

		replace_with_or_abort(self, |self_| match self_ {
			// Replace the source string directly
			Shader::Source(source) => source.replace(&from, &to).into(),
			// Make the path into a ShaderBuilder instead, and add a define directive
			Shader::Path(path) => ShaderBuilder::new().include(path).define(from, to).into(),
			// Add a define directive to the ShaderBuilder
			Shader::Builder(mut builder) => builder.define(from, to).into(),
			// Nothing to change in a uniform
			Shader::Uniform { .. } => self_,
		});

		obfuscated
	}

	fn get_raw_source(self, state: &mut ShaderBuilderState) -> Result<ShaderSource> {
		match self {
			Shader::Source(source) => Ok(ShaderSource::from_source(source)),
			Shader::Path(path) => {
				let path = rooted_path!(path);

				// Get the source from the shader map
				let source_data = state
					.shader_map
					.get(path.as_str())
					.ok_or(anyhow!("File not found: {}", path.as_str()))?
					.data;
				let source =
					String::from_utf8(source_data.to_vec()).or(Err(anyhow!("Invalid UTF8 file: {}", path.as_str())))?;

				Ok(ShaderSource::from_source(source))
			}
			Shader::Builder(mut builder) => builder.build_source(state),
			Shader::Uniform { name, arc } => {
				let struct_source = arc.get_source_code(state.binding_group_offset, 0, name);

				state.binding_group_offset += 1;

				// TODO

				Ok(ShaderSource::from_source(struct_source))
			}
		}
	}

	fn build_recursively(self, state: &mut ShaderBuilderState) -> Result<ShaderSource> {
		// Check that the file wasn't already included
		if state.blacklist.contains(&self) {
			// Not an error, just includes empty source
			return Ok(ShaderSource::from_source("".to_string()));
		}

		// Blacklist the shader from including it anymore
		state.blacklist.insert(self.clone());

		// The path of the current shader file
		let parent_path = self.get_parent();

		// Get the source from the shader
		let mut shader_source = self.get_raw_source(state)?;

		let mut byte_offset: isize = 0;
		let mut includes = Vec::<(String, Range<usize>)>::new();

		// Find all `#include "path/to/shader.wgsl"` in the source
		let re = Regex::new(r#"(?m)^#include "(.+?)"$"#).unwrap();

		for caps in re.captures_iter(&shader_source.source) {
			// The bytes that the `#include "path/to/shader.wgsl"` statement occupies
			let range = caps.get(0).unwrap().range();
			// The `path/to/shader.wgsl` part
			let path_str = caps.get(1).unwrap().as_str().to_owned();
			includes.push((path_str, range));
		}

		// Replace the include statements in the source with the actual source of each
		// file
		for (path_str, range) in includes {
			// Offset the range by byte_offset
			let range = (range.start as isize + byte_offset) as usize..(range.end as isize + byte_offset) as usize;

			// Fix up the path
			let path_relative: Utf8UnixPathBuf = path!(&path_str)
				.try_into()
				.or(Err(anyhow!("Invalid file `{}`", path_str)))?;
			let path_absolute = rooted_path!(parent_path.join(path_relative));

			// Recursively build the source of the included file
			let source_to_include = path_absolute.into_shader().build_recursively(state)?;

			// Get the byte-size of the file to be inserted, to shift the other insertions
			// afterwards
			byte_offset += (source_to_include.source.len() as isize) - (range.len() as isize);

			// Replace the whole range with the included file source
			shader_source.source.replace_range(range, &source_to_include.source);
		}

		Ok(shader_source)
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

trait ShaderPath {}
impl ShaderPath for TypedPath<'_> {}
impl ShaderPath for TypedPathBuf {}
impl ShaderPath for Utf8TypedPath<'_> {}
impl ShaderPath for Utf8TypedPathBuf {}
impl ShaderPath for UnixPath {}
impl ShaderPath for UnixPathBuf {}
impl ShaderPath for Utf8UnixPath {}
impl ShaderPath for Utf8UnixPathBuf {}
impl ShaderPath for WindowsPath {}
impl ShaderPath for WindowsPathBuf {}
impl ShaderPath for Utf8WindowsPath {}
impl ShaderPath for Utf8WindowsPathBuf {}

trait IntoShader {
	fn into_shader(self) -> Shader;
}

impl IntoShader for String {
	fn into_shader(self) -> Shader {
		Shader::Source(self)
	}
}

impl IntoShader for &str {
	fn into_shader(self) -> Shader {
		Shader::Source(self.to_owned())
	}
}

impl IntoShader for ShaderBuilder {
	fn into_shader(self) -> Shader {
		Shader::Builder(self)
	}
}

impl IntoShader for &mut ShaderBuilder {
	fn into_shader(self) -> Shader {
		Shader::Builder(mem::take(self))
	}
}

impl<P> IntoShader for P
where
	P: TryInto<Utf8UnixPathBuf> + ShaderPath,
{
	fn into_shader(self) -> Shader {
		// The case where a path is valid as Windows path but not as Unix is so rare
		// that it's okay to unwrap here instead of delegating the error to
		// ShaderBuilder.build
		Shader::Path(self.try_into().or(Err(anyhow!("Invalid shader path"))).unwrap())
	}
}

impl<T> From<T> for Shader
where
	T: IntoShader,
{
	fn from(value: T) -> Self {
		value.into_shader()
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShaderSource {
	pub source: String,
}

impl ShaderSource {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn from_source(source: String) -> Self {
		Self { source }
	}

	pub fn extend(&mut self, other: ShaderSource) -> &mut Self {
		self.source.push_str(&other.source);
		self
	}

	pub fn build(self, device: &Device) -> CompiledShader {
		let shader_module = device.create_shader_module(ShaderModuleDescriptor {
			label: None,
			source: wgpu::ShaderSource::Wgsl(<Cow<str>>::from(self.source)),
		});

		CompiledShader { shader_module }
	}
}

pub struct CompiledShader {
	pub shader_module: ShaderModule,
}
