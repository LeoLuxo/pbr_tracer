use std::sync::Arc;

use brainrot::bevy;
use wgpu::{
	util::{BufferInitDescriptor, DeviceExt},
	BindingResource, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, Features,
};

use super::{BufferUploadable, PartialLayoutEntry, ShaderBufferDescriptor, ShaderBufferResource};
use crate::{core::gpu::Gpu, libs::smart_arc::Sarc};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub enum StorageBufferDescriptor<T, S>
where
	T: BufferUploadable,
	S: Into<String> + Clone,
{
	New {
		var_name: S,
		read_only: bool,
		size: u64,
	},
	FromData {
		var_name: S,
		read_only: bool,
		data: T,
	},
	FromBuffer {
		var_name: S,
		read_only: bool,
		buffer: Sarc<Buffer>,
	},
}

impl<T, S> ShaderBufferDescriptor for StorageBufferDescriptor<T, S>
where
	T: BufferUploadable,
	S: Into<String> + Clone,
{
	fn as_resource(&self, gpu: &Gpu) -> Sarc<dyn ShaderBufferResource> {
		let resource = match self {
			StorageBufferDescriptor::New {
				var_name,
				read_only,
				size,
			} => StorageBuffer::new_from_size::<T>(gpu, *size, var_name.to_owned().into(), *read_only),
			StorageBufferDescriptor::FromData {
				var_name,
				read_only,
				data,
			} => StorageBuffer::new_from_data::<T>(gpu, data, var_name.to_owned().into(), *read_only),
			StorageBufferDescriptor::FromBuffer {
				var_name,
				read_only,
				buffer,
			} => StorageBuffer::new::<T>(buffer.clone(), var_name.to_owned().into(), *read_only),
		};

		Sarc(Arc::new(resource) as Arc<dyn ShaderBufferResource>)
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(bevy::Component)]
pub struct StorageBuffer {
	pub buffer: Sarc<Buffer>,
	pub var_name: String,
	pub read_only: bool,
	type_name: String,
	struct_definition: Option<String>,
}

impl StorageBuffer {
	pub fn new_from_size<T: BufferUploadable>(gpu: &Gpu, size: u64, var_name: String, read_only: bool) -> Self {
		Self::new::<T>(
			Sarc::new(Self::raw_buffer_from_size(
				gpu,
				size,
				Some(&format!("StorageBuffer<{}> '{}'", T::type_name(), var_name)),
			)),
			var_name,
			read_only,
		)
	}

	pub fn new_from_data<T: BufferUploadable>(gpu: &Gpu, data: &T, var_name: String, read_only: bool) -> Self {
		Self::new::<T>(
			Sarc::new(Self::raw_buffer_from_data::<T>(
				gpu,
				data,
				Some(&format!("StorageBuffer<{}> '{}'", T::type_name(), var_name)),
			)),
			var_name,
			read_only,
		)
	}

	pub fn new<T: BufferUploadable>(buffer: Sarc<Buffer>, var_name: String, read_only: bool) -> Self {
		StorageBuffer {
			buffer,
			var_name,
			read_only,
			type_name: T::type_name(),
			struct_definition: T::struct_definition(),
		}
	}

	pub fn raw_buffer_from_type<T: BufferUploadable>(gpu: &Gpu, label: Option<&str>) -> Buffer {
		Self::raw_buffer_from_size(gpu, T::get_size(), label)
	}

	pub fn raw_buffer_from_size(gpu: &Gpu, size: u64, label: Option<&str>) -> Buffer {
		gpu.device.create_buffer(&BufferDescriptor {
			label: label.or(Some(&format!("StorageBuffer<size: {}>", size))),
			size,
			usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
			mapped_at_creation: false,
		})
	}

	pub fn raw_buffer_from_data<T: BufferUploadable>(gpu: &Gpu, data: &T, label: Option<&str>) -> Buffer {
		gpu.device.create_buffer_init(&BufferInitDescriptor {
			label: label.or(Some(&format!("StorageBuffer<{}>", T::type_name()))),
			contents: &data.get_bytes(),
			usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
		})
	}
}

impl ShaderBufferResource for StorageBuffer {
	fn binding_source_code(&self, group: u32, binding: u32) -> Vec<String> {
		vec![format!(
			"@group({}) @binding({}) var<storage, {}> {}: {};",
			group,
			binding,
			if self.read_only { "read" } else { "read_write" },
			self.var_name,
			self.type_name
		)]
	}

	fn other_source_code(&self) -> Option<&str> {
		self.struct_definition.as_deref()
	}

	fn layouts(&self, _features: Features) -> Vec<PartialLayoutEntry> {
		vec![PartialLayoutEntry {
			ty: BindingType::Buffer {
				ty: BufferBindingType::Storage {
					read_only: self.read_only,
				},
				has_dynamic_offset: false,
				min_binding_size: None,
			},
			count: None,
		}]
	}

	fn binding_resources(&self) -> Vec<BindingResource> {
		vec![self.buffer.as_entire_binding()]
	}
}
