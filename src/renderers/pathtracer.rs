use brainrot::{path, Shader, ShaderBuilder};

use crate::core::rendering::compute::{RenderFragment, Renderer};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct PhysBasedRaytracer;
impl Renderer for PhysBasedRaytracer {}

impl RenderFragment for PhysBasedRaytracer {
	fn shader() -> impl Into<Shader> {
		ShaderBuilder::new()
			.include_path("pathtracer.wgsl")
			.include_path("raymarch/raymarch.wgsl")
	}
}
