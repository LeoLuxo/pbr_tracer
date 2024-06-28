// use pbr_tracer_derive::ShaderType;

use pbr_tracer_derive::ShaderStruct;

use crate::libs::{
	buffer::ShaderType,
	shader::{Shader, ShaderBuilder},
	shader_fragment::ShaderFragment,
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

/// Shader API:\
/// `fn send_ray(ray_origin: vec3f, ray_dir: vec3f) -> vec4f`
pub trait Intersector: ShaderFragment {}

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
			.include_uniform_buffer("settings", RaymarchSettings::default())
			.into()
	}
}
