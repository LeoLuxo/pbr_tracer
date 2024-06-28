use wgpu::{
	BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
	BindingResource, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, ShaderStages,
};

use super::{upload_bytes_to_buffer, DataBufferBounds, DataBufferDescriptor, ShaderBufferDescriptor};
use crate::{core::gpu::Gpu, libs::buffer::ShaderType};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct Uniform<T: DataBufferBounds> {
	pub var_name: String,
	pub data: T,
}

impl<T: DataBufferBounds> Uniform<T> {
	pub fn new(var_name: impl Into<String>, data: T) -> Self {
		Self {
			var_name: var_name.into(),
			data,
		}
	}
}

impl<T: DataBufferBounds> ShaderBufferDescriptor for Uniform<T> {
	fn label(&self, label_type: &str) -> String {
		format!("{} <{}> {}", self.var_name, <T as ShaderType>::type_name(), label_type)
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

	fn create_bind_group_layout(&self, gpu: &Gpu, visibility: ShaderStages) -> BindGroupLayout {
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
			label: Some(&self.label("Bind Group Layout")),
		})
	}

	fn create_bind_group(&self, gpu: &Gpu, binding_resource: BindingResource, layout: &BindGroupLayout) -> BindGroup {
		gpu.device.create_bind_group(&BindGroupDescriptor {
			layout,
			entries: &[BindGroupEntry {
				binding: 0,
				resource: binding_resource,
			}],
			label: Some(&self.label("Bind Group")),
		})
	}
}

impl<T: DataBufferBounds> DataBufferDescriptor for Uniform<T> {
	fn create_buffer(&self, gpu: &Gpu) -> Buffer {
		let buffer = gpu.device.create_buffer(&BufferDescriptor {
			label: Some(&self.label("Buffer")),
			size: self.data.get_size(),
			usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		upload_bytes_to_buffer(gpu, &buffer, &self.data.get_bytes(), 0);
		buffer
	}
}
