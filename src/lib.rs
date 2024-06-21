pub mod core;
pub mod renderers;

use core::{
	display::DisplayPlugin,
	event_processing::EventProcessingPlugin,
	events::EventsPlugin,
	gameloop::{GameloopPlugin, Render},
	gpu::GpuPlugin,
	render_target::WindowRenderTargetPlugin,
	rendering::{
		composite::{CompositeRenderPass, CompositeRendererPlugin},
		compute::{ComputeRenderPass, ComputeRendererPlugin},
		render::{InnerRenderPass, PostRenderPass, PreRenderPass, RenderPass, RenderPlugin},
		render_fragments::PostProcessingPipeline,
	},
};

use bevy_ecs::schedule::IntoSystemSetConfigs;
use bevy_tasks::{AsyncComputeTaskPool, TaskPool};
use brainrot::{bevy::App, include_shader_source_map, size, ShaderSourceMap};
use renderers::{pathtracer::PhysBasedRaytracer, post_processing::GammaCorrection};

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

pub fn run() {
	AsyncComputeTaskPool::get_or_init(TaskPool::new);

	App::new()
		// Standalone raytracer plugins
		.add_plugin(GpuPlugin)
		.add_plugin(ComputeRendererPlugin {
			resolution: size!(10, 10),
			renderer: PhysBasedRaytracer {
				ppp: Some(PostProcessingPipeline::new().add_effect(GammaCorrection)),
			},
		})
		// Core plugins
		.add_plugin(EventProcessingPlugin)
		.add_plugin(EventsPlugin)
		.add_plugin(GameloopPlugin)
		.add_plugin(DisplayPlugin)
		.add_plugin(WindowRenderTargetPlugin)
		// Rendering plugins
		.add_plugin(RenderPlugin)
		.add_plugin(CompositeRendererPlugin)
		// Configure Renderpass order
		.configure_sets(
			Render,
			((
				PreRenderPass,
				(ComputeRenderPass, CompositeRenderPass).chain().in_set(InnerRenderPass),
				PostRenderPass,
			)
				.chain()
				.in_set(RenderPass),),
		)
		.run();
}
