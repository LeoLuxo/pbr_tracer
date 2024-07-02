use wgpu::{
	util::{BufferInitDescriptor, DeviceExt},
	BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
	BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, ShaderStages,
};

use super::{DataBufferDescriptor, DataBufferUploadable, ShaderBufferDescriptor};
use crate::{
	core::gpu::Gpu,
	libs::{buffer::ShaderType, smart_arc::Sarc},
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct UniformBuffer<T: DataBufferUploadable + ShaderType> {
	pub var_name: String,
	pub backing: UniformBufferBacking<T>,
}

pub enum UniformBufferBacking<T: DataBufferUploadable + ShaderType> {
	NewSized(u64),
	FromData(T),
	FromBuffer(Sarc<Buffer>),
}

impl<T: DataBufferUploadable + ShaderType> UniformBuffer<T> {
	pub fn new(var_name: impl Into<String>, size: u64) -> Self {
		Self {
			var_name: var_name.into(),
			backing: UniformBufferBacking::NewSized(size),
		}
	}

	pub fn from_data(var_name: impl Into<String>, data: T) -> Self {
		Self {
			var_name: var_name.into(),
			backing: UniformBufferBacking::FromData(data),
		}
	}

	pub fn from_buffer(var_name: impl Into<String>, buffer: Sarc<Buffer>) -> Self {
		Self {
			var_name: var_name.into(),
			backing: UniformBufferBacking::FromBuffer(buffer),
		}
	}
}

impl<T: DataBufferUploadable + ShaderType> ShaderBufferDescriptor for UniformBuffer<T> {
	fn label(&self, label_type: &str) -> String {
		format!(
			"UniformBuffer<{}> \"{}\" {}",
			<T as ShaderType>::type_name(),
			self.var_name,
			label_type
		)
	}

	fn binding_source_code(&self, group: u32, binding_offset: u32) -> String {
		format!(
			"@group({}) @binding({}) var<uniform> {}: {};",
			group,
			binding_offset,
			self.var_name,
			<T as ShaderType>::type_name()
		)
	}

	fn other_source_code(&self) -> Option<String> {
		<T as ShaderType>::struct_definition()
	}

	fn create_bind_group_layout(&self, gpu: &Gpu, visibility: ShaderStages) -> BindGroupLayout {
		gpu.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
			label: Some(&self.label("Bind Group Layout")),
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
		})
	}
}

impl<T: DataBufferUploadable + ShaderType> DataBufferDescriptor for UniformBuffer<T> {
	fn create_bind_group(&self, gpu: &Gpu, layout: &BindGroupLayout, buffer: &Buffer) -> BindGroup {
		gpu.device.create_bind_group(&BindGroupDescriptor {
			label: Some(&self.label("Bind Group")),
			layout,
			entries: &[BindGroupEntry {
				binding: 0,
				resource: buffer.as_entire_binding(),
			}],
		})
	}

	fn create_buffer(&self, gpu: &Gpu) -> Sarc<Buffer> {
		match &self.backing {
			UniformBufferBacking::NewSized(size) => {
				let buffer = gpu.device.create_buffer(&BufferDescriptor {
					label: Some(&self.label("Buffer")),
					size: *size,
					usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
					mapped_at_creation: false,
				});

				Sarc::new(buffer)
			}

			UniformBufferBacking::FromData(data) => {
				let buffer = gpu.device.create_buffer_init(&BufferInitDescriptor {
					label: Some(&self.label("Buffer")),
					contents: &data.get_bytes(),
					usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
				});

				Sarc::new(buffer)
			}

			UniformBufferBacking::FromBuffer(buffer) => buffer.clone(),
		}
	}
}
