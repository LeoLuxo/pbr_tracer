use bevy_ecs::{
	query::With,
	schedule::IntoSystemConfigs,
	system::{Res, ResMut},
};
use brainrot::{
	bevy::{self, App, Plugin},
	vec2,
	vek::Vec2,
	ScreenSize,
};
use wgpu::{
	Buffer, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, FilterMode,
	SamplerBorderColor, ShaderStages, StorageTextureAccess,
};

use super::camera_view::CameraView;
use crate::{
	core::{camera::Camera, gameloop::Render, gpu::Gpu, render_target::RenderTarget},
	libs::{
		buffer::{
			storage_texture_buffer::StorageTexture, uniform_buffer::UniformBufferDescriptor, BufferMappingApplicable,
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
	pub workgroup_size: Vec2<u32>,
	pub resolution: ScreenSize,
	pub filter_mode: FilterMode,
	pub renderer: R,
}

impl<R> Plugin for ComputeRendererPlugin<R>
where
	R: Renderer + 'static,
{
	fn build(&self, app: &mut App) {
		let camera_buffer = app
			.world
			.query_filtered::<&Sarc<Buffer>, With<Camera>>()
			.single(&app.world)
			.clone();

		let gpu = app.world.resource::<Gpu>();

		// TODO: Somehow clean up all the plugin vs resource instance stuff?
		let compute_renderer = ComputeRenderer::new(
			gpu,
			self.workgroup_size,
			self.resolution,
			self.filter_mode,
			&self.renderer,
			camera_buffer,
		);

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
	workgroup_size: Vec2<u32>,
	resolution: ScreenSize,
	pipeline: ComputePipeline,
	shader: CompiledShader,
	pub output_textures: Vec<Sarc<Tex>>,
}

impl ComputeRenderer {
	pub fn new(
		gpu: &Gpu,
		workgroup_size: Vec2<u32>,
		resolution: ScreenSize,
		filter_mode: FilterMode,
		renderer: &dyn Renderer,
		camera_buffer: Sarc<Buffer>,
	) -> Self {
		// Dynamically create shader from the renderer
		let mut shader = ShaderBuilder::new();
		shader
			.include_path("compute.wgsl")
			.include(renderer.shader())
			.define("WORKGROUP_X", format!("{}", workgroup_size.x))
			.define("WORKGROUP_Y", format!("{}", workgroup_size.y))
			.include_buffer(UniformBufferDescriptor::FromBuffer::<CameraView, _> {
				var_name: "camera",
				buffer: camera_buffer,
			});

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
			shader.include_buffer(StorageTexture::FromTex {
				var_name: var_name.clone(),
				access: StorageTextureAccess::ReadWrite,
				tex: tex.clone(),
			});
		}

		let output_textures = output_textures.into_iter().map(|(_, tex)| tex).collect::<Vec<_>>();

		// Compile the shader
		let shader = shader
			.build(gpu, "Compute shader", &ShaderAssets, ShaderStages::COMPUTE, 0)
			.expect("Couldn't build shader");

		let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Compute Pipeline Layout"),
			bind_group_layouts: &shader.layouts(),
			push_constant_ranges: &[],
		});

		let pipeline = gpu.device.create_compute_pipeline(&ComputePipelineDescriptor {
			label: Some("Compute pipeline"),
			layout: Some(&pipeline_layout),
			module: &shader.shader_module,
			entry_point: "main",
		});

		Self {
			workgroup_size,
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

		compute_pass.apply_buffer_mapping(&compute_renderer.shader.binding);

		let workgroups = <Vec2<u32>>::from(compute_renderer.resolution) / compute_renderer.workgroup_size + vec2!(1);
		compute_pass.dispatch_workgroups(workgroups.x, workgroups.y, 1);
	}

	render_target.command_queue.push(encoder.finish());
}
