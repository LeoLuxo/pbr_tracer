pub mod core;
pub mod rendering;
pub mod util;

use core::{
	display::DisplayPlugin,
	event::EventPlugin,
	gameloop::{GameloopPlugin, Render},
};

use bevy_ecs::schedule::IntoSystemSetConfigs;
use bevy_tasks::{AsyncComputeTaskPool, TaskPool};
use brainrot::bevy::App;
use rendering::{
	compose::{ComposeRenderPass, ComposeRendererPlugin},
	render_target::{InnerRenderPass, PostRenderPass, PreRenderPass, RenderPass, WindowRenderTargetPlugin},
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
		.add_plugin(GameloopPlugin)
		.add_plugin(EventPlugin)
		.add_plugin(DisplayPlugin)
		.add_plugin(WindowRenderTargetPlugin)
		// // Rendering plugins
		.add_plugin(ComposeRendererPlugin)
		// .add_plugin(WindowRenderTargetPlugin)
		// .add_plugin(ChunkRendererPlugin)
		// .add_plugin(DebugRendererPlugin)
		// .add_plugin(DebugGuiPlugin)
		// // Universe plugins
		// .add_plugin(CameraPlugin)
		// .add_plugin(TerrainPlugin)
		// // Configure Renderpass order
		.configure_sets(
			Render,
			((
				PreRenderPass,
				(ComposeRenderPass,).chain().in_set(InnerRenderPass),
				PostRenderPass,
			)
				.chain()
				.in_set(RenderPass),),
		)
		.run();
}
