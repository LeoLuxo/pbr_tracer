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

pub struct Storage<T: DataBufferBounds, const READ_ONLY: bool = false> {
	pub var_name: String,
	pub data: StorageData<T>,
}

pub enum StorageData<T> {
	Size(u64),
	Data(T),
}

impl<T: DataBufferBounds> Storage<T> {
	pub fn with_size(var_name: impl Into<String>, size: u64) -> Self {
		Self {
			var_name: var_name.into(),
			data: StorageData::Size(size),
		}
	}

	pub fn with_data(var_name: impl Into<String>, data: T) -> Self {
		Self {
			var_name: var_name.into(),
			data: StorageData::Data(data),
		}
	}
}

impl<T: DataBufferBounds, const READ_ONLY: bool> ShaderBufferDescriptor for Storage<T, READ_ONLY> {
	fn label(&self, label_type: &str) -> String {
		format!("{} <{}> {}", self.var_name, <T as ShaderType>::type_name(), label_type)
	}

	fn binding_source_code(&self, group: u32, binding_offset: u32) -> String {
		format!(
			"@group({}) @binding({}) var<storage, {}> {}: {};",
			group,
			binding_offset,
			if READ_ONLY { "read" } else { "read_write" },
			self.var_name,
			<T as ShaderType>::type_name()
		)
	}

	fn other_source_code(&self) -> Option<String> {
		<T as ShaderType>::struct_definition()
	}

	fn create_bind_group_layout(&self, gpu: &Gpu, visibility: ShaderStages) -> BindGroupLayout {
		gpu.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
			entries: &[BindGroupLayoutEntry {
				binding: 0,
				visibility,
				ty: BindingType::Buffer {
					ty: BufferBindingType::Storage { read_only: READ_ONLY },
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

impl<T: DataBufferBounds> DataBufferDescriptor for Storage<T> {
	fn create_buffer(&self, gpu: &Gpu) -> Buffer {
		let buffer = gpu.device.create_buffer(&BufferDescriptor {
			label: Some(&self.label("Buffer")),
			size: match &self.data {
				StorageData::Size(size) => *size,
				StorageData::Data(data) => data.get_size(),
			},
			usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		if let StorageData::Data(data) = &self.data {
			upload_bytes_to_buffer(gpu, &buffer, &data.get_bytes(), 0);
		}

		buffer
	}
}
