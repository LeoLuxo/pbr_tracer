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
	util::RenderEncoder, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
	BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, ComputePass, Device,
	RenderPass, ShaderStages,
};

use super::{gameloop::PreRender, gpu::Gpu};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub trait UniformBuffer: std::fmt::Debug {
	fn get_source_code(&self, group: u32, binding: u32, name: String) -> String;
	fn get_size(&self) -> usize;
	fn get_data(&self) -> Vec<u8>;
}

pub trait Bufferable: Default + UniformBuffer + bytemuck::Pod {}

#[derive(derive_more::Deref, derive_more::DerefMut, Clone)]
pub struct UniformBufferArc(Arc<dyn UniformBuffer>);

impl UniformBufferArc {
	pub fn new<U>() -> Self
	where
		U: Bufferable,
	{
		Self(Arc::new(U::default()) as Arc<dyn UniformBuffer>)
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

#[derive(bevy::Bundle)]
pub struct BufferBundle<T>
where
	T: bevy::Component + bytemuck::Pod,
{
	data: T,
	buffer: SizedUniformBuffer<T>,
}

pub struct BindGroupMapping(HashMap<u32, Arc<BindGroup>>);

impl<'a> BindGroupMapping {
	pub fn apply_to_render_pass(&'a self, render_pass: &'a mut RenderPass<'a>) {
		for (index, bind_group) in &self.0 {
			render_pass.set_bind_group(*index, bind_group.as_ref(), &[]);
		}
	}
	pub fn apply_to_compute_pass(&'a self, compute_pass: &mut ComputePass<'a>) {
		for (index, bind_group) in &self.0 {
			compute_pass.set_bind_group(*index, bind_group.as_ref(), &[]);
		}
	}
}

pub struct BufferRegistrar<'a> {
	app: &'a mut App,
	shader_stages: ShaderStages,
	bind_group_offset: u32,
	bind_group_layouts: Vec<Arc<BindGroupLayout>>,
	bind_groups: Vec<Arc<BindGroup>>,
}

impl<'a> BufferRegistrar<'a> {
	pub fn new(app: &'a mut App, bind_group_offset: u32, shader_stages: ShaderStages) -> Self {
		Self {
			app,
			bind_group_offset,
			shader_stages,
			bind_group_layouts: Vec::new(),
			bind_groups: Vec::new(),
		}
	}

	pub fn add_uniform_buffer<T>(&mut self, data: T)
	where
		T: bytemuck::Pod + bevy::Component + Send + Sync,
	{
		register_uniform::<T>(self.app);

		let device = &self.app.world.resource::<Gpu>().device;
		let buffer = SizedUniformBuffer::<T>::new(device, std::any::type_name::<T>(), self.shader_stages);

		self.bind_group_layouts.push(buffer.bind_group_layout.clone());

		let buffer_bundle = BufferBundle { data, buffer };

		self.app.world.spawn(buffer_bundle);
	}

	pub fn bind_group_layouts(&'a self) -> Vec<Arc<BindGroupLayout>> {
		self.bind_group_layouts.clone()
	}

	pub fn bind_groups(&'a self) -> Vec<Arc<BindGroup>> {
		self.bind_groups.clone()
	}

	pub fn bind_group_mapping(&self) -> BindGroupMapping {
		BindGroupMapping(
			self.bind_groups
				.iter()
				.zip(0u32..)
				.map(|(v, i)| (i + self.bind_group_offset, v.clone()))
				.collect(),
		)
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

fn upload_buffers<T>(gpu: Res<Gpu>, q: Query<(&T, &SizedUniformBuffer<T>)>)
where
	T: bytemuck::Pod + bevy::Component + Send + Sync,
{
	for (uniform, uniform_buffer) in q.iter() {
		gpu.queue
			.write_buffer(&uniform_buffer.buffer, 0, bytemuck::bytes_of(uniform));
	}
}
