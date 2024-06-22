use std::iter;

use brainrot::{Shader, ShaderBuilder};

use crate::core::rendering::render_fragments::{
	PostProcessingPipeline, RenderFragment, RenderFragmentIterator, Renderer,
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct PhysBasedRaytracer {
	pub ppp: Option<PostProcessingPipeline>,
}

impl Renderer for PhysBasedRaytracer {}

impl RenderFragment for PhysBasedRaytracer {
	fn shader(&self) -> Shader {
		let mut builder = ShaderBuilder::new();
		builder
			.include_path("pathtracer.wgsl")
			.include_path("raymarch/raymarch.wgsl");

		// Conditionally include post-processing pipeline
		if let Some(ppp) = &self.ppp {
			builder.include(ppp.shader()).define(
				"CALL_POST_PROCESSING_PIPELINE",
				"color = post_processing_pipeline(coord, color);",
			);
		} else {
			builder.define("CALL_POST_PROCESSING_PIPELINE", "");
		}

		builder.into()
	}

	fn iter_sub_fragments<'a>(&'a self) -> RenderFragmentIterator<'a> {
		if let Some(ppp) = &self.ppp {
			Box::new(iter::once(ppp as &'a dyn RenderFragment))
		} else {
			Box::new(iter::empty())
		}
	}
}
