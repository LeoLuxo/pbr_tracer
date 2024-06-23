use brainrot::{path, Shader};

use super::render_fragments::RenderFragment;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

/// Shader API:\
/// `fn send_ray(ray_origin: vec3f, ray_dir: vec3f) -> vec4f`
pub trait Tracer: RenderFragment {}

pub struct Raymarcher;

impl Tracer for Raymarcher {}

impl RenderFragment for Raymarcher {
	fn shader(&self) -> Shader {
		path!("raymarch/raymarch.wgsl").into()
	}

	fn fragments(&self) -> Vec<&dyn RenderFragment> {
		vec![self]
	}
}
