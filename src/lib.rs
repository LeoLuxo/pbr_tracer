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
use brainrot::{bevy::App, engine_3d::ShaderFile};
use once_cell::sync::Lazy;
use rendering::{
	compose::{ComposeRenderPass, ComposeRendererPlugin},
	compute::{ComputeRenderPass, ComputeRendererPlugin},
	render::{InnerRenderPass, PostRenderPass, PreRenderPass, RenderPass, RenderPlugin},
};

use include_dir::{include_dir, Dir};
/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

static SHADER_FILES: Lazy<Vec<ShaderFile>> = Lazy::new(|| {
	static DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/shader");

	DIR.files()
		.map(|f| ShaderFile {
			file_name: f.path().to_str().unwrap(),
			shader_source: f.contents_utf8().unwrap(),
		})
		.collect()
});

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
				(ComputeRenderPass, ComposeRenderPass).chain().in_set(InnerRenderPass),
				PostRenderPass,
			)
				.chain()
				.in_set(RenderPass),),
		)
		.run();
}
