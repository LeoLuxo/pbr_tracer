use brainrot::{path, vek::Extent2};
use wgpu::{TextureAspect, TextureFormat, TextureUsages};

use super::post_processing::PostProcessingPipeline;
use crate::libs::{
	shader::{Shader, ShaderBuilder},
	shader_fragment::{Renderer, ShaderFragment},
	texture::{TexDescriptor, TextureAssetDimensions},
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

/// Shader API:\
/// `fn intersect_scene(ray_origin: vec3f, ray_dir: vec3f) -> Intersection`
pub trait Intersector: ShaderFragment {}

/// Shader API:\
/// `fn shade(intersection: Intersection) -> vec4f`
pub trait Shading: ShaderFragment {}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct MultiPurposeRenderer<I, S>
where
	I: Intersector,
	S: Shading,
{
	pub intersector: I,
	pub shading: S,
	pub post_processing: PostProcessingPipeline,
}

impl<I, S> Renderer for MultiPurposeRenderer<I, S>
where
	I: Intersector,
	S: Shading,
{
	fn output_textures(&self, resolution: Extent2<u32>) -> Vec<(String, TexDescriptor)> {
		let depth = TexDescriptor {
			label: "Depth output texture",
			dimensions: TextureAssetDimensions::D2(resolution),
			format: TextureFormat::Rgba32Float,
			usage: Some(TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC),
			aspect: TextureAspect::All,
		};

		let normal = TexDescriptor {
			label: "Normal output texture",
			dimensions: TextureAssetDimensions::D2(resolution),
			format: TextureFormat::Rgba32Float,
			usage: Some(TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC),
			aspect: TextureAspect::All,
		};

		std::vec![
			("output_color".to_string(), self.default_color_texture(resolution)),
			("output_normal".to_string(), normal),
			("output_depth".to_string(), depth),
		]
	}
}

impl<I, S> ShaderFragment for MultiPurposeRenderer<I, S>
where
	I: Intersector,
	S: Shading,
{
	fn shader(&self) -> Shader {
		ShaderBuilder::new()
			.include_path("mpr.wgsl")
			.include(self.intersector.shader())
			.include(self.shading.shader())
			.include(self.post_processing.shader())
			.into()
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct DebugRenderer;

impl Renderer for DebugRenderer {}
impl ShaderFragment for DebugRenderer {
	fn shader(&self) -> Shader {
		path!("/debug.wgsl").into()
	}
}
