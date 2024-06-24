use brainrot::{
	bevy::{self, App},
	Shader,
};
use wgpu::{Device, ShaderStages};

use crate::core::buffer::{self, UniformBuffer};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct Buffers<'a> {
	app: &'a mut App,
	device: &'a Device,
}

impl<'a> Buffers<'a> {
	pub fn add_uniform_buffer<T>(&mut self, data: T)
	where
		T: bytemuck::Pod + bevy::Component + Send + Sync,
	{
		buffer::register_uniform::<T>(self.app);

		let buffer_bundle = (
			data,
			UniformBuffer::<T>::new(self.device, std::any::type_name::<T>(), ShaderStages::COMPUTE),
		);

		self.app.world.spawn(buffer_bundle);
	}
}

pub trait ShaderFragment: Sync + Send {
	fn shader(&self) -> Shader;
	fn fragments(&self) -> Vec<&dyn ShaderFragment>;
	fn declare_buffers(&self, buffers: Buffers) {}
}

/// Shader API:\
/// `fn render_pixel(pixel_coord: vec2f, pixel_size: vec2f) -> vec4f`
pub trait Renderer: ShaderFragment {}
