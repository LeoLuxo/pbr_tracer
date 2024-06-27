use std::{collections::HashMap, marker::PhantomData, mem};

use bevy_ecs::system::{Query, Res};
use brainrot::{
	bevy::{self, App},
	vek,
};
use wgpu::{
	BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
	BindingType, BufferAddress, BufferBindingType, BufferDescriptor, BufferUsages, ComputePass, RenderPass,
	ShaderStages,
};

use super::{gameloop::PreRender, gpu::Gpu};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub trait ShaderStruct {
	fn get_source_code() -> String;
}

pub trait WgslType {
	fn name() -> String;
}

#[rustfmt::skip] impl                              WgslType for bool            {fn name() -> String {"bool".to_string()}}
#[rustfmt::skip] impl                              WgslType for i32             {fn name() -> String {"i32".to_string()}}
#[rustfmt::skip] impl                              WgslType for u32             {fn name() -> String {"u32".to_string()}}
#[rustfmt::skip] impl                              WgslType for f32             {fn name() -> String {"f32".to_string()}}
#[rustfmt::skip] impl<T: WgslType>                 WgslType for vek::Vec2<T>    {fn name() -> String {format!("vec2<{}>", T::name())}}
#[rustfmt::skip] impl<T: WgslType>                 WgslType for vek::Vec3<T>    {fn name() -> String {format!("vec3<{}>", T::name())}}
#[rustfmt::skip] impl<T: WgslType>                 WgslType for vek::Vec4<T>    {fn name() -> String {format!("vec4<{}>", T::name())}}
#[rustfmt::skip] impl<T: WgslType>                 WgslType for vek::Extent2<T> {fn name() -> String {format!("vec2<{}>", T::name())}}
#[rustfmt::skip] impl<T: WgslType>                 WgslType for vek::Extent3<T> {fn name() -> String {format!("vec3<{}>", T::name())}}
#[rustfmt::skip] impl<T: WgslType>                 WgslType for vek::Rgb<T>     {fn name() -> String {format!("vec3<{}>", T::name())}}
#[rustfmt::skip] impl<T: WgslType>                 WgslType for vek::Rgba<T>    {fn name() -> String {format!("vec4<{}>", T::name())}}
#[rustfmt::skip] impl<T: WgslType>                 WgslType for vek::Mat2<T>    {fn name() -> String {format!("mat2x2<{}>", T::name())}}
#[rustfmt::skip] impl<T: WgslType>                 WgslType for vek::Mat3<T>    {fn name() -> String {format!("mat3x3<{}>", T::name())}}
#[rustfmt::skip] impl<T: WgslType>                 WgslType for vek::Mat4<T>    {fn name() -> String {format!("mat4x4<{}>", T::name())}}
#[rustfmt::skip] impl<E: WgslType>                 WgslType for [E]             {fn name() -> String {format!("array<{}>", E::name())}}
#[rustfmt::skip] impl<E: WgslType, const N: usize> WgslType for [E; N]          {fn name() -> String {format!("array<{},{}>", E::name(), N)}}

// Incompatible:
// impl WgslType for f16 {fn name() -> String {format!("f16")}}

pub trait BufferUploadable: std::fmt::Debug {
	fn get_size(&self) -> u64;
	fn get_data(&self) -> Vec<u8>;
}

macro_rules! impl_buffer_uploadable {
	() => {
		fn get_size(&self) -> u64 {
			mem::size_of::<Self>() as u64
		}

		fn get_data(&self) -> Vec<u8> {
			bytemuck::bytes_of(self).to_owned()
		}
	};
}

trait BufferUploadableSubType: WgslType + bytemuck::Pod + std::fmt::Debug {}
impl<T: WgslType + bytemuck::Pod + std::fmt::Debug> BufferUploadableSubType for T {}

#[rustfmt::skip] impl                                             BufferUploadable for bool            {impl_buffer_uploadable!();}
#[rustfmt::skip] impl                                             BufferUploadable for i32             {impl_buffer_uploadable!();}
#[rustfmt::skip] impl                                             BufferUploadable for u32             {impl_buffer_uploadable!();}
#[rustfmt::skip] impl                                             BufferUploadable for f32             {impl_buffer_uploadable!();}
#[rustfmt::skip] impl<T: BufferUploadableSubType>                 BufferUploadable for vek::Vec2<T>    {impl_buffer_uploadable!();}
#[rustfmt::skip] impl<T: BufferUploadableSubType>                 BufferUploadable for vek::Vec3<T>    {impl_buffer_uploadable!();}
#[rustfmt::skip] impl<T: BufferUploadableSubType>                 BufferUploadable for vek::Vec4<T>    {impl_buffer_uploadable!();}
#[rustfmt::skip] impl<T: BufferUploadableSubType>                 BufferUploadable for vek::Extent2<T> {impl_buffer_uploadable!();}
#[rustfmt::skip] impl<T: BufferUploadableSubType>                 BufferUploadable for vek::Extent3<T> {impl_buffer_uploadable!();}
#[rustfmt::skip] impl<T: BufferUploadableSubType>                 BufferUploadable for vek::Rgb<T>     {impl_buffer_uploadable!();}
#[rustfmt::skip] impl<T: BufferUploadableSubType>                 BufferUploadable for vek::Rgba<T>    {impl_buffer_uploadable!();}
#[rustfmt::skip] impl<T: BufferUploadableSubType>                 BufferUploadable for vek::Mat2<T>    {impl_buffer_uploadable!();}
#[rustfmt::skip] impl<T: BufferUploadableSubType>                 BufferUploadable for vek::Mat3<T>    {impl_buffer_uploadable!();}
#[rustfmt::skip] impl<T: BufferUploadableSubType>                 BufferUploadable for vek::Mat4<T>    {impl_buffer_uploadable!();}
#[rustfmt::skip] impl<E: BufferUploadableSubType, const N: usize> BufferUploadable for [E; N]          {impl_buffer_uploadable!();}

