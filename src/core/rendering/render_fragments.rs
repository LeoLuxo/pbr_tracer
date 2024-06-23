use brainrot::{Shader, ShaderBuilder};
use velcro::vec;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub trait RenderFragment: Sync + Send {
	fn shader(&self) -> Shader;
	fn fragments(&self) -> Vec<&dyn RenderFragment>;
}

/// Shader API:\
/// `fn post_processing_pipeline(coord: vec2f, color: vec4f) -> vec4f`
#[derive(Default)]
pub struct PostProcessingPipeline(Vec<Box<dyn RenderFragment>>);

impl PostProcessingPipeline {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with(mut self, effect: impl PostProcessingEffect + 'static) -> Self {
		self.0.push(Box::new(effect));
		self
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

	fn fragments(&self) -> Vec<&dyn RenderFragment> {
		vec![
			self as &dyn RenderFragment,
			..self.0.iter().flat_map(|v| (**v).fragments()),
		]
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
