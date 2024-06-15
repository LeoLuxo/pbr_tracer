use bevy_ecs::{schedule::IntoSystemConfigs, system::Res};
use brainrot::{
	bevy::{self, App, Plugin},
	src,
};
use wgpu::{include_wgsl, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor};

use crate::core::{display::Gpu, gameloop::Render};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct ComputeRendererPlugin;

impl Plugin for ComputeRendererPlugin {
	fn build(&self, app: &mut App) {
		let gpu = app.world.resource::<Gpu>();

		let compute_renderer = ComputeRenderer::new(gpu);

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
}

impl ComputeRenderer {
	pub fn new(gpu: &Gpu) -> Self {
		// Statically include the shader in the executable
		let shader = gpu
			.device
			.create_shader_module(include_wgsl!(src!("shader/compute.wgsl")));

		let pipeline = gpu.device.create_compute_pipeline(&ComputePipelineDescriptor {
			label: Some("Compute pipeline"),
			layout: None,
			module: &shader,
			entry_point: "main",
		});

		Self { pipeline }
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

fn render(compute_renderer: Res<ComputeRenderer>, gpu: Res<Gpu>) {
	let mut command_encoder = gpu.device.create_command_encoder(&CommandEncoderDescriptor {
		label: Some("ComputeRenderer Command Encoder"),
	});

	{
		let mut compute_pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
			label: Some("ComputeRenderer Compute Pass"),
			timestamp_writes: None,
		});

		compute_pass.set_pipeline(&compute_renderer.pipeline);
		compute_pass.dispatch_workgroups(100, 100, 1);
	}
}
