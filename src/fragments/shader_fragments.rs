
use crate::core::shader::{Shader, ShaderBuilder};

use crate::core::buffer::BufferRegistrar;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub trait ShaderFragment: Sync + Send {
	fn shader(&self) -> Shader;
	fn fragments(&self) -> Vec<&dyn ShaderFragment>;
	fn declare_buffers(&self, _buffers: &mut BufferRegistrar) {}
}

/// Shader API:\
/// `fn render_pixel(pixel_coord: vec2f, pixel_size: vec2f) -> vec4f`
pub trait Renderer: ShaderFragment {}
