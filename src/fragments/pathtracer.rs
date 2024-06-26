use super::{intersector::Intersector, post_processing::PostProcessingPipeline};
use crate::{
	core::shader::{Shader, ShaderBuilder},
	fragments::shader_fragments::{Renderer, ShaderFragment},
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct PhysBasedRaytracer<T>
where
	T: Intersector,
{
	pub intersector: T,
	pub ppp: PostProcessingPipeline,
}

impl<T: Intersector> Renderer for PhysBasedRaytracer<T> {}
impl<T: Intersector> ShaderFragment for PhysBasedRaytracer<T> {
	fn shader(&self) -> Shader {
		ShaderBuilder::new()
			// Base code
			.include_path("pbrt.wgsl")
			// Include tracer pipeline
			.include(self.intersector.shader())
			// Include post-processing pipeline
			.include(self.ppp.shader())
			.define(
				"CALL_POST_PROCESSING_PIPELINE",
				"color = post_processing_pipeline(coord, color);",
			)
			.into()
	}
}
