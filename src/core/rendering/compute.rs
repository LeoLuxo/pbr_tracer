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
	SamplerBorderColor, ShaderStages, StorageTextureAccess,
};

use crate::{
	core::{gameloop::Render, gpu::Gpu, render_target::RenderTarget},
	libs::{
		buffer::{
			storage_texture_buffer::{StorageTextureBuffer, StorageTextureBufferBacking},
			BufferMappingApplicable,
		},
		shader::{CompiledShader, ShaderBuilder},
		shader_fragment::Renderer,
		smart_arc::Sarc,
		texture::{SamplerEdges, Tex, TexSamplerDescriptor},
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
	pub filter_mode: FilterMode,
	pub renderer: R,
}

impl<R> Plugin for ComputeRendererPlugin<R>
where
	R: Renderer + 'static,
{
	fn build(&self, app: &mut App) {
		let gpu = app.world.resource::<Gpu>();

		let compute_renderer = ComputeRenderer::new(gpu, self.resolution, self.filter_mode, &self.renderer);

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
	resolution: ScreenSize,
	pipeline: ComputePipeline,
	shader: CompiledShader,
	pub output_textures: Vec<Sarc<Tex>>,
}

impl ComputeRenderer {
	pub fn new(gpu: &Gpu, resolution: ScreenSize, filter_mode: FilterMode, renderer: &dyn Renderer) -> Self {
		// Dynamically create shader from the renderer
		let mut shader = ShaderBuilder::new();
		shader.include_path("compute.wgsl").include(renderer.shader());
		// .include_value("out_resolution", resolution);

		// The sampler that will be added to all output textures
		let output_sampler = Some(TexSamplerDescriptor {
			edges: SamplerEdges::ClampToColor(SamplerBorderColor::TransparentBlack),
			filter: filter_mode,
			compare: None,
		});

		// The list of output textures given by the renderer
		let output_textures = renderer
			.output_textures(resolution)
			.into_iter()
			.map(|(name, desc)| (name, Sarc::new(Tex::create(gpu, desc, output_sampler))))
			.collect::<Vec<_>>();

		// Add the output textures to the shader
		for (var_name, tex) in &output_textures {
			shader.include_texture(StorageTextureBuffer::new(
				var_name,
				StorageTextureAccess::ReadWrite,
				StorageTextureBufferBacking::WithBacking(tex.clone()),
			));
		}

		let output_textures = output_textures.into_iter().map(|(_, tex)| tex).collect::<Vec<_>>();

		// Compile the shader
		let shader = shader
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
			resolution,
			pipeline,
			shader,
			output_textures,
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

	{
		let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
			label: Some("ComputeRenderer Compute Pass"),
			timestamp_writes: None,
		});

		compute_pass.set_pipeline(&compute_renderer.pipeline);

		compute_pass.apply_buffer_mapping(&compute_renderer.shader.buffers);

		// TODO: Change workgroup size to 64
		compute_pass.dispatch_workgroups(compute_renderer.resolution.w, compute_renderer.resolution.h, 1);
	}

	render_target.command_queue.push(encoder.finish());
}
