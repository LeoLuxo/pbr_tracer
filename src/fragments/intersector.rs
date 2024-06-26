use std::mem;

use brainrot::bevy;

use super::shader_fragments::ShaderFragment;
use crate::core::{
	buffer::{Bufferable, UniformBuffer},
	shader::{Shader, ShaderBuilder},
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
#[derive(bevy::Component, bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug, PartialEq)]
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

impl Bufferable for RaymarchSettings {}
impl UniformBuffer for RaymarchSettings {
	fn get_source_code(&self, group: u32, binding: u32, name: &str) -> String {
		format!(
			r#"
			struct RaymarchSettings {{
				epsilon: f32,
				min_march: f32,
				max_march: f32,
				max_march_steps: u32,
			}};
			@group({group}) @binding({binding}) var<uniform> {name}: RaymarchSettings;
		"#
		)
	}

	fn get_size(&self) -> u64 {
		mem::size_of::<Self>() as u64
	}

	fn get_data(&self) -> Vec<u8> {
		bytemuck::bytes_of(self).to_owned()
	}
}

impl Intersector for Raymarcher {}
impl ShaderFragment for Raymarcher {
	fn shader(&self) -> Shader {
		ShaderBuilder::new()
			.include_path("raymarch/raymarch.wgsl")
			.include_uniform("settings", RaymarchSettings::default())
			.into()
	}
}
