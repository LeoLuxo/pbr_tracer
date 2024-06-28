use bevy_ecs::{
	schedule::IntoSystemConfigs,
	system::{Res, ResMut},
};
use brainrot::{
	bevy::{self, App, Plugin},
	ScreenSize, TextureAsset,
};
use velcro::vec;
use wgpu::{
	BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
	CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, FilterMode,
	ShaderStages, StorageTextureAccess, TextureFormat, TextureViewDimension,
};

use crate::{
	core::{gameloop::Render, gpu::Gpu, render_target::RenderTarget},
	libs::{
		shader::{CompiledShader, ShaderBuilder},
		shader_fragment::{Renderer, ShaderFragment},
	},
	ShaderAssets,
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct ComputeRendererPlugin<R: Renderer> {
	pub resolution: ScreenSize,
	pub renderer: R,
}

impl<R> Plugin for ComputeRendererPlugin<R>
where
	R: Renderer + 'static,
{
	fn build(&self, app: &mut App) {
		// TODO refactor the bufer stuff somehow
		// Maybe declarative?

		let gpu = app.world.resource::<Gpu>();

		let compute_renderer = ComputeRenderer::new(gpu, self.resolution, &self.renderer);

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

#[derive(bevy::Resource)]
pub struct ComputeRenderer {
	pipeline: ComputePipeline,
	pub output_texture: TextureAsset,
	pub shader: CompiledShader,
}

impl ComputeRenderer {
	pub fn new(gpu: &Gpu, resolution: ScreenSize, renderer: &dyn ShaderFragment) -> Self {
		// Dynamically create shader from the renderer
		let shader = ShaderBuilder::new()
			.include_path("compute.wgsl")
			.include(renderer.shader())
			.build(gpu, &ShaderAssets, ShaderStages::COMPUTE, 1)
			.expect("Couldn't build shader");

		// The output texture that the compute will write to
		let output_texture = TextureAsset::create_storage_sampler_texture(
			&gpu.device,
			resolution,
			FilterMode::Nearest,
			TextureFormat::Rgba32Float,
			Some("Output texture"),
		);

		let texture_bind_grounp_layout = gpu.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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

		let bind_group_layouts = &vec![&texture_bind_grounp_layout, ..shader.buffers.layouts()];

		let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Compute Pipeline Layout"),
			bind_group_layouts,
			push_constant_ranges: &[],
		});

		let pipeline = gpu.device.create_compute_pipeline(&ComputePipelineDescriptor {
			label: Some("Compute pipeline"),
			layout: Some(&pipeline_layout),
			module: &shader.shader_module,
			entry_point: "main",
		});

		Self {
			pipeline,
			output_texture,
			shader,
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
		compute_renderer.shader.buffers.apply_to_compute_pass(&mut compute_pass);

		compute_pass.dispatch_workgroups(out_width, out_height, 1);
	}

	render_target.command_queue.push(encoder.finish());
}
