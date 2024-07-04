use brainrot::vek::Extent2;
use wgpu::{Texture, TextureAspect, TextureFormat, TextureUsages};

use super::{
	buffer::texture_buffer::TextureBuffer,
	smart_arc::Sarc,
	texture::{Tex, TexDescriptor, TextureAssetDimensions},
};
use crate::libs::shader::Shader;

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
pub trait Renderer: ShaderFragment {
	fn default_color_texture(resolution: Extent2<u32>) -> TextureBuffer<'static> {
		TexDescriptor {
			label: "Default Renderer output texture",
			dimensions: TextureAssetDimensions::D2(resolution),
			format: TextureFormat::Rgba32Float,
			usage: Some(TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC),
			aspect: TextureAspect::All,
		}
	}

	fn output_textures(&self, resolution: Extent2<u32>) -> Vec<TexDescriptor> {
		vec![Self::default_color_texture(resolution)]
	}
}
