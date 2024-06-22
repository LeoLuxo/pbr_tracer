use std::iter::{self, Empty};

use brainrot::{Shader, ShaderBuilder};

use crate::core::{
	gameloop::Render,
	rendering::render_fragments::{PostProcessingPipeline, RenderFragment, Renderer},
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
}

struct RenderFragmentIter<'a>(Box<dyn Iterator<Item = &'a dyn RenderFragment> + 'a>);

impl<'a> RenderFragmentIter<'a> {
	pub fn empty() -> Self {
		RenderFragmentIter(Box::new(iter::empty::<&dyn RenderFragment>()))
	}
}

impl<'a, T, I> From<T> for RenderFragmentIter<'a>
where
	T: Iterator<Item = &'a I> + 'a,
	I: RenderFragment + 'a,
{
	fn from(value: T) -> Self {
		Self(Box::new(value.map(|v| v as &dyn RenderFragment)))
	}
}

impl PhysBasedRaytracer {
	fn iter_sub_fragments(&self) -> RenderFragmentIter {
		if let Some(ppp) = &self.ppp {
			iter::once(ppp).into()
		} else {
			RenderFragmentIter::empty()
		}
	}
}
