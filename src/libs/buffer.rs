pub mod sampled_texture_buffer;
pub mod storage_buffer;
pub mod storage_texture_buffer;
pub mod uniform_buffer;

use std::{fmt::Debug, mem, num::NonZero};

use brainrot::vek::{self};
use wgpu::{
	BindGroup, BindGroupLayout, BindGroupLayoutEntry, BindingResource, BindingType, ComputePass, Features, RenderPass,
	ShaderStages,
};

use super::smart_arc::Sarc;
use crate::core::gpu::Gpu;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub trait ShaderType {
	fn type_name() -> String;
	fn struct_definition() -> Option<String> {
		None
	}
}

#[rustfmt::skip] impl                ShaderType for bool            {fn type_name() -> String {"bool".to_string()}}
#[rustfmt::skip] impl                ShaderType for i32             {fn type_name() -> String {"i32".to_string()}}
#[rustfmt::skip] impl                ShaderType for u32             {fn type_name() -> String {"u32".to_string()}}
#[rustfmt::skip] impl                ShaderType for f32             {fn type_name() -> String {"f32".to_string()}}
#[rustfmt::skip] impl<T: ShaderType> ShaderType for vek::Vec2<T>    {fn type_name() -> String {format!("vec2<{}>", T::type_name())}}
#[rustfmt::skip] impl<T: ShaderType> ShaderType for vek::Vec3<T>    {fn type_name() -> String {format!("vec3<{}>", T::type_name())}}
#[rustfmt::skip] impl<T: ShaderType> ShaderType for vek::Vec4<T>    {fn type_name() -> String {format!("vec4<{}>", T::type_name())}}
#[rustfmt::skip] impl<T: ShaderType> ShaderType for vek::Extent2<T> {fn type_name() -> String {format!("vec2<{}>", T::type_name())}}
#[rustfmt::skip] impl<T: ShaderType> ShaderType for vek::Extent3<T> {fn type_name() -> String {format!("vec3<{}>", T::type_name())}}
#[rustfmt::skip] impl<T: ShaderType> ShaderType for vek::Rgb<T>     {fn type_name() -> String {format!("vec3<{}>", T::type_name())}}
#[rustfmt::skip] impl<T: ShaderType> ShaderType for vek::Rgba<T>    {fn type_name() -> String {format!("vec4<{}>", T::type_name())}}
#[rustfmt::skip] impl<T: ShaderType> ShaderType for vek::Mat2<T>    {fn type_name() -> String {format!("mat2x2<{}>", T::type_name())}}
#[rustfmt::skip] impl<T: ShaderType> ShaderType for vek::Mat3<T>    {fn type_name() -> String {format!("mat3x3<{}>", T::type_name())}}
#[rustfmt::skip] impl<T: ShaderType> ShaderType for vek::Mat4<T>    {fn type_name() -> String {format!("mat4x4<{}>", T::type_name())}}

#[rustfmt::skip] impl<E: ShaderType>                 ShaderType for [E]    {fn type_name() -> String {format!("array<{}>", E::type_name())}}
#[rustfmt::skip] impl<E: ShaderType, const N: usize> ShaderType for [E; N] {fn type_name() -> String {format!("array<{},{}>", E::type_name(), N)}}

// Incompatible:
// impl WgslType for f16 {fn name() -> String {format!("f16")}}

pub trait DataBufferUploadable: std::fmt::Debug {
	fn get_size(&self) -> u64;
	fn get_bytes(&self) -> Vec<u8>;
	fn type_name(&self) -> String;
	fn struct_definition(&self) -> Option<String>;
}

// This blanket impl excludes [E]
impl<T: ShaderType + bytemuck::Pod + Sized + std::fmt::Debug> DataBufferUploadable for T {
	fn get_size(&self) -> u64 {
		mem::size_of::<Self>() as u64
	}

	fn get_bytes(&self) -> Vec<u8> {
		bytemuck::bytes_of(self).to_owned()
	}

	fn type_name(&self) -> String {
		<Self as ShaderType>::type_name()
	}

	fn struct_definition(&self) -> Option<String> {
		<Self as ShaderType>::struct_definition()
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct PartialLayoutEntry {
	pub ty: BindingType,
	pub count: Option<NonZero<u32>>,
}

impl PartialLayoutEntry {
	pub fn into_layout_entry(self, binding: u32, visibility: ShaderStages) -> BindGroupLayoutEntry {
		BindGroupLayoutEntry {
			binding,
			visibility,
			ty: self.ty,
			count: self.count,
		}
	}
}

pub trait ShaderBufferDescriptor {
	fn as_resource(&self, gpu: &Gpu) -> Sarc<dyn ShaderBufferResource>;
}

pub trait ShaderBufferResource {
	fn binding_source_code(&self, group: u32, binding: u32) -> Vec<String>;
	fn other_source_code(&self) -> Option<&str>;
	fn layouts(&self, features: Features) -> Vec<PartialLayoutEntry>;
	fn binding_resources(&self) -> Vec<BindingResource>;
}

// pub trait ShaderBuffer {
// 	fn label(&self) -> &str;
// 	fn bind_group_layout(&self) -> &BindGroupLayout;
// 	fn bind_group(&self) -> &BindGroup;
// }

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct ShaderBufferBindGroup {
	pub index: u32,
	pub bind_group_layout: BindGroupLayout,
	pub bind_group: BindGroup,
}

impl Debug for ShaderBufferBindGroup {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ShaderBufferBindGroup")
			.field("index", &self.index)
			.finish()
	}
}

pub trait BufferMappingApplicable<'a> {
	fn apply_buffer_mapping(&mut self, buffer_mapping: &'a ShaderBufferBindGroup);
}

impl<'a> BufferMappingApplicable<'a> for ComputePass<'a> {
	fn apply_buffer_mapping(&mut self, buffer_mapping: &'a ShaderBufferBindGroup) {
		self.set_bind_group(buffer_mapping.index, &buffer_mapping.bind_group, &[]);
	}
}

impl<'a> BufferMappingApplicable<'a> for RenderPass<'a> {
	fn apply_buffer_mapping(&mut self, buffer_mapping: &'a ShaderBufferBindGroup) {
		self.set_bind_group(buffer_mapping.index, &buffer_mapping.bind_group, &[]);
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

// pub fn register_uniform_auto_update<T>(app: &mut App)
// where
// 	T: DataBufferUploadable + bevy::Component + Send + Sync,
// {
// 	app.add_systems(PreRender, upload_buffers_system::<T>);
// }

// fn upload_buffers_system<T>(gpu: Res<Gpu>, q: Query<(&T, &GenericDataBuffer)>)
// where
// 	T: DataBufferUploadable + bevy::Component + Send + Sync,
// {
// 	for (data, buffer) in q.iter() {
// 		buffer.upload_bytes(&gpu, &data.get_bytes(), 0);
// 	}
// }
