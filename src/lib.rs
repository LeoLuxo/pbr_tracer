pub mod core;
pub mod rendering;

use core::{
	display::DisplayPlugin,
	event_processing::EventProcessingPlugin,
	events::EventsPlugin,
	gameloop::{GameloopPlugin, Render},
	render_target::WindowRenderTargetPlugin,
};

use bevy_ecs::schedule::IntoSystemSetConfigs;
use bevy_tasks::{AsyncComputeTaskPool, TaskPool};
use brainrot::bevy::App;
use rendering::{
	compose::{ComposeRenderPass, ComposeRendererPlugin},
	compute::{ComputeRenderPass, ComputeRendererPlugin},
	render::{InnerRenderPass, PostRenderPass, PreRenderPass, RenderPass, RenderPlugin},
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub trait EntityLabel {}

// Some extra useful types

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
		// Core plugins
		.add_plugin(EventProcessingPlugin)
		.add_plugin(EventsPlugin)
		.add_plugin(GameloopPlugin)
		.add_plugin(DisplayPlugin)
		.add_plugin(WindowRenderTargetPlugin)
		// Rendering plugins
		.add_plugin(RenderPlugin)
		.add_plugin(ComputeRendererPlugin)
		.add_plugin(ComposeRendererPlugin)
		// Configure Renderpass order
		.configure_sets(
			Render,
			((
				PreRenderPass,
				(ComposeRenderPass, ComputeRenderPass).chain().in_set(InnerRenderPass),
				PostRenderPass,
			)
				.chain()
				.in_set(RenderPass),),
		)
		.run();
}
