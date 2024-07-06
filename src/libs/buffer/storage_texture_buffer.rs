use std::sync::Arc;

use image::DynamicImage;
use wgpu::{
	BindingResource, BindingType, Features, StorageTextureAccess, TextureAspect, TextureDimension, TextureFormat,
	TextureUsages, TextureViewDimension,
};

use super::{ShaderBufferDescriptor, ShaderBufferResource};
use crate::{
	core::gpu::Gpu,
	libs::{
		buffer::PartialLayoutEntry,
		smart_arc::Sarc,
		texture::{self, Tex, TexDescriptor, TextureAssetDimensions},
	},
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub enum StorageTexture<S: Into<String> + Clone> {
	New {
		var_name: S,
		access: StorageTextureAccess,
		dimensions: TextureAssetDimensions,
		format: TextureFormat,
		usage: Option<TextureUsages>,
		aspect: TextureAspect,
	},
	FromImage {
		var_name: S,
		access: StorageTextureAccess,
		image: DynamicImage,
		format: TextureFormat,
		usage: Option<TextureUsages>,
	},
	FromTex {
		var_name: S,
		access: StorageTextureAccess,
		tex: Sarc<Tex>,
	},
}

impl<S: Into<String> + Clone> ShaderBufferDescriptor for StorageTexture<S> {
	fn as_resource(&self, gpu: &Gpu) -> Sarc<dyn ShaderBufferResource> {
		let resource = match self {
			StorageTexture::New {
				var_name,
				access,
				dimensions,
				format,
				usage,
				aspect,
			} => {
				let var_name = var_name.to_owned().into();

				let tex = Sarc::new(Tex::create(
					gpu,
					TexDescriptor {
						label: &format!("StorageTexture '{}'", var_name),
						dimensions: *dimensions,
						format: *format,
						usage: *usage,
						aspect: *aspect,
					},
					None,
				));

				StorageTextureResource {
					tex,
					var_name,
					access: *access,
					dimension: dimensions.get_dimension().compatible_texture_dimension(),
					view_dimension: dimensions.get_dimension(),
					format: *format,
				}
			}

			StorageTexture::FromImage {
				var_name,
				access,
				image,
				format,
				usage,
			} => {
				let var_name = var_name.to_owned().into();
				let tex = Sarc::new(Tex::from_image(
					gpu,
					&format!("StorageTexture '{}'", var_name),
					image,
					*format,
					*usage,
					None,
				));

				StorageTextureResource {
					tex,
					var_name,
					access: *access,
					dimension: TextureDimension::D2,
					view_dimension: TextureViewDimension::D2,
					format: *format,
				}
			}

			StorageTexture::FromTex { var_name, access, tex } => StorageTextureResource {
				tex: tex.clone(),
				var_name: var_name.to_owned().into(),
				access: *access,
				dimension: tex.dimension(),
				view_dimension: tex.view_dimension(),
				format: tex.format(),
			},
		};

		Sarc(Arc::new(resource) as Arc<dyn ShaderBufferResource>)
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct StorageTextureResource {
	pub tex: Sarc<Tex>,
	pub var_name: String,
	pub access: StorageTextureAccess,
	pub dimension: TextureDimension,
	pub view_dimension: TextureViewDimension,
	pub format: TextureFormat,
}

impl ShaderBufferResource for StorageTextureResource {
	fn binding_source_code(&self, group: u32, binding: u32) -> Vec<String> {
		let dimension = texture::dimension_to_string(self.dimension);
		let format = texture::format_to_string(self.format);
		let access = texture::access_to_string(self.access);

		vec![format!(
			"@group({}) @binding({}) var {}: texture_storage_{}<{}, {}>;",
			group, binding, self.var_name, dimension, format, access
		)]
	}

	fn other_source_code(&self) -> Option<&str> {
		None
	}

	fn layouts(&self, _features: Features) -> Vec<PartialLayoutEntry> {
		vec![PartialLayoutEntry {
			ty: BindingType::StorageTexture {
				access: self.access,
				format: self.format,
				view_dimension: self.view_dimension,
			},
			count: None,
		}]
	}

	fn binding_resources(&self) -> Vec<BindingResource> {
		vec![BindingResource::TextureView(&self.tex.view)]
	}
}
