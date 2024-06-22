use brainrot::{Shader, ShaderBuilder};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct RenderFragmentIterator<'a>(Box<dyn Iterator<Item = &'a (dyn RenderFragment + 'a)> + 'a>);

impl<'a, T> From<dyn Iterator<Item = T>> for RenderFragmentIterator<'a>
where
	T: RenderFragment,
{
	fn from(value: Iterator) -> Self {
		todo!()
	}
}

impl FromIterator for RenderFragmentIterator {
	fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
		todo!()
	}
}

pub trait RenderFragment: Sync + Send {
	fn shader(&self) -> Shader;
	fn iter_sub_fragments(&self) -> RenderFragmentIterator<'_>;
	// fn register(&self) -> Shader;
}

/// Shader API:\
/// `fn post_processing_pipeline(coord: vec2f, color: vec4f) -> vec4f`
#[derive(Default)]
pub struct PostProcessingPipeline(Vec<Box<dyn PostProcessingEffect>>);

impl PostProcessingPipeline {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn add_effect(mut self, effect: impl PostProcessingEffect + 'static) -> Self {
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

	fn iter_sub_fragments(&self) -> &dyn Iterator<Item = &dyn RenderFragment> {
		todo!()
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
