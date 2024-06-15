use std::{marker::PhantomData, mem};

use bevy_ecs::{
	bundle::Bundle,
	system::{Query, Res},
};
use brainrot::bevy::{self, App, Component};
use wgpu::{
	BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
	BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, Device, Queue, ShaderStages,
};

use super::{display::Gpu, gameloop::PreRender};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(bevy::Component, Debug)]
pub struct UniformBuffer<T>
where
	T: Sized + bytemuck::Pod,
{
	pub buffer: Buffer,
	pub bind_group_layout: BindGroupLayout,
	pub bind_group: BindGroup,
	_marker: PhantomData<T>,
}

impl<T> UniformBuffer<T>
where
	T: Sized + bytemuck::Pod,
{
	pub fn new(device: &Device, name: &str, visibility: ShaderStages) -> Self {
		// Create a uniform buffer for data in T
		// In wgpu, uniforms need to be explicitly created as buffers
		let buffer = device.create_buffer(&BufferDescriptor {
			label: Some(&format!("{name} Buffer")),
			size: mem::size_of::<T>() as u64,
			usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
			entries: &[BindGroupLayoutEntry {
				binding: 0,
				visibility,
				ty: BindingType::Buffer {
					ty: BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			}],
			label: Some(&format!("{name} Bindgroup Layout")),
		});

		let bind_group = device.create_bind_group(&BindGroupDescriptor {
			layout: &bind_group_layout,
			entries: &[BindGroupEntry {
				binding: 0,
				resource: buffer.as_entire_binding(),
			}],
			label: Some(&format!("{name} Bindgroup")),
		});

		UniformBuffer::<T> {
			buffer,
			bind_group_layout,
			bind_group,
			_marker: Default::default(),
		}
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub fn register_uniform<T>(app: &mut App)
where
	T: bytemuck::Pod + bevy::Component + Send + Sync,
{
	app.add_systems(PreRender, upload_buffers::<T>);
}

fn upload_buffers<T>(gpu: Res<Gpu>, q: Query<(&T, &UniformBuffer<T>)>)
where
	T: bytemuck::Pod + bevy::Component + Send + Sync,
{
	for (uniform, uniform_buffer) in q.iter() {
		gpu.queue
			.write_buffer(&uniform_buffer.buffer, 0, bytemuck::bytes_of(uniform));
	}
}
