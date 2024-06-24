use brainrot::{path, Shader, ShaderBuilder};
use velcro::vec;

use super::shader_fragments::ShaderFragment;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

/// Shader API:\
/// `fn post_processing_effect(coord: vec2f, color: vec4f) -> vec4f`
pub trait PostProcessingEffect: ShaderFragment {}

/// Shader API:\
/// `fn post_processing_pipeline(coord: vec2f, color: vec4f) -> vec4f`
#[derive(Default)]
pub struct PostProcessingPipeline(Vec<Box<dyn PostProcessingEffect>>);

impl PostProcessingPipeline {
	pub fn empty() -> Self {
		Self::default()
	}

	pub fn with(mut self, effect: impl PostProcessingEffect + 'static) -> Self {
		self.0.push(Box::new(effect));
		self
	}
}

impl ShaderFragment for PostProcessingPipeline {
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

	fn fragments(&self) -> Vec<&dyn ShaderFragment> {
		vec![
			self as &dyn ShaderFragment,
			..self.0.iter().flat_map(|v| (**v).fragments()),
		]
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct GammaCorrection;

impl PostProcessingEffect for GammaCorrection {}
impl ShaderFragment for GammaCorrection {
	fn shader(&self) -> Shader {
		path!("/post_processing/gamma.wgsl").into()
	}

	fn fragments(&self) -> Vec<&dyn ShaderFragment> {
		vec![self]
	}
}
