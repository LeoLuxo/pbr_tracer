use brainrot::{path, Shader};

use crate::core::rendering::render_fragments::{RenderFragment, Renderer};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(Clone)]
pub struct DebugRenderer;

impl Renderer for DebugRenderer {}

impl RenderFragment for DebugRenderer {
	fn shader(&self) -> Shader {
		path!("/debug.wgsl").into()
	}
}
