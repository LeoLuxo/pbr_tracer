use std::{borrow::Cow, collections::HashSet, fmt::Display, hash::Hash, mem, ops::Range, sync::Arc};

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
use velcro::{hash_map, iter};
use wgpu::{BindGroup, BindGroupLayout, Device, ShaderModule, ShaderModuleDescriptor, ShaderStages};

use super::{
	buffer::{BindGroupMapping, Buffer, BufferDataType, BufferType, BufferUploadable, ShaderStruct},
	embed::Assets,
	gpu::Gpu,
	smart_arc::SmartArc,
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

	pub fn include_struct<S>(&mut self) -> &mut Self
	where
		S: ShaderStruct,
	{
		self.include(S::get_source_code())
	}

	pub fn include_buffer<B, D>(&mut self, buffer_type: B, data: D) -> &mut Self
	where
		B: BufferType + 'static,
		D: BufferDataType<B> + 'static,
	{
		let buffer_type = SmartArc(Arc::new(buffer_type) as Arc<dyn BufferType>);
		let data = SmartArc(Arc::new(data) as Arc<dyn BufferUploadable>);
		self.include(Shader::Buffer { buffer_type, data })
	}

	pub fn define<K, V>(&mut self, key: K, value: V) -> &mut Self
	where
		K: Into<String>,
		V: Into<String>,
	{
		self.define_directives.insert(key.into(), value.into());
		self
	}

	pub fn build<T: Assets>(
		&mut self,
		gpu: &Gpu,
		shader_map: &T,
		shader_stages: ShaderStages,
		bind_group_offset: u32,
	) -> Result<CompiledShader> {
		let shader_source = self.build_source(gpu, shader_map, shader_stages, bind_group_offset)?;
		println!("{}", shader_source);

		let compiled_shader = shader_source.build(&gpu.device);
		Ok(compiled_shader)
	}

	pub fn build_source<T: Assets>(
		&mut self,
		gpu: &Gpu,
		shader_map: &T,
		shader_stages: ShaderStages,
		bind_group_offset: u32,
	) -> Result<ShaderSource> {
		let mut state = ShaderBuilderState::new(gpu, shader_map, shader_stages, bind_group_offset);
		self.build_source_from_state(&mut state)
	}

	fn build_source_from_state(&mut self, state: &mut ShaderBuilderState) -> Result<ShaderSource> {
		let mut builder = mem::take(self);

		let mut shader_source = ShaderSource::empty();

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

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

struct ShaderBuilderState<'a> {
	pub gpu: &'a Gpu,
	pub shader_map: &'a dyn Assets,
	pub shader_stages: ShaderStages,
	pub blacklist: HashSet<Shader>,
	pub bind_group_offset: u32,
}

impl<'a> ShaderBuilderState<'a> {
	pub fn new<T: Assets>(
		gpu: &'a Gpu,
		shader_map: &'a T,
		shader_stages: ShaderStages,
		bind_group_offset: u32,
	) -> Self {
		Self {
			gpu,
			blacklist: HashSet::new(),
			shader_stages,
			bind_group_offset,
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
	Buffer {
		buffer_type: SmartArc<dyn BufferType>,
		data: SmartArc<dyn BufferUploadable>,
	},
}

impl Shader {
	pub fn get_parent(&self) -> Utf8UnixPathBuf {
		match self {
			Shader::Source(_) => root!(),
			Shader::Path(path) => path.parent().map(|x| x.to_owned()).unwrap_or(root!()),
			Shader::Builder(_) => root!(),
			Shader::Buffer { .. } => root!(),
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
			Shader::Buffer { .. } => self_,
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
			Shader::Builder(mut builder) => builder.build_source_from_state(state),
			Shader::Buffer { buffer_type, data } => {
				let source = buffer_type.get_source_code(state.bind_group_offset, 0);

				// println!("\n\nsource of the buffer: {}\n", source);

				let buffer = Buffer::new(state.gpu, state.shader_stages, data.get_size(), buffer_type.as_ref());

				buffer.upload_bytes(state.gpu, &data.get_data(), 0);

				let shader_source = ShaderSource::from_buffer(
					source,
					buffer.bind_group_layout,
					buffer.bind_group,
					state.bind_group_offset,
				);

				state.bind_group_offset += 1;

				Ok(shader_source)
			}
		}
	}

	fn build_recursively(self, state: &mut ShaderBuilderState) -> Result<ShaderSource> {
		// Check that the file wasn't already included
		if state.blacklist.contains(&self) {
			// Not an error, just includes empty source
			return Ok(ShaderSource::empty());
		}

		// Blacklist the shader from including it anymore
		state.blacklist.insert(self.clone());

		// The path of the current shader file
		let parent_path = self.get_parent();

		// Get the source from the shader
		let mut shader_source = self.get_raw_source(state)?;

		let mut byte_offset: isize = 0;
		let mut includes = Vec::<(String, Range<usize>)>::new();

		// println!("RECURSIVE:\n{}\n======", shader_source);

		// Find all `#include "path/to/shader.wgsl"` in the source
		let re = Regex::new(r#"(?m)^#include "(.+?)""#).unwrap();

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

			// println!("{:?}", range);

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
			shader_source.extend_range(source_to_include, range);
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

#[derive(Debug, Default)]
pub struct ShaderSource {
	pub source: String,
	pub layouts: Vec<BindGroupLayout>,
	pub groups: BindGroupMapping,
}

impl ShaderSource {
	pub fn empty() -> Self {
		Self::default()
	}

	pub fn from_source(source: String) -> Self {
		Self {
			source,
			..Default::default()
		}
	}

	pub fn from_buffer(
		source: String,
		bind_group_layout: BindGroupLayout,
		bind_group: BindGroup,
		bind_group_index: u32,
	) -> Self {
		Self {
			source,
			layouts: vec![bind_group_layout],
			groups: BindGroupMapping(hash_map!(bind_group_index: bind_group)),
		}
	}

	pub fn extend_range(&mut self, other: ShaderSource, range: Range<usize>) -> &mut Self {
		self.source.replace_range(range, &other.source);
		self.extend_extras(other)
	}

	pub fn extend(&mut self, other: ShaderSource) -> &mut Self {
		self.source.push_str(&other.source);
		self.extend_extras(other)
	}

	pub fn build(self, device: &Device) -> CompiledShader {
		let shader_module = device.create_shader_module(ShaderModuleDescriptor {
			label: None,
			source: wgpu::ShaderSource::Wgsl(<Cow<str>>::from(self.source)),
		});

		CompiledShader {
			shader_module,
			layouts: self.layouts,
			groups: self.groups,
		}
	}

	fn extend_extras(&mut self, other: ShaderSource) -> &mut Self {
		self.layouts.extend(other.layouts);
		self.groups.0.extend(other.groups.0);
		self
	}
}

impl Display for ShaderSource {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(
			f,
			"ShaderSource:\nsource: {}\nlayouts: {:?}\ngroups: {:?}",
			&self.source, &self.layouts, &self.groups
		)
	}
}

pub struct CompiledShader {
	pub shader_module: ShaderModule,
	pub layouts: Vec<BindGroupLayout>,
	pub groups: BindGroupMapping,
}
