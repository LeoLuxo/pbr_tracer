use brainrot::vec3;
use pbr_tracer_derive::ShaderStruct;

use super::post_processing::PostProcessingPipeline;
use crate::libs::{
	buffer::ShaderType,
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

pub struct Raymarcher;

#[repr(C)]
#[derive(ShaderStruct, bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug, PartialEq)]
pub struct RaymarchSettings {
	epsilon: f32,
	min_march: f32,
	max_march: f32,
	max_march_steps: u32,
}

impl Default for RaymarchSettings {
	fn default() -> Self {
		Self {
			epsilon: 0.00001,
			min_march: 0.001,
			max_march: 1000.0,
			max_march_steps: 100,
		}
	}
}

impl Intersector for Raymarcher {}
impl ShaderFragment for Raymarcher {
	fn shader(&self) -> Shader {
		ShaderBuilder::new()
			.include_path("raymarch/raymarch.wgsl")
			.include_value("settings", RaymarchSettings::default())
			.into()
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct SimpleDiffuse;

impl<I: Intersector> Shading<I> for SimpleDiffuse {}
impl ShaderFragment for SimpleDiffuse {
	fn shader(&self) -> Shader {
		ShaderBuilder::new()
			.include_path("/shading/simple_diffuse.wgsl")
			.include_value("sun_direction", vec3!(1.0, -1.0, 1.0).normalized())
			.into()
	}
}
