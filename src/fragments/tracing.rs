use brainrot::{path, Shader};

use super::shader_fragments::ShaderFragment;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

/// Shader API:\
/// `fn send_ray(ray_origin: vec3f, ray_dir: vec3f) -> vec4f`
pub trait Tracer: ShaderFragment {}

pub struct Raymarcher;

impl Tracer for Raymarcher {}

impl ShaderFragment for Raymarcher {
	fn shader(&self) -> Shader {
		path!("raymarch/raymarch.wgsl").into()
	}

	fn fragments(&self) -> Vec<&dyn ShaderFragment> {
		vec![self]
	}
}
