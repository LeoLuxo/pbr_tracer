use brainrot::{path, Shader};

use crate::core::rendering::render_fragments::{PostProcessingEffect, RenderFragment};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct GammaCorrection;

impl PostProcessingEffect for GammaCorrection {}

impl RenderFragment for GammaCorrection {
	fn shader(&self) -> Shader {
		path!("/post_processing/gamma.wgsl").into()
	}

	// fn iter_sub_fragments(&self) -> &dyn Iterator<Item = &dyn RenderFragment> {
	// 	todo!()
	// }
}
