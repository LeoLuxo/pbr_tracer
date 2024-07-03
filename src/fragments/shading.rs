use brainrot::vec3;

use super::mpr::{Intersector, Shading};
use crate::libs::{
	shader::{Shader, ShaderBuilder},
	shader_fragment::ShaderFragment,
};

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
