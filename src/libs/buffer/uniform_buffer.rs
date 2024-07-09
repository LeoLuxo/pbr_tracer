use std::sync::Arc;

use brainrot::bevy::{self};
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

pub enum UniformBufferDescriptor<T, S>
where
	T: BufferUploadable,
	S: Into<String> + Clone,
{
	New { var_name: S, size: u64 },
	FromData { var_name: S, data: T },
	FromBuffer { var_name: S, buffer: Sarc<Buffer> },
}

impl<T, S> ShaderBufferDescriptor for UniformBufferDescriptor<T, S>
where
	T: BufferUploadable,
	S: Into<String> + Clone,
{
	fn as_resource(&self, gpu: &Gpu) -> Sarc<dyn ShaderBufferResource> {
		let resource = match self {
			UniformBufferDescriptor::New { var_name, size } => {
				UniformBuffer::new_from_size::<T>(gpu, *size, var_name.to_owned().into())
			}
			UniformBufferDescriptor::FromData { var_name, data } => {
				UniformBuffer::new_from_data::<T>(gpu, data, var_name.to_owned().into())
			}
			UniformBufferDescriptor::FromBuffer { var_name, buffer } => {
				UniformBuffer::new::<T>(buffer.clone(), var_name.to_owned().into())
			}
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
pub struct UniformBuffer {
	pub buffer: Sarc<Buffer>,
	pub var_name: String,
	type_name: String,
	struct_definition: Option<String>,
}

impl UniformBuffer {
	pub fn new_from_size<T: BufferUploadable>(gpu: &Gpu, size: u64, var_name: String) -> Self {
		Self::new::<T>(
			Sarc::new(Self::raw_buffer_from_size(
				gpu,
				size,
				Some(&format!("UniformBuffer<{}> '{}'", T::type_name(), var_name)),
			)),
			var_name,
		)
	}

	pub fn new_from_data<T: BufferUploadable>(gpu: &Gpu, data: &T, var_name: String) -> Self {
		Self::new::<T>(
			Sarc::new(Self::raw_buffer_from_data::<T>(
				gpu,
				data,
				Some(&format!("UniformBuffer<{}> '{}'", T::type_name(), var_name)),
			)),
			var_name,
		)
	}

	pub fn new<T: BufferUploadable>(buffer: Sarc<Buffer>, var_name: String) -> Self {
		UniformBuffer {
			buffer,
			var_name,
			type_name: T::type_name(),
			struct_definition: T::struct_definition(),
		}
	}

	pub fn raw_buffer_from_type<T: BufferUploadable>(gpu: &Gpu, label: Option<&str>) -> Buffer {
		Self::raw_buffer_from_size(gpu, T::get_size(), label)
	}

	pub fn raw_buffer_from_size(gpu: &Gpu, size: u64, label: Option<&str>) -> Buffer {
		gpu.device.create_buffer(&BufferDescriptor {
			label: label.or(Some(&format!("UniformBuffer<size: {}>", size))),
			size,
			usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
			mapped_at_creation: false,
		})
	}

	pub fn raw_buffer_from_data<T: BufferUploadable>(gpu: &Gpu, data: &T, label: Option<&str>) -> Buffer {
		gpu.device.create_buffer_init(&BufferInitDescriptor {
			label: label.or(Some(&format!("UniformBuffer<{}>", T::type_name()))),
			contents: &data.get_bytes(),
			usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
		})
	}
}

impl ShaderBufferResource for UniformBuffer {
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
