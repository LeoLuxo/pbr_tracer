use brainrot::vec3;
use wgpu::{StorageTextureAccess, TextureFormat};

use super::mpr::Shading;
use crate::{
	libs::{
		buffer::storage_texture_buffer::{StorageTextureBuffer, StorageTextureBufferBacking},
		shader::{Shader, ShaderBuilder},
		shader_fragment::ShaderFragment,
	},
	TextureAssets,
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct SimpleDiffuse;

impl Shading for SimpleDiffuse {}
impl ShaderFragment for SimpleDiffuse {
	fn shader(&self) -> Shader {
		ShaderBuilder::new()
			.include_path("/shading/simple_diffuse.wgsl")
			.include_value("sun_direction", vec3!(1.0, -1.0, 1.0).normalized())
			.into()
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct CelShading;

impl Shading for CelShading {}
impl ShaderFragment for CelShading {
	fn shader(&self) -> Shader {
		let gradient = StorageTextureBuffer::new(
			"cel_gradient",
			StorageTextureAccess::ReadOnly,
			StorageTextureBufferBacking::FromImage {
				label: "cel_gradient",
				image: TextureAssets::get_image("cel_gradient.png"),
				format: TextureFormat::Rgba8Unorm,
				usage: None,
			},
		);

		ShaderBuilder::new()
			.include_path("/shading/cel_shading.wgsl")
			.include_value("sun_direction", vec3!(1.0, -1.0, 0.0).normalized())
			.include_texture(gradient)
			.into()
	}
}
