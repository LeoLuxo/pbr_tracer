use crate::core::shader::Shader;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub trait ShaderFragment: Sync + Send {
	fn shader(&self) -> Shader;
}

/// Shader API:\
/// `fn render_pixel(pixel_coord: vec2f, pixel_size: vec2f) -> vec4f`
pub trait Renderer: ShaderFragment {}
