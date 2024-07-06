use std::sync::Arc;

use wgpu::{
	util::{BufferInitDescriptor, DeviceExt},
	BindingResource, BindingType, Buffer, BufferAddress, BufferBindingType, BufferDescriptor, BufferUsages, Features,
};

use super::{DataBufferUploadable, PartialLayoutEntry, ShaderBufferDescriptor, ShaderBufferResource};
use crate::{
	core::gpu::Gpu,
	libs::{buffer::ShaderType, smart_arc::Sarc},
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub enum StorageBuffer<T, S>
where
	T: DataBufferUploadable + ShaderType,
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

impl<T, S> ShaderBufferDescriptor for StorageBuffer<T, S>
where
	T: DataBufferUploadable + ShaderType,
	S: Into<String> + Clone,
{
	fn as_resource(&self, gpu: &Gpu) -> Sarc<dyn ShaderBufferResource> {
		let type_name = <T as ShaderType>::type_name();
		let struct_definition = <T as ShaderType>::struct_definition();

		let resource = match self {
			StorageBuffer::New {
				var_name,
				read_only,
				size,
			} => {
				let var_name = var_name.to_owned().into();
				let buffer = Sarc::new(gpu.device.create_buffer(&BufferDescriptor {
					label: Some(&format!("StorageBuffer<{}> '{}'", type_name, var_name)),
					size: *size,
					usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
					mapped_at_creation: false,
				}));

				StorageBufferResource {
					buffer,
					var_name,
					read_only: *read_only,
					type_name,
					struct_definition,
				}
			}

			StorageBuffer::FromData {
				var_name,
				read_only,
				data,
			} => {
				let var_name = var_name.to_owned().into();
				let buffer = Sarc::new(gpu.device.create_buffer_init(&BufferInitDescriptor {
					label: Some(&format!("StorageBuffer<{}> '{}'", type_name, var_name)),
					contents: &data.get_bytes(),
					usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
				}));

				StorageBufferResource {
					buffer,
					var_name,
					read_only: *read_only,
					type_name,
					struct_definition,
				}
			}

			StorageBuffer::FromBuffer {
				var_name,
				read_only,
				buffer,
			} => StorageBufferResource {
				buffer: buffer.clone(),
				var_name: var_name.to_owned().into(),
				read_only: *read_only,
				type_name,
				struct_definition,
			},
		};

		Sarc(Arc::new(resource) as Arc<dyn ShaderBufferResource>)
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct StorageBufferResource {
	pub buffer: Sarc<Buffer>,
	pub var_name: String,
	pub read_only: bool,
	pub type_name: String,
	pub struct_definition: Option<String>,
}

impl StorageBufferResource {
	pub fn upload_bytes(&self, gpu: &Gpu, bytes: &[u8], offset: BufferAddress) {
		gpu.queue.write_buffer(&self.buffer, offset, bytes)
	}
}

impl ShaderBufferResource for StorageBufferResource {
	fn binding_source_code(&self, group: u32, binding_offset: u32) -> Vec<String> {
		vec![format!(
			"@group({}) @binding({}) var<storage, {}> {}: {};",
			group,
			binding_offset,
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
