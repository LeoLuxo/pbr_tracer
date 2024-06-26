use std::{
	collections::HashMap,
	fmt,
	hash::{Hash, Hasher},
	marker::PhantomData,
	mem,
	sync::Arc,
};

use bevy_ecs::system::{Query, Res};
use brainrot::bevy::{self, App};
use wgpu::{
	BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
	BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, ComputePass, Device, RenderPass,
	ShaderStages,
};

use super::{gameloop::PreRender, gpu::Gpu};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub trait UniformBuffer: std::fmt::Debug {
	fn get_source_code(&self, group: u32, binding: u32, name: &str) -> String;
	fn get_size(&self) -> u64;
	fn get_data(&self) -> Vec<u8>;
}

pub trait Bufferable: Default + UniformBuffer + bytemuck::Pod {}

#[derive(derive_more::Deref, derive_more::DerefMut, Clone)]
pub struct UniformBufferArc(Arc<dyn UniformBuffer>);

impl UniformBufferArc {
	pub fn new<U>(uniform: U) -> Self
	where
		U: Bufferable,
	{
		Self(Arc::new(uniform) as Arc<dyn UniformBuffer>)
	}
}

impl PartialEq for UniformBufferArc {
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.0, &other.0)
	}
}

impl Eq for UniformBufferArc {}

impl Hash for UniformBufferArc {
	fn hash<H>(&self, hasher: &mut H)
	where
		H: Hasher,
	{
		// Voodoo magic, but basically we're hashing using the numeric value of the
		// pointer of the Arc
		hasher.write_usize(Arc::as_ptr(&self.0) as *const () as usize);
	}
}

impl std::fmt::Debug for UniformBufferArc {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple("UniformBufferArc").field(&self.0).finish()
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(bevy::Component, Debug)]
pub struct SizedUniformBuffer<T>
where
	T: Sized + bytemuck::Pod,
{
	pub buffer: Buffer,
	pub bind_group_layout: Arc<BindGroupLayout>,
	pub bind_group: Arc<BindGroup>,
	_marker: PhantomData<T>,
}

impl<T> SizedUniformBuffer<T>
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

		SizedUniformBuffer::<T> {
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

#[derive(Debug, Default)]
pub struct BindGroupMapping(pub HashMap<u32, BindGroup>);

impl<'a> BindGroupMapping {
	pub fn apply_to_render_pass(&'a self, render_pass: &mut RenderPass<'a>) {
		for (index, bind_group) in &self.0 {
			render_pass.set_bind_group(*index, bind_group, &[]);
		}
	}
	pub fn apply_to_compute_pass(&'a self, compute_pass: &mut ComputePass<'a>) {
		for (index, bind_group) in &self.0 {
			compute_pass.set_bind_group(*index, bind_group, &[]);
		}
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub fn create_buffer(gpu: &Gpu, name: &str, size: u64) -> Buffer {
	gpu.device.create_buffer(&BufferDescriptor {
		label: Some(&format!("{name} Buffer")),
		size,
		usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
		mapped_at_creation: false,
	})
}

pub fn create_bind_group_layout(gpu: &Gpu, name: &str, visibility: ShaderStages) -> BindGroupLayout {
	gpu.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
	})
}

pub fn create_bind_group(gpu: &Gpu, name: &str, buffer: &Buffer, layout: &BindGroupLayout) -> BindGroup {
	gpu.device.create_bind_group(&BindGroupDescriptor {
		layout,
		entries: &[BindGroupEntry {
			binding: 0,
			resource: buffer.as_entire_binding(),
		}],
		label: Some(&format!("{name} Bindgroup")),
	})
}

pub fn upload_buffer_bytes(gpu: &Gpu, buffer: &Buffer, bytes: &[u8]) {
	gpu.queue.write_buffer(buffer, 0, bytes);
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

fn upload_buffers<T>(gpu: Res<Gpu>, q: Query<(&T, &SizedUniformBuffer<T>)>)
where
	T: bytemuck::Pod + bevy::Component + Send + Sync,
{
	for (uniform, uniform_buffer) in q.iter() {
		gpu.queue
			.write_buffer(&uniform_buffer.buffer, 0, bytemuck::bytes_of(uniform));
	}
}
