use bevy_ecs::{
	schedule::IntoSystemConfigs,
	system::{Res, ResMut},
};
use brainrot::{
	bevy::{self, App, Plugin},
	ScreenSize,
};
use wgpu::{
	CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, FilterMode,
	SamplerBorderColor, ShaderStages, StorageTextureAccess, TextureAspect, TextureFormat, TextureUsages,
};

use crate::{
	core::{gameloop::Render, gpu::Gpu, render_target::RenderTarget},
	libs::{
		buffer::texture_buffer::{TextureBuffer, TextureBufferBacking},
		shader::{CompiledShader, ShaderBuilder},
		shader_fragment::{Renderer, ShaderFragment},
		smart_arc::Sarc,
		texture::{Edges, TextureAsset, TextureAssetDescriptor, TextureAssetDimensions, TextureAssetSamplerDescriptor},
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
	pub output_texture: Sarc<TextureAsset>,
	shader: CompiledShader,
}

impl ComputeRenderer {
	pub fn new(gpu: &Gpu, resolution: ScreenSize, renderer: &dyn ShaderFragment) -> Self {
		// The output texture that the compute will write to
		let output_texture = Sarc::new(TextureAsset::create_with_sampler(
			gpu,
			TextureAssetDescriptor {
				label: "Compute Renderer output",
				dimensions: TextureAssetDimensions::D2(resolution),
				format: TextureFormat::Rgba32Float,
				usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
				aspect: TextureAspect::All,
			},
			TextureAssetSamplerDescriptor {
				edges: Edges::ClampToColor(SamplerBorderColor::TransparentBlack),
				filter: FilterMode::Nearest,
				compare: None,
			},
		));

		// Dynamically create shader from the renderer
		let shader = ShaderBuilder::new()
			.include_texture(TextureBuffer::new(
				"out_texture",
				StorageTextureAccess::ReadWrite,
				TextureBufferBacking::From(output_texture.clone()),
			))
			.include_path("compute.wgsl")
			.include(renderer.shader())
			.build(gpu, &ShaderAssets, ShaderStages::COMPUTE, 0)
			.expect("Couldn't build shader");

		let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Compute Pipeline Layout"),
			bind_group_layouts: &shader.buffers.layouts(),
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

	let out_width = compute_renderer.output_texture.texture.width();
	let out_height = compute_renderer.output_texture.texture.height();

	{
		let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
			label: Some("ComputeRenderer Compute Pass"),
			timestamp_writes: None,
		});

		compute_pass.set_pipeline(&compute_renderer.pipeline);

		compute_renderer.shader.buffers.apply_to_compute_pass(&mut compute_pass);

		compute_pass.dispatch_workgroups(out_width, out_height, 1);
	}

	render_target.command_queue.push(encoder.finish());
}
