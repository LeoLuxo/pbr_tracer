use brainrot::{path, Shader};

use super::render_fragments::{RenderFragment, Renderer};

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

	fn fragments(&self) -> Vec<&dyn RenderFragment> {
		vec![self]
	}
}
