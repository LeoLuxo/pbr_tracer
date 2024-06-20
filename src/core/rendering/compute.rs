use bevy_ecs::{
	schedule::IntoSystemConfigs,
	system::{Res, ResMut},
};
use brainrot::{
	bevy::{self, App, Plugin},
	Shader, ShaderBuilder, TextureAsset,
};
use wgpu::{
	BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
	CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, FilterMode,
	ShaderStages, StorageTextureAccess, TextureFormat, TextureViewDimension,
};

use crate::{
	core::{gameloop::Render, gpu::Gpu, render_target::RenderTarget},
	SHADER_MAP,
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct ComputeRendererPlugin<P>
where
	P: Renderer,
{
	pub asd: P,
}

impl<P> Plugin for ComputeRendererPlugin<P>
where
	P: Renderer + 'static,
{
	fn build(&self, app: &mut App) {
		let gpu = app.world.resource::<Gpu>();

		let compute_renderer = ComputeRenderer::new::<crate::CurrentRenderer>(gpu);

		app.world.insert_resource(compute_renderer);

		app.add_systems(Render, (render).in_set(ComputeRenderPass).chain());
	}
}

#[derive(bevy::SystemSet, Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ComputeRenderPass;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub trait RenderFragment: Sync + Send {
	fn shader() -> impl Into<Shader>;
}

/// Shader API:
/// render_pixel(coords: vec3f) -> vec4f
pub trait Renderer: RenderFragment {}

#[derive(bevy::Resource)]
pub struct ComputeRenderer {
	pipeline: ComputePipeline,
	pub output_texture: TextureAsset,
}

impl ComputeRenderer {
	pub fn new<R: Renderer>(gpu: &Gpu) -> Self {
		// Statically include the shader in the executable
		// let shader = gpu
		// 	.device
		// 	.create_shader_module(include_wgsl!(src!("shader/compute.wgsl")));
		let shader = ShaderBuilder::new()
			.include(R::shader())
			.include_path("compute.wgsl")
			.build(&SHADER_MAP, &gpu.device)
			.expect("Couldn't build shader");

		// The output texture that the compute will write to
		let output_texture = TextureAsset::create_storage_sampler_texture(
			&gpu.device,
			(480, 270).into(),
			FilterMode::Linear,
			TextureFormat::Rgba32Float,
			Some("Output texture"),
		);

		let bind_group_layout = gpu.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
			label: Some("Compute Bind Group Layout"),
			entries: &[BindGroupLayoutEntry {
				binding: 0,
				visibility: ShaderStages::COMPUTE,
				ty: BindingType::StorageTexture {
					access: StorageTextureAccess::ReadWrite,
					format: output_texture.texture.format(),
					view_dimension: TextureViewDimension::D2,
				},
				count: None,
			}],
		});

		let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Compute Pipeline Layout"),
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[],
		});

		let pipeline = gpu.device.create_compute_pipeline(&ComputePipelineDescriptor {
			label: Some("Compute pipeline"),
			layout: Some(&pipeline_layout),
			module: &shader,
			entry_point: "main",
		});

		Self {
			pipeline,
			output_texture,
		}
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

fn render(compute_renderer: Res<ComputeRenderer>, mut render_target: ResMut<RenderTarget<'static>>, gpu: Res<Gpu>) {
	let mut encoder = gpu.device.create_command_encoder(&CommandEncoderDescriptor {
		label: Some("ComputeRenderer Command Encoder"),
	});

	let compute_bind_group = gpu.device.create_bind_group(&BindGroupDescriptor {
		label: Some("ComputeRenderer Bind Group"),
		layout: &compute_renderer.pipeline.get_bind_group_layout(0),
		entries: &[BindGroupEntry {
			binding: 0,
			resource: wgpu::BindingResource::TextureView(&compute_renderer.output_texture.view),
		}],
	});

	let out_width = compute_renderer.output_texture.texture.width();
	let out_height = compute_renderer.output_texture.texture.height();

	{
		let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
			label: Some("ComputeRenderer Compute Pass"),
			timestamp_writes: None,
		});

		compute_pass.set_pipeline(&compute_renderer.pipeline);

		compute_pass.set_bind_group(0, &compute_bind_group, &[]);

		compute_pass.dispatch_workgroups(out_width, out_height, 1);
	}

	render_target.command_queue.push(encoder.finish());
}
