pub mod core;
pub mod renderers;

use core::{
	display::DisplayPlugin,
	event_processing::EventProcessingPlugin,
	events::EventsPlugin,
	gameloop::{GameloopPlugin, Render},
	gpu::GpuPlugin,
	render_target::WindowRenderTargetPlugin,
};

use bevy_ecs::schedule::IntoSystemSetConfigs;
use bevy_tasks::{AsyncComputeTaskPool, TaskPool};
use brainrot::{bevy::App, include_shader_source_map, ShaderSourceMap};
use core::rendering::{
	compose::{ComposeRenderPass, ComposeRendererPlugin},
	compute::{ComputeRenderPass, ComputeRendererPlugin},
	render::{InnerRenderPass, PostRenderPass, PreRenderPass, RenderPass, RenderPlugin},
};
use renderers::pathtracer::PhysBasedRaytracer;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

const SHADER_MAP: ShaderSourceMap = include_shader_source_map!();

pub trait EntityLabel {}

/// The default `EventLoop` type to avoid having to add the extra unit type
type EventLoop = winit::event_loop::EventLoop<()>;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

type CurrentRenderer = PhysBasedRaytracer;

pub fn run() {
	AsyncComputeTaskPool::get_or_init(TaskPool::new);

	App::new()
		// Standalone raytracer plugins
		.add_plugin(GpuPlugin)
		.add_plugin(ComputeRendererPlugin {
			asd: PhysBasedRaytracer,
		})
		// Core plugins
		.add_plugin(EventProcessingPlugin)
		.add_plugin(EventsPlugin)
		.add_plugin(GameloopPlugin)
		.add_plugin(DisplayPlugin)
		.add_plugin(WindowRenderTargetPlugin)
		// Rendering plugins
		.add_plugin(RenderPlugin)
		.add_plugin(ComposeRendererPlugin)
		// Configure Renderpass order
		.configure_sets(
			Render,
			((
				PreRenderPass,
				(ComputeRenderPass, ComposeRenderPass).chain().in_set(InnerRenderPass),
				PostRenderPass,
			)
				.chain()
				.in_set(RenderPass),),
		)
		.run();
}
