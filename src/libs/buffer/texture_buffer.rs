use image::DynamicImage;
use wgpu::{
	BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
	BindingResource, BindingType, ShaderStages, StorageTextureAccess, TextureAspect, TextureDimension, TextureFormat,
	TextureUsages, TextureViewDimension,
};

use super::{ShaderBufferDescriptor, TextureBufferDescriptor};
use crate::{
	core::gpu::Gpu,
	libs::{
		smart_arc::Sarc,
		texture::{self, TextureAsset, TextureAssetDescriptor, TextureAssetDimensions},
	},
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct TextureBuffer<'a> {
	pub var_name: String,
	pub access: StorageTextureAccess,
	pub backing: TextureBufferBacking<'a>,
}

pub enum TextureBufferBacking<'a> {
	WithBacking(Sarc<TextureAsset>),
	New {
		label: &'a str,
		dimensions: TextureAssetDimensions,
		format: TextureFormat,
		usage: Option<TextureUsages>,
		aspect: TextureAspect,
	},
	FromImage {
		label: &'a str,
		image: DynamicImage,
		format: TextureFormat,
		usage: Option<TextureUsages>,
	},
}

impl<'a> TextureBuffer<'a> {
	pub fn new(var_name: impl Into<String>, access: StorageTextureAccess, backing: TextureBufferBacking<'a>) -> Self {
		Self {
			var_name: var_name.into(),
			access,
			backing,
		}
	}

	fn get_dimension(&self) -> TextureDimension {
		match &self.backing {
			TextureBufferBacking::WithBacking(texture) => texture.dimension(),
			TextureBufferBacking::New { dimensions, .. } => dimensions.get_dimension().compatible_texture_dimension(),
			TextureBufferBacking::FromImage { .. } => TextureDimension::D2,
		}
	}

	fn get_view_dimension(&self) -> TextureViewDimension {
		match &self.backing {
			TextureBufferBacking::WithBacking(texture) => texture.view_dimension(),
			TextureBufferBacking::New { dimensions, .. } => dimensions.get_dimension(),
			TextureBufferBacking::FromImage { .. } => TextureViewDimension::D2,
		}
	}

	fn get_format(&self) -> TextureFormat {
		match &self.backing {
			TextureBufferBacking::WithBacking(texture) => texture.format(),
			TextureBufferBacking::New { format, .. } => *format,
			TextureBufferBacking::FromImage { format, .. } => *format,
		}
	}
}

impl ShaderBufferDescriptor for TextureBuffer<'_> {
	fn label(&self, label_type: &str) -> String {
		format!("TextureBuffer \"{}\" {}", self.var_name, label_type)
	}

	fn binding_source_code(&self, group: u32, binding_offset: u32) -> String {
		let dimension = texture::dimension_to_string(self.get_dimension());
		let format = texture::format_to_string(self.get_format());
		let access = texture::access_to_string(self.access);

		format!(
			"@group({}) @binding({}) var {}: texture_storage_{}<{}, {}>;",
			group, binding_offset, self.var_name, dimension, format, access
		)
	}

	fn other_source_code(&self) -> Option<String> {
		None
	}

	fn create_bind_group_layout(&self, gpu: &Gpu, visibility: ShaderStages) -> BindGroupLayout {
		gpu.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
			label: Some(&self.label("Bind Group Layout")),
			entries: &[BindGroupLayoutEntry {
				binding: 0,
				visibility,
				ty: BindingType::StorageTexture {
					access: self.access,
					format: self.get_format(),
					view_dimension: self.get_view_dimension(),
				},
				count: None,
			}],
		})
	}
}

impl TextureBufferDescriptor for TextureBuffer<'_> {
	fn create_bind_group(&self, gpu: &Gpu, layout: &BindGroupLayout, texture: &TextureAsset) -> BindGroup {
		gpu.device.create_bind_group(&BindGroupDescriptor {
			label: Some(&self.label("Bind Group")),
			layout,
			entries: &[BindGroupEntry {
				binding: 0,
				resource: BindingResource::TextureView(&texture.view),
			}],
		})
	}

	fn create_texture(&self, gpu: &Gpu) -> Sarc<TextureAsset> {
		match &self.backing {
			TextureBufferBacking::WithBacking(texture) => texture.clone(),

			TextureBufferBacking::New {
				label,
				dimensions,
				format,
				usage,
				aspect,
			} => Sarc::new(TextureAsset::create(
				gpu,
				TextureAssetDescriptor {
					label,
					dimensions: *dimensions,
					format: *format,
					usage: *usage,
					aspect: *aspect,
				},
			)),

			TextureBufferBacking::FromImage {
				label,
				image,
				format,
				usage,
			} => Sarc::new(TextureAsset::from_image_storage(gpu, label, image, *format, *usage)),
		}
	}

	fn is_output_texture(&self) -> bool {
		matches!(
			self.access,
			StorageTextureAccess::ReadWrite | StorageTextureAccess::WriteOnly
		)
	}
}
