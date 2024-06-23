use std::{
	iter,
	slice::{self, Iter},
};

use brainrot::{Shader, ShaderBuilder};
use dyn_clone::DynClone;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub enum RenderFragmentIter<'a> {
	Borrowed(Box<dyn Iterator<Item = &'a dyn RenderFragment> + 'a>),
	Owned(Box<dyn Iterator<Item = Box<dyn RenderFragment>>>),
}

impl<'a> RenderFragmentIter<'a> {
	pub fn empty() -> Self {
		RenderFragmentIter::Borrowed(Box::new(iter::empty::<&dyn RenderFragment>()))
	}

	pub fn new_borrowed(iter: impl Iterator<Item = &'a (impl RenderFragment + 'a)> + 'a) -> Self {
		let iter = iter.into_iter().map(|v| v as &'a dyn RenderFragment);
		RenderFragmentIter::Borrowed(Box::new(iter))
	}

	pub fn new_owned(iter: impl Iterator<Item = T> + 'static) where T : -> Self {
		let iter = iter
			.into_iter()
			.map(|v| dyn_clone::clone_box(&*v) as Box<dyn RenderFragment>);
		RenderFragmentIter::Owned(Box::new(iter))
	}
}

// impl<'a, T, F> From<T> for RenderFragmentIter<'a>
// where
// 	T: Iterator<Item = &'a F> + 'static,
// 	F: RenderFragment + 'static,
// {
// 	fn from(value: T) -> Self {
// 		Self::Owned(Box::new(
// 			value.map(|v| dyn_clone::clone_box(&*v) as Box<dyn RenderFragment>),
// 		))
// 	}
// }

// impl<'a, T> From<T> for RenderFragmentIter<'a>
// where
// 	T: Iterator<Item = Box<dyn RenderFragment>>,
// {
// 	fn from(value: T) -> Self {
// 		Self::Borrowed(Box::new(value.map(|v| v as &dyn RenderFragment)))
// 	}
// }

// impl<'a, F> FromIterator<&'a F> for RenderFragmentIter<'a>
// where
// 	F: RenderFragment,
// {
// 	fn from_iter<T>(iter: T) -> Self
// 	where
// 		T: IntoIterator<Item = &'a F>,
// 		<T as std::iter::IntoIterator>::IntoIter: 'a,
// 	{
// 		let iter = iter.into_iter().map(|v| v as &'a dyn RenderFragment);
// 		Self::Borrowed(Box::new(iter))
// 	}
// }

// impl<F> FromIterator<Box<F>> for RenderFragmentIter<'_>
// where
// 	F: RenderFragment,
// {
// 	fn from_iter<T>(iter: T) -> Self
// 	where
// 		T: IntoIterator<Item = Box<F>>,
// 	{
// 		Self::Owned(Box::new(iter.into_iter().map(|v| v as Box<dyn
// RenderFragment>))) 	}
// }

pub trait RenderFragment: Sync + Send + DynClone {
	fn shader(&self) -> Shader;
}

dyn_clone::clone_trait_object!(RenderFragment);

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

/// Shader API:\
/// `fn post_processing_pipeline(coord: vec2f, color: vec4f) -> vec4f`
#[derive(Clone, Default)]
pub struct PostProcessingPipeline(Vec<Box<dyn RenderFragment>>);

impl PostProcessingPipeline {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with(mut self, effect: impl PostProcessingEffect + 'static) -> Self {
		self.0.push(Box::new(effect));
		self
	}

	fn sub_fragments(&self) -> RenderFragmentIter {
		RenderFragmentIter::new_owned(self.0.iter())
	}
}

impl RenderFragment for PostProcessingPipeline {
	fn shader(&self) -> Shader {
		// Set up the pipeline function
		let mut builder = ShaderBuilder::new();
		builder.include_path("post_processing/pipeline.wgsl");

		let mut pipeline = String::new();

		// Go through all the effects, obfuscate their post_processing_effect() function
		// to a unique name and add a call to that function to the pipeline
		for effect in &self.0 {
			let mut shader = (*effect).shader();
			let func_name = shader.obfuscate_fn("post_processing_effect");
			pipeline += &format!("color = {}(coord, color);\n", func_name);
			builder.include(shader);
		}

		// Add the pipeline callers
		builder.define("CALL_EFFECTS", pipeline);

		builder.into()
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

/// Shader API:\
/// `fn render_pixel(pixel_coord: vec2f, pixel_size: vec2f) -> vec4f`
pub trait Renderer: RenderFragment {}

/// Shader API:\
/// `fn post_processing_effect(coord: vec2f, color: vec4f) -> vec4f`
pub trait PostProcessingEffect: RenderFragment {}
