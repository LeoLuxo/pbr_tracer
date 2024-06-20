use brainrot::{bevy, path, rooted_path, Shader};

use crate::rendering::compute::{RenderFragment, Renderer};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct PhysBasedRaytracer;
impl Renderer for PhysBasedRaytracer {}

impl RenderFragment for PhysBasedRaytracer {
	fn shader() -> impl Into<Shader> {
		path!("pathtracer.wgsl")
	}
}
