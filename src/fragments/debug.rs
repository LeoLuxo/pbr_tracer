use brainrot::path;

use super::shader_fragments::{Renderer, ShaderFragment};
use crate::core::shader::{Shader, ShaderBuilder};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct DebugRenderer;

impl Renderer for DebugRenderer {}
impl ShaderFragment for DebugRenderer {
	fn shader(&self) -> Shader {
		path!("/debug.wgsl").into()
	}

	fn fragments(&self) -> Vec<&dyn ShaderFragment> {
		vec![self]
	}
}
