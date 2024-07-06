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

pub enum UniformBuffer<T, S>
where
	T: DataBufferUploadable + ShaderType,
	S: Into<String> + Clone,
{
	New { var_name: S, size: u64 },
	FromData { var_name: S, data: T },
	FromBuffer { var_name: S, buffer: Sarc<Buffer> },
}

impl<T, S> ShaderBufferDescriptor for UniformBuffer<T, S>
where
	T: DataBufferUploadable + ShaderType,
	S: Into<String> + Clone,
{
	fn as_resource(&self, gpu: &Gpu) -> Sarc<dyn ShaderBufferResource> {
		let type_name = <T as ShaderType>::type_name();
		let struct_definition = <T as ShaderType>::struct_definition();

		let resource = match self {
			UniformBuffer::New { var_name, size } => {
				let var_name = var_name.to_owned().into();
				let buffer = Sarc::new(gpu.device.create_buffer(&BufferDescriptor {
					label: Some(&format!("UniformBuffer<{}> '{}'", type_name, &var_name)),
					size: *size,
					usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
					mapped_at_creation: false,
				}));

				UniformBufferResource {
					buffer,
					var_name,
					type_name,
					struct_definition,
				}
			}

			UniformBuffer::FromData { var_name, data } => {
				let var_name = var_name.to_owned().into();
				let buffer = Sarc::new(gpu.device.create_buffer_init(&BufferInitDescriptor {
					label: Some(&format!("UniformBuffer<{}> '{}'", type_name, var_name)),
					contents: &data.get_bytes(),
					usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
				}));

				UniformBufferResource {
					buffer,
					var_name,
					type_name,
					struct_definition,
				}
			}

			UniformBuffer::FromBuffer { var_name, buffer } => UniformBufferResource {
				buffer: buffer.clone(),
				var_name: var_name.to_owned().into(),
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

pub struct UniformBufferResource {
	pub buffer: Sarc<Buffer>,
	pub var_name: String,
	pub type_name: String,
	pub struct_definition: Option<String>,
}

impl UniformBufferResource {
	pub fn upload_bytes(&self, gpu: &Gpu, bytes: &[u8], offset: BufferAddress) {
		gpu.queue.write_buffer(&self.buffer, offset, bytes)
	}
}

impl ShaderBufferResource for UniformBufferResource {
	fn binding_source_code(&self, group: u32, binding: u32) -> Vec<String> {
		vec![format!(
			"@group({}) @binding({}) var<uniform> {}: {};",
			group, binding, self.var_name, self.type_name
		)]
	}

	fn other_source_code(&self) -> Option<&str> {
		self.struct_definition.as_deref()
	}

	fn layouts(&self, _features: Features) -> Vec<PartialLayoutEntry> {
		vec![PartialLayoutEntry {
			ty: BindingType::Buffer {
				ty: BufferBindingType::Uniform,
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
