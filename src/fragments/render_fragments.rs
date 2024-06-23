use brainrot::Shader;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub trait RenderFragment<Properties = ()>: Sync + Send
where
	Properties: Default,
{
	fn shader(&self) -> Shader;
	fn fragments(&self) -> Vec<&dyn RenderFragment>;
	fn properties(&self) -> Properties {
		Properties::default()
	}
}

/// Shader API:\
/// `fn render_pixel(pixel_coord: vec2f, pixel_size: vec2f) -> vec4f`
pub trait Renderer: RenderFragment {}
