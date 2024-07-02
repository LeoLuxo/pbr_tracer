pub mod storage_buffer;
pub mod texture_buffer;
pub mod texture_sampler_buffer;
pub mod uniform_buffer;

use std::{collections::HashMap, mem};

use bevy_ecs::system::{Query, Res};
use brainrot::{
	bevy::{self, App},
	vek::{self},
};
use wgpu::{BindGroup, BindGroupLayout, Buffer, BufferAddress, ComputePass, RenderPass, ShaderStages};

use super::{smart_arc::Sarc, texture::TextureAsset};
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

pub trait ShaderBuffer {
	fn label(&self) -> &str;
	fn bind_group_layout(&self) -> &BindGroupLayout;
	fn bind_group(&self) -> &BindGroup;
}

pub trait ShaderBufferDescriptor {
	fn label(&self, label_type: &str) -> String;
	fn binding_source_code(&self, group: u32, binding_offset: u32) -> String;
	fn other_source_code(&self) -> Option<String>;
	fn create_bind_group_layout(&self, gpu: &Gpu, visibility: ShaderStages) -> BindGroupLayout;
}

pub trait DataBufferDescriptor: ShaderBufferDescriptor {
	fn create_buffer(&self, gpu: &Gpu) -> Sarc<Buffer>;
	fn create_bind_group(&self, gpu: &Gpu, layout: &BindGroupLayout, buffer: &Buffer) -> BindGroup;
}

pub trait TextureBufferDescriptor: ShaderBufferDescriptor {
	fn create_texture(&self, gpu: &Gpu) -> Sarc<TextureAsset>;
	fn create_bind_group(&self, gpu: &Gpu, layout: &BindGroupLayout, texture: &TextureAsset) -> BindGroup;
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(bevy::Component, Clone, Debug, PartialEq, Eq)]
pub struct GenericDataBuffer {
	pub label: String,
	pub buffer: Sarc<Buffer>,
	pub bind_group_layout: Sarc<BindGroupLayout>,
	pub bind_group: Sarc<BindGroup>,
}

impl GenericDataBuffer {
	pub fn new(gpu: &Gpu, visibility: ShaderStages, shader_buffer: &dyn DataBufferDescriptor) -> Self {
		let label = shader_buffer.label("(as GenericDataBuffer)");
		let buffer = shader_buffer.create_buffer(gpu);
		let bind_group_layout = Sarc::new(shader_buffer.create_bind_group_layout(gpu, visibility));
		let bind_group = Sarc::new(shader_buffer.create_bind_group(gpu, &bind_group_layout, &buffer));

		GenericDataBuffer {
			label,
			buffer,
			bind_group_layout,
			bind_group,
		}
	}

	fn upload_bytes(&self, gpu: &Gpu, bytes: &[u8], offset: BufferAddress) {
		gpu.queue.write_buffer(&self.buffer, offset, bytes)
	}
}

impl ShaderBuffer for GenericDataBuffer {
	fn label(&self) -> &str {
		&self.label
	}

	fn bind_group_layout(&self) -> &BindGroupLayout {
		&self.bind_group_layout
	}

	fn bind_group(&self) -> &BindGroup {
		&self.bind_group
	}
}

#[derive(bevy::Component, Debug)]
pub struct GenericTextureBuffer {
	pub label: String,
	pub texture: Sarc<TextureAsset>,
	pub bind_group_layout: BindGroupLayout,
	pub bind_group: BindGroup,
}

impl GenericTextureBuffer {
	pub fn new(gpu: &Gpu, visibility: ShaderStages, shader_buffer: &dyn TextureBufferDescriptor) -> Self {
		let label = shader_buffer.label("(as GenericTextureBuffer)");
		let texture = shader_buffer.create_texture(gpu);
		let bind_group_layout = shader_buffer.create_bind_group_layout(gpu, visibility);
		let bind_group = shader_buffer.create_bind_group(gpu, &bind_group_layout, &texture);

		GenericTextureBuffer {
			label,
			texture,
			bind_group_layout,
			bind_group,
		}
	}
}

impl ShaderBuffer for GenericTextureBuffer {
	fn label(&self) -> &str {
		&self.label
	}

	fn bind_group_layout(&self) -> &BindGroupLayout {
		&self.bind_group_layout
	}

	fn bind_group(&self) -> &BindGroup {
		&self.bind_group
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(Default)]
pub struct BufferMapping(pub HashMap<u32, Sarc<dyn ShaderBuffer + Sync + Send>>);

impl BufferMapping {
	pub fn layouts(&self) -> Vec<&BindGroupLayout> {
		let mut entries = self.0.iter().collect::<Vec<_>>();
		entries.sort_by_key(|(i, _)| *i);
		entries.iter().map(|(_, v)| v.bind_group_layout()).collect()
	}
}

impl std::fmt::Debug for BufferMapping {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut tuple = f.debug_tuple("BufferMapping");

		for (index, shader_buffer) in &self.0 {
			tuple.field(&(index, shader_buffer.label()));
		}

		tuple.finish()
	}
}

pub trait BufferMappingApplicable<'a> {
	fn apply_buffer_mapping(&mut self, buffer_mapping: &'a BufferMapping);
}

impl<'a> BufferMappingApplicable<'a> for ComputePass<'a> {
	fn apply_buffer_mapping(&mut self, buffer_mapping: &'a BufferMapping) {
		for (index, shader_buffer) in &buffer_mapping.0 {
			self.set_bind_group(*index, shader_buffer.bind_group(), &[]);
		}
	}
}

impl<'a> BufferMappingApplicable<'a> for RenderPass<'a> {
	fn apply_buffer_mapping(&mut self, buffer_mapping: &'a BufferMapping) {
		for (index, shader_buffer) in &buffer_mapping.0 {
			self.set_bind_group(*index, shader_buffer.bind_group(), &[]);
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
	T: DataBufferUploadable + bevy::Component + Send + Sync,
{
	app.add_systems(PreRender, upload_buffers_system::<T>);
}

fn upload_buffers_system<T>(gpu: Res<Gpu>, q: Query<(&T, &GenericDataBuffer)>)
where
	T: DataBufferUploadable + bevy::Component + Send + Sync,
{
	for (data, buffer) in q.iter() {
		buffer.upload_bytes(&gpu, &data.get_bytes(), 0);
	}
}
