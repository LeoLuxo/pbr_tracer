pub mod core;
pub mod fragments;
pub mod libs;

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
	},
};

use bevy_ecs::schedule::IntoSystemSetConfigs;
use bevy_tasks::{AsyncComputeTaskPool, TaskPool};
use brainrot::{bevy::App, size};
use fragments::{
	intersectors::Raymarcher,
	pathtracer::PhysBasedRaytracer,
	post_processing::{GammaCorrection, PostProcessingPipeline},
};
use rust_embed::Embed;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(Embed)]
#[folder = "src/shader/"]
#[prefix = "/"]
struct ShaderAssets;

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
			resolution: size!(1000, 500),
			renderer: PhysBasedRaytracer {
				intersector: Raymarcher,
				ppp: PostProcessingPipeline::empty().with(GammaCorrection),
			},
			// renderer: DebugRenderer,
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
