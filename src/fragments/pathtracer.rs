use brainrot::{Shader, ShaderBuilder};
use velcro::vec;

use super::{post_processing::PostProcessingPipeline, tracing::Tracer};
use crate::fragments::shader_fragments::{Renderer, ShaderFragment};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct PhysBasedRaytracer<T>
where
	T: Tracer,
{
	pub tracer: T,
	pub ppp: PostProcessingPipeline,
}

impl<T: Tracer> Renderer for PhysBasedRaytracer<T> {}

impl<T: Tracer> ShaderFragment for PhysBasedRaytracer<T> {
	fn shader(&self) -> Shader {
		ShaderBuilder::new()
			// Base code
			.include_path("pbrt.wgsl")
			// Include tracer pipeline
			.include(self.tracer.shader())
			// Include post-processing pipeline
			.include(self.ppp.shader())
			.define(
				"CALL_POST_PROCESSING_PIPELINE",
				"color = post_processing_pipeline(coord, color);",
			)
			.into()
	}

	fn fragments(&self) -> Vec<&dyn ShaderFragment> {
		vec![self, &self.tracer, &self.ppp]
	}
}
