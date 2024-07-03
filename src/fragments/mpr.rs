use brainrot::path;

use super::post_processing::PostProcessingPipeline;
use crate::libs::{
	shader::{Shader, ShaderBuilder},
	shader_fragment::{Renderer, ShaderFragment},
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
pub trait Shading<I: Intersector>: ShaderFragment {}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct MultiPurposeRenderer<I, S>
where
	I: Intersector,
	S: Shading<I>,
{
	pub intersector: I,
	pub shading: S,
	pub post_processing: PostProcessingPipeline,
}

impl<I, S> Renderer for MultiPurposeRenderer<I, S>
where
	I: Intersector,
	S: Shading<I>,
{
}

impl<I, S> ShaderFragment for MultiPurposeRenderer<I, S>
where
	I: Intersector,
	S: Shading<I>,
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
