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

pub struct StorageBuffer<T: DataBufferUploadable + ShaderType> {
	pub var_name: String,
	pub read_only: bool,
	pub backing: StorageBufferBacking<T>,
}

pub enum StorageBufferBacking<T: DataBufferUploadable + ShaderType> {
	NewSized(u64),
	FromData(T),
	FromBuffer(Sarc<Buffer>),
}

impl<T: DataBufferUploadable + ShaderType> StorageBuffer<T> {
	pub fn new(var_name: impl Into<String>, size: u64, read_only: bool) -> Self {
		Self {
			var_name: var_name.into(),
			read_only,
			backing: StorageBufferBacking::NewSized(size),
		}
	}

	pub fn from_data(var_name: impl Into<String>, data: T, read_only: bool) -> Self {
		Self {
			var_name: var_name.into(),
			read_only,
			backing: StorageBufferBacking::FromData(data),
		}
	}

	pub fn from_buffer(var_name: impl Into<String>, buffer: Sarc<Buffer>, read_only: bool) -> Self {
		Self {
			var_name: var_name.into(),
			read_only,
			backing: StorageBufferBacking::FromBuffer(buffer),
		}
	}
}

impl<T: DataBufferUploadable + ShaderType> ShaderBufferDescriptor for StorageBuffer<T> {
	fn label(&self, label_type: &str) -> String {
		format!(
			"StorageBuffer<{}> \"{}\" {}",
			<T as ShaderType>::type_name(),
			self.var_name,
			label_type
		)
	}

	fn binding_source_code(&self, group: u32, binding_offset: u32) -> String {
		format!(
			"@group({}) @binding({}) var<storage, {}> {}: {};",
			group,
			binding_offset,
			if self.read_only { "read" } else { "read_write" },
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
					ty: BufferBindingType::Storage {
						read_only: self.read_only,
					},
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			}],
		})
	}
}

impl<T: DataBufferUploadable + ShaderType> DataBufferDescriptor for StorageBuffer<T> {
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
			StorageBufferBacking::NewSized(size) => {
				let buffer = gpu.device.create_buffer(&BufferDescriptor {
					label: Some(&self.label("Buffer")),
					size: *size,
					usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
					mapped_at_creation: false,
				});

				Sarc::new(buffer)
			}

			StorageBufferBacking::FromData(data) => {
				let buffer = gpu.device.create_buffer_init(&BufferInitDescriptor {
					label: Some(&self.label("Buffer")),
					contents: &data.get_bytes(),
					usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
				});

				Sarc::new(buffer)
			}

			StorageBufferBacking::FromBuffer(buffer) => buffer.clone(),
		}
	}
}
