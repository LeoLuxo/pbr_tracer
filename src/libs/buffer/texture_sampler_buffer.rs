use wgpu::{
	BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
	BindingResource, BindingType, ShaderStages, TextureDimension, TextureFormat, TextureViewDimension,
};

use super::{ShaderBufferDescriptor, TextureBufferDescriptor};
use crate::{
	core::gpu::Gpu,
	libs::{
		smart_arc::Sarc,
		texture::{self, TextureAsset, TextureAssetDescriptor, TextureAssetSamplerDescriptor},
	},
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct TextureSamplerBuffer<'a> {
	pub texture_var_name: String,
	pub sampler_var_name: String,
	pub backing: TextureBufferBacking<'a>,
}

pub enum TextureBufferBacking<'a> {
	New(TextureAssetDescriptor<'a>, TextureAssetSamplerDescriptor),
	From(Sarc<TextureAsset>),
}

impl<'a> TextureSamplerBuffer<'a> {
	pub fn new(
		texture_var_name: impl Into<String>,
		sampler_var_name: impl Into<String>,
		backing: TextureBufferBacking<'a>,
	) -> Self {
		Self {
			texture_var_name: texture_var_name.into(),
			sampler_var_name: sampler_var_name.into(),
			backing,
		}
	}

	fn get_dimension(&self) -> TextureDimension {
		match &self.backing {
			TextureBufferBacking::New(desc, _) => desc.dimensions.get_dimension().compatible_texture_dimension(),
			TextureBufferBacking::From(texture) => texture.dimension(),
		}
	}

	fn get_view_dimension(&self) -> TextureViewDimension {
		match &self.backing {
			TextureBufferBacking::New(desc, _) => desc.dimensions.get_dimension(),
			TextureBufferBacking::From(texture) => texture.view_dimension(),
		}
	}

	fn get_format(&self) -> TextureFormat {
		match &self.backing {
			TextureBufferBacking::New(desc, _) => desc.format,
			TextureBufferBacking::From(texture) => texture.format(),
		}
	}
}

impl ShaderBufferDescriptor for TextureSamplerBuffer<'_> {
	fn label(&self, label_type: &str) -> String {
		format!(
			"TextureSamplerBuffer \"{}/{}\" {}",
			self.texture_var_name, self.sampler_var_name, label_type
		)
	}

	fn binding_source_code(&self, group: u32, binding_offset: u32) -> String {
		let dimension = texture::dimension_to_string(self.get_dimension());
		let sample_type = texture::format_to_type_string(self.get_format());

		let line1 = format!(
			"@group({}) @binding({}) var {}: texture_{}<{}>;",
			group, binding_offset, self.texture_var_name, dimension, sample_type
		);

		let line2 = format!(
			"@group({}) @binding({}) var {}: sampler;",
			group,
			binding_offset + 1,
			self.sampler_var_name
		);

		line1 + "\n" + &line2
	}

	fn other_source_code(&self) -> Option<String> {
		None
	}

	fn create_bind_group_layout(&self, gpu: &Gpu, visibility: ShaderStages) -> BindGroupLayout {
		gpu.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
			label: Some(&self.label("Bind Group Layout")),
			entries: &[
				BindGroupLayoutEntry {
					binding: 0,
					visibility,
					ty: BindingType::Texture {
						sample_type: self
							.get_format()
							.sample_type(None, Some(gpu.device.features()))
							.expect("Incompatible format"),
						view_dimension: self.get_view_dimension(),
						multisampled: false,
					},
					count: None,
				},
				BindGroupLayoutEntry {
					binding: 1,
					visibility,
					ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
			],
		})
	}
}

impl TextureBufferDescriptor for TextureSamplerBuffer<'_> {
	fn create_bind_group(&self, gpu: &Gpu, layout: &BindGroupLayout, texture: &TextureAsset) -> BindGroup {
		gpu.device.create_bind_group(&BindGroupDescriptor {
			label: Some(&self.label("Bind Group")),
			layout,
			entries: &[
				BindGroupEntry {
					binding: 0,
					resource: BindingResource::TextureView(&texture.view),
				},
				BindGroupEntry {
					binding: 1,
					resource: BindingResource::Sampler(
						texture
							.sampler
							.as_ref()
							.expect("Cannot use a TextureAsset without a sampler for a TextureSamplerBuffer"),
					),
				},
			],
		})
	}

	fn create_texture(&self, gpu: &Gpu) -> Sarc<TextureAsset> {
		match &self.backing {
			TextureBufferBacking::New(desc, sampler_desc) => {
				Sarc::new(TextureAsset::create_with_sampler(gpu, *desc, *sampler_desc))
			}
			TextureBufferBacking::From(texture) => texture.clone(),
		}
	}
}
