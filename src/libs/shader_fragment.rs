use brainrot::vek::Extent2;
use wgpu::{TextureAspect, TextureFormat, TextureUsages};

use super::texture::{TexDescriptor, TextureAssetDimensions};
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
/// `fn render_pixel(pixel_coord: vec2u, pixel_size: vec2u)`
pub trait Renderer: ShaderFragment {
	fn default_color_texture(&self, resolution: Extent2<u32>) -> TexDescriptor<'static> {
		TexDescriptor {
			label: "Renderer default output texture",
			dimensions: TextureAssetDimensions::D2(resolution),
			format: TextureFormat::Rgba32Float,
			usage: Some(TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC),
			aspect: TextureAspect::All,
		}
	}

	fn output_textures(&self, resolution: Extent2<u32>) -> Vec<(String, TexDescriptor)> {
		vec![("output_color".to_string(), self.default_color_texture(resolution))]
	}
}
