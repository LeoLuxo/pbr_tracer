pub mod uniform;

use std::{collections::HashMap, mem};

use bevy_ecs::system::{Query, Res};
use brainrot::{
	bevy::{self, App},
	vek,
};
use wgpu::{BindGroup, BindGroupLayout, BindingResource, Buffer, BufferAddress, ComputePass, RenderPass, ShaderStages};

use super::smart_arc::SmartArc;
use crate::core::{gameloop::PreRender, gpu::Gpu};

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

pub trait DataBufferBounds: DataBufferUploadable + ShaderType {}
impl<T: DataBufferUploadable + ShaderType> DataBufferBounds for T {}

pub trait ShaderBuffer {
	fn bind_group_layout(&self) -> &BindGroupLayout;
	fn bind_group(&self) -> &BindGroup;
	fn upload_bytes(&self, gpu: &Gpu, bytes: &[u8], offset: BufferAddress);
}

pub trait ShaderBufferDescriptor {
	fn label(&self, label_type: &str) -> String;
	fn binding_source_code(&self, group: u32, binding_offset: u32) -> String;
	fn create_bind_group_layout(&self, gpu: &Gpu, visibility: ShaderStages) -> BindGroupLayout;
	fn create_bind_group(&self, gpu: &Gpu, binding_resource: BindingResource, layout: &BindGroupLayout) -> BindGroup;
}

pub trait DataBufferDescriptor: ShaderBufferDescriptor {
	fn create_buffer(&self, gpu: &Gpu) -> Buffer;
}

pub trait TextureBufferDescriptor: ShaderBufferDescriptor {
	// fn create_texture(&self, gpu: &Gpu) -> Buffer;
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(bevy::Component, Debug)]
pub struct DataBuffer {
	pub buffer: Buffer,
	pub bind_group_layout: BindGroupLayout,
	pub bind_group: BindGroup,
}

impl DataBuffer {
	pub fn new(gpu: &Gpu, visibility: ShaderStages, shader_buffer: &dyn DataBufferDescriptor) -> Self {
		let buffer = shader_buffer.create_buffer(gpu);
		let bind_group_layout = shader_buffer.create_bind_group_layout(gpu, visibility);
		let bind_group = shader_buffer.create_bind_group(gpu, buffer.as_entire_binding(), &bind_group_layout);

		DataBuffer {
			buffer,
			bind_group_layout,
			bind_group,
		}
	}
}

impl ShaderBuffer for DataBuffer {
	fn bind_group_layout(&self) -> &BindGroupLayout {
		&self.bind_group_layout
	}

	fn bind_group(&self) -> &BindGroup {
		&self.bind_group
	}

	fn upload_bytes(&self, gpu: &Gpu, bytes: &[u8], offset: BufferAddress) {
		upload_bytes_to_buffer(gpu, &self.buffer, bytes, offset)
	}
}

pub fn upload_bytes_to_buffer(gpu: &Gpu, buffer: &Buffer, bytes: &[u8], offset: BufferAddress) {
	gpu.queue.write_buffer(buffer, offset, bytes)
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(Debug, Default)]
pub struct BufferMapping(pub HashMap<u32, SmartArc<dyn ShaderBuffer + Sync + Send>>);

impl<'a> BufferMapping {
	pub fn apply_to_render_pass(&'a self, render_pass: &mut RenderPass<'a>) {
		for (index, shader_buffer) in &self.0 {
			render_pass.set_bind_group(*index, shader_buffer.bind_group(), &[]);
		}
	}

	pub fn apply_to_compute_pass(&'a self, compute_pass: &mut ComputePass<'a>) {
		for (index, shader_buffer) in &self.0 {
			compute_pass.set_bind_group(*index, shader_buffer.bind_group(), &[]);
		}
	}

	pub fn layouts(&self) -> impl Iterator<Item = &BindGroupLayout> {
		self.0.values().map(|buffer| buffer.bind_group_layout())
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub fn register_uniform_auto_update<T>(app: &mut App)
where
	T: DataBufferUploadable + bevy::Component + Send + Sync,
{
	app.add_systems(PreRender, upload_buffers_system::<T>);
}

fn upload_buffers_system<T>(gpu: Res<Gpu>, q: Query<(&T, &DataBuffer)>)
where
	T: DataBufferUploadable + bevy::Component + Send + Sync,
{
	for (data, buffer) in q.iter() {
		buffer.upload_bytes(&gpu, &data.get_bytes(), 0);
	}
}
