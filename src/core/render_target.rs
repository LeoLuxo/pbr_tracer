use std::sync::Arc;

use bevy_ecs::{
	event::EventReader,
	system::{Res, ResMut},
};
use brainrot::{
	bevy::{self, App, Plugin},
	Converter, ScreenSize,
};
use wgpu::{
	CommandBuffer, PresentMode, Surface, SurfaceCapabilities, SurfaceConfiguration, SurfaceTexture, TextureUsages,
	TextureView,
};
use winit::window::Window;

use super::{
	event_processing::{EventReaderProcessor, ProcessedChangeEvents},
	gpu::Gpu,
};
use crate::core::{display::AppWindow, events::WindowResizedEvent, gameloop::Update};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct WindowRenderTargetPlugin;

impl Plugin for WindowRenderTargetPlugin {
	fn build(&self, app: &mut App) {
		let app_window = app.world.resource::<AppWindow>();
		let gpu = app.world.resource::<Gpu>();

		let render_target = RenderTarget::from_window(app_window.winit_window.clone(), gpu);

		app.world.insert_resource(render_target);

		app.add_systems(Update, resize);
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

//TODO make RenderTarget into a component (or other) to support multiple draw
// surfaces

#[derive(bevy::Resource)]
pub struct RenderTarget<'a> {
	pub surface: Surface<'a>,
	pub size: ScreenSize,
	pub capabilities: SurfaceCapabilities,
	pub config: SurfaceConfiguration,

	pub command_queue: Vec<CommandBuffer>,

	pub current_texture: Option<SurfaceTexture>,
	pub current_view: Option<TextureView>,
}

impl<'a> RenderTarget<'a> {
	fn from_window(window: Arc<Window>, gpu: &Gpu) -> Self {
		// Window is passed as arc so that the surface creation can be done safely

		let size = window.inner_size().convert();

		// Create the rendering surface on which wgpu will render, from a
		// raw_window_handle
		let surface = gpu
			.instance
			.create_surface(window)
			.expect("Couldn't create surface from window");

		// Describes what the surface is compatible with on the given adapter
		let capabilities = surface.get_capabilities(&gpu.adapter);

		// According to the docs, the first format is normally the preferred one
		// Force it to be srgb so that gamma correction is done by the GPU
		let surface_format = capabilities.formats[0].add_srgb_suffix();
		println!("{:?}", capabilities.formats);

		let present_mode = if capabilities.present_modes.contains(&PresentMode::Mailbox) {
			// For some reason FIFO is jittery on my desktop PC, so prioritize Mailbox
			PresentMode::Mailbox
		} else {
			PresentMode::AutoNoVsync
		};

		// Configure the surface
		let config = SurfaceConfiguration {
			usage: TextureUsages::RENDER_ATTACHMENT,
			format: surface_format,
			width: size.w,
			height: size.h,
			present_mode,
			desired_maximum_frame_latency: 2,
			alpha_mode: capabilities.alpha_modes[0],
			view_formats: vec![],
		};

		surface.configure(&gpu.device, &config);

		Self {
			surface,
			size,
			capabilities,
			config,
			command_queue: vec![],
			current_texture: None,
			current_view: None,
		}
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

fn resize(
	mut render_target: ResMut<RenderTarget<'static>>,
	gpu: Res<Gpu>,
	window_events: EventReader<WindowResizedEvent>,
) {
	if let Some(size) = window_events.process().latest() {
		render_target.config.width = size.w;
		render_target.config.height = size.h;
		render_target.surface.configure(&gpu.device, &render_target.config);
	}
}