// Incompatible:
// impl<E: WgslType> BufferUploadable for [E] {impl_buffer_uploadable!();}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub trait BufferType {
	fn create_buffer(&self, gpu: &Gpu, size: u64) -> wgpu::Buffer;
	fn create_bind_group_layout(&self, gpu: &Gpu, visibility: ShaderStages) -> wgpu::BindGroupLayout;
	fn create_bind_group(&self, gpu: &Gpu, buffer: &wgpu::Buffer, layout: &BindGroupLayout) -> BindGroup;
	fn get_source_code(&self, group: u32, binding_offset: u32) -> String;
}

pub struct Uniform<'a, DataType>
where
	DataType: BufferUploadable + WgslType,
{
	pub var_name: &'a str,
	_data_type: PhantomData<DataType>,
}

impl<'a, DataType> Uniform<'a, DataType>
where
	DataType: BufferUploadable + WgslType,
{
	pub fn new(var_name: &'a str) -> Self {
		Self {
			var_name,
			_data_type: PhantomData,
		}
	}
}

impl<DataType> BufferType for Uniform<'_, DataType>
where
	DataType: BufferUploadable + WgslType,
{
	fn create_buffer(&self, gpu: &Gpu, size: u64) -> wgpu::Buffer {
		gpu.device.create_buffer(&BufferDescriptor {
			label: Some(&format!("{} <{}> Buffer", self.var_name, DataType::name())),
			size,
			usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
			mapped_at_creation: false,
		})
	}

	fn create_bind_group_layout(&self, gpu: &Gpu, visibility: ShaderStages) -> wgpu::BindGroupLayout {
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
			label: Some(&format!("{} <{}> Bingdgroup Layout", self.var_name, DataType::name())),
		})
	}

	fn create_bind_group(&self, gpu: &Gpu, buffer: &wgpu::Buffer, layout: &BindGroupLayout) -> BindGroup {
		gpu.device.create_bind_group(&BindGroupDescriptor {
			layout,
			entries: &[BindGroupEntry {
				binding: 0,
				resource: buffer.as_entire_binding(),
			}],
			label: Some(&format!("{} <{}> Bingdgroup", self.var_name, DataType::name())),
		})
	}

	fn get_source_code(&self, group: u32, binding_offset: u32) -> String {
		format!(
			"@group({}) @binding({}) var<uniform> {}: {};",
			group,
			binding_offset,
			self.var_name,
			DataType::name()
		)
	}
}

pub trait BufferDataType<B: BufferType>: BufferUploadable {}
impl<T: BufferUploadable + WgslType> BufferDataType<Uniform<'_, T>> for T {}

#[derive(bevy::Component, Debug)]
pub struct Buffer {
	pub buffer: wgpu::Buffer,
	pub bind_group_layout: wgpu::BindGroupLayout,
	pub bind_group: wgpu::BindGroup,
}

impl Buffer {
	pub fn new(gpu: &Gpu, visibility: ShaderStages, size: u64, buffer_type: &dyn BufferType) -> Self {
		// Create a uniform buffer for data in T
		// In wgpu, uniforms need to be explicitly created as buffers
		let buffer = buffer_type.create_buffer(gpu, size);
		let bind_group_layout = buffer_type.create_bind_group_layout(gpu, visibility);
		let bind_group = buffer_type.create_bind_group(gpu, &buffer, &bind_group_layout);

		Buffer {
			buffer,
			bind_group_layout,
			bind_group,
		}
	}

	pub fn upload_bytes(&self, gpu: &Gpu, bytes: &[u8], offset: BufferAddress) {
		upload_bytes_to_buffer(gpu, &self.buffer, bytes, offset)
	}
}

pub fn upload_bytes_to_buffer(gpu: &Gpu, raw_buffer: &wgpu::Buffer, bytes: &[u8], offset: BufferAddress) {
	gpu.queue.write_buffer(raw_buffer, offset, bytes)
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(Debug, Default)]
pub struct BindGroupMapping(pub HashMap<u32, BindGroup>);

impl<'a> BindGroupMapping {
	pub fn apply_to_render_pass(&'a self, render_pass: &mut RenderPass<'a>) {
		for (index, bind_group) in &self.0 {
			render_pass.set_bind_group(*index, bind_group, &[]);
		}
	}

	pub fn apply_to_compute_pass(&'a self, compute_pass: &mut ComputePass<'a>) {
		for (index, bind_group) in &self.0 {
			compute_pass.set_bind_group(*index, bind_group, &[]);
		}
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub fn register_uniform_auto_update<T>(app: &mut App)
where
	T: BufferUploadable + bevy::Component + Send + Sync,
{
	app.add_systems(PreRender, upload_buffers_system::<T>);
}

fn upload_buffers_system<T>(gpu: Res<Gpu>, q: Query<(&T, &Buffer)>)
where
	T: BufferUploadable + bevy::Component + Send + Sync,
{
	for (data, buffer) in q.iter() {
		buffer.upload_bytes(&gpu, &data.get_data(), 0);
	}
}
