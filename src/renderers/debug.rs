use brainrot::{path, Shader};

use crate::core::rendering::render_fragments::{RenderFragment, Renderer};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct DebugRenderer;

impl Renderer for DebugRenderer {}

impl RenderFragment for DebugRenderer {
	fn shader(&self) -> Shader {
		path!("/debug.wgsl").into()
	}

	// fn iter_sub_fragments(&self) -> &dyn Iterator<Item = &dyn RenderFragment> {
	// 	todo!()
	// }
}
