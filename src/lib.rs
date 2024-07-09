pub mod core;
pub mod fragments;
pub mod libs;

use core::{
	camera::CameraPlugin,
	display::DisplayPlugin,
	event_processing::EventProcessingPlugin,
	events::EventsPlugin,
	gameloop::{GameloopPlugin, Render},
	gpu::GpuPlugin,
	render_target::WindowRenderTargetPlugin,
	rendering::{
		camera_view::CameraViewPlugin,
		composite::{CompositeRenderPass, CompositeRendererPlugin},
		compute::{ComputeRenderPass, ComputeRendererPlugin},
		render::{InnerRenderPass, PostRenderPass, PreRenderPass, RenderPass, RenderPlugin},
	},
};

use bevy_ecs::schedule::IntoSystemSetConfigs;
use bevy_tasks::{AsyncComputeTaskPool, TaskPool};
use brainrot::{bevy::App, size, vec2};
use fragments::{intersector::*, mpr::MultiPurposeRenderer, post_processing::PostProcessingPipeline, shading::*};
use image::DynamicImage;
use rust_embed::Embed;
use wgpu::FilterMode;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(Embed)]
#[folder = "src/shader/"]
#[prefix = "/"]
struct ShaderAssets;

#[derive(Embed)]
#[folder = "assets/"]
struct TextureAssets;

impl TextureAssets {
	pub fn get_image(path: &str) -> DynamicImage {
		image::load_from_memory(&Self::get(path).expect("Invalid image path").data)
			.expect("Couldn't load image bytes from memory")
	}
}

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

	let renderer = MultiPurposeRenderer {
		intersector: Raymarcher,
		shading: CelShading,
		post_processing: PostProcessingPipeline::empty(),
	};

	App::new()
		// Core plugins
		.add_plugin(GpuPlugin)
		.add_plugin(CameraPlugin)
		.add_plugin(CameraViewPlugin)
		.add_plugin(EventProcessingPlugin)
		.add_plugin(EventsPlugin)
		.add_plugin(GameloopPlugin)
		.add_plugin(DisplayPlugin)
		.add_plugin(WindowRenderTargetPlugin)
		// Compute renderer
		.add_plugin(ComputeRendererPlugin {
			workgroup_size: vec2!(16, 16),
			resolution: size!(2000, 1000),
			filter_mode: FilterMode::Linear,
			renderer,
			// renderer: DebugRenderer,
		})
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
