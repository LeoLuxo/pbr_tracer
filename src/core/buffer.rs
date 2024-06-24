use std::{marker::PhantomData, mem, sync::Arc};

use bevy_ecs::system::{Query, Res};
use brainrot::bevy::{self, App};
use wgpu::{
	BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
	BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, Device, ShaderStages,
};

use super::{gameloop::PreRender, gpu::Gpu};

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
	pub bind_group_layout: Arc<BindGroupLayout>,
	pub bind_group: Arc<BindGroup>,
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
			bind_group_layout: Arc::new(bind_group_layout),
			bind_group: Arc::new(bind_group),
			_marker: Default::default(),
		}
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(bevy::Bundle)]
pub struct BufferBundle<T>
where
	T: bevy::Component + bytemuck::Pod,
{
	data: T,
	buffer: UniformBuffer<T>,
}

pub struct BufferRegistrar<'a> {
	app: &'a mut App,
	device: &'a Device,
	shader_stages: ShaderStages,
	bind_group_offset: u32,
	bind_group_layouts: Vec<Arc<BindGroupLayout>>,
	bind_groups: Vec<Arc<BindGroup>>,
}

impl<'a> BufferRegistrar<'a> {
	pub fn new(app: &'a mut App, device: &'a mut Device, bind_group_offset: u32, shader_stages: ShaderStages) -> Self {
		Self {
			app,
			device,
			bind_group_offset,
			shader_stages,
			bind_group_layouts: Vec::new(),
			bind_groups: Vec::new(),
		}
	}

	fn add_uniform_buffer<T>(&mut self, data: T)
	where
		T: bytemuck::Pod + bevy::Component + Send + Sync,
	{
		register_uniform::<T>(self.app);

		let buffer = UniformBuffer::<T>::new(self.device, std::any::type_name::<T>(), self.shader_stages);

		self.bind_group_layouts.push(buffer.bind_group_layout.clone());

		let buffer_bundle = BufferBundle { data, buffer };

		self.app.world.spawn(buffer_bundle);
	}

	pub fn bind_group_layouts(&'a self) -> Vec<&'a BindGroupLayout> {
		self.bind_group_layouts.iter().map(|v| v.as_ref()).collect()
	}

	pub fn bind_groups(&'a self) -> Vec<&'a BindGroup> {
		self.bind_groups.iter().map(|v| v.as_ref()).collect()
	}

	pub fn finish<L>(self, label: L) {}
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
