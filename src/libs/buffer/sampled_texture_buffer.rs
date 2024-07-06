use std::sync::Arc;

use image::DynamicImage;
use wgpu::{
	BindingResource, BindingType, CompareFunction, Features, FilterMode, SamplerBindingType, TextureAspect,
	TextureDimension, TextureFormat, TextureUsages, TextureViewDimension,
};

use super::{ShaderBufferDescriptor, ShaderBufferResource};
use crate::{
	core::gpu::Gpu,
	libs::{
		buffer::PartialLayoutEntry,
		smart_arc::Sarc,
		texture::{self, SamplerEdges, Tex, TexDescriptor, TexSamplerDescriptor, TextureAssetDimensions},
	},
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub enum SampledTexture<S: Into<String> + Clone> {
	New {
		texture_var_name: S,
		sampler_var_name: S,
		dimensions: TextureAssetDimensions,
		format: TextureFormat,
		usage: Option<TextureUsages>,
		aspect: TextureAspect,
		filter: FilterMode,
		edges: SamplerEdges,
		compare: Option<CompareFunction>,
	},
	FromImage {
		texture_var_name: S,
		sampler_var_name: S,
		image: DynamicImage,
		format: TextureFormat,
		usage: Option<TextureUsages>,
		filter: FilterMode,
		edges: SamplerEdges,
		compare: Option<CompareFunction>,
	},
	FromTex {
		texture_var_name: S,
		sampler_var_name: S,
		tex: Sarc<Tex>,
	},
}

impl<S: Into<String> + Clone> ShaderBufferDescriptor for SampledTexture<S> {
	fn as_resource(&self, gpu: &Gpu) -> Sarc<dyn ShaderBufferResource> {
		let resource = match self {
			SampledTexture::New {
				texture_var_name,
				sampler_var_name,
				dimensions,
				format,
				usage,
				aspect,
				filter,
				edges,
				compare,
			} => {
				let texture_var_name = texture_var_name.to_owned().into();
				let sampler_var_name = sampler_var_name.to_owned().into();

				let tex = Sarc::new(Tex::create(
					gpu,
					TexDescriptor {
						label: &format!("SampledTexture '{}/{}'", texture_var_name, sampler_var_name),
						dimensions: *dimensions,
						format: *format,
						usage: *usage,
						aspect: *aspect,
					},
					Some(TexSamplerDescriptor {
						filter: *filter,
						edges: *edges,
						compare: *compare,
					}),
				));

				SampledTextureResource {
					tex,
					texture_var_name,
					sampler_var_name,
					dimension: dimensions.get_dimension().compatible_texture_dimension(),
					view_dimension: dimensions.get_dimension(),
					format: *format,
				}
			}

			SampledTexture::FromImage {
				texture_var_name,
				sampler_var_name,
				image,
				format,
				usage,
				filter,
				edges,
				compare,
			} => {
				let texture_var_name = texture_var_name.to_owned().into();
				let sampler_var_name = sampler_var_name.to_owned().into();

				let tex = Sarc::new(Tex::from_image(
					gpu,
					&format!("SampledTexture '{}/{}'", texture_var_name, sampler_var_name),
					image,
					*format,
					*usage,
					Some(TexSamplerDescriptor {
						filter: *filter,
						edges: *edges,
						compare: *compare,
					}),
				));

				SampledTextureResource {
					tex,
					texture_var_name,
					sampler_var_name,
					dimension: TextureDimension::D2,
					view_dimension: TextureViewDimension::D2,
					format: *format,
				}
			}

			SampledTexture::FromTex {
				texture_var_name,
				sampler_var_name,
				tex,
			} => SampledTextureResource {
				tex: tex.clone(),
				texture_var_name: texture_var_name.to_owned().into(),
				sampler_var_name: sampler_var_name.to_owned().into(),
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

pub struct SampledTextureResource {
	pub tex: Sarc<Tex>,
	pub texture_var_name: String,
	pub sampler_var_name: String,
	pub dimension: TextureDimension,
	pub view_dimension: TextureViewDimension,
	pub format: TextureFormat,
}

impl ShaderBufferResource for SampledTextureResource {
	fn binding_source_code(&self, group: u32, binding: u32) -> Vec<String> {
		let dimension = texture::dimension_to_string(self.dimension);
		let sample_type = texture::format_to_type_string(self.format);

		vec![
			format!(
				"@group({}) @binding({}) var {}: texture_{}<{}>;",
				group, binding, self.texture_var_name, dimension, sample_type
			),
			format!(
				"@group({}) @binding({}) var {}: sampler;",
				group,
				binding + 1,
				self.sampler_var_name
			),
		]
	}

	fn other_source_code(&self) -> Option<&str> {
		None
	}

	fn layouts(&self, features: Features) -> Vec<PartialLayoutEntry> {
		vec![
			PartialLayoutEntry {
				ty: BindingType::Texture {
					sample_type: self
						.format
						.sample_type(None, Some(features))
						.expect("Incompatible format"),
					view_dimension: self.view_dimension,
					multisampled: false,
				},
				count: None,
			},
			PartialLayoutEntry {
				ty: BindingType::Sampler(SamplerBindingType::Filtering),
				count: None,
			},
		]
	}

	fn binding_resources(&self) -> Vec<BindingResource> {
		vec![
			BindingResource::TextureView(&self.tex.view),
			BindingResource::Sampler(
				self.tex
					.sampler
					.as_ref()
					.expect("Cannot use a TextureAsset without a sampler for a TextureSamplerBuffer"),
			),
		]
	}
}
