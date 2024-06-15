use std::sync::Arc;

use bevy_ecs::{
	event::EventReader,
	schedule::{IntoSystemConfigs, IntoSystemSetConfigs},
	system::{Res, ResMut},
};
use brainrot::{
	bevy::{self, App, Plugin},
	math::Converter,
	ScreenSize,
};
use wgpu::{
	CommandBuffer, PresentMode, Surface, SurfaceCapabilities, SurfaceConfiguration, SurfaceTexture, TextureUsages,
	TextureView, TextureViewDescriptor,
};
use winit::window::Window;

use super::event_processing::{EventReaderProcessor, ProcessedChangeEvents};
use crate::core::{
	display::{AppWindow, Gpu},
	events::WindowResizedEvent,
	gameloop::{Render, Update},
};

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
		app.add_systems(
			Render,
			(
				prepare_render_pass.in_set(PreRenderPass),
				finish_render_pass.in_set(PostRenderPass),
			)
				.chain()
				.in_set(RenderPass),
		);
		app.configure_sets(Render, InnerRenderPass.run_if(is_render_pass_valid));
	}
}

#[derive(bevy::SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderPass;

#[derive(bevy::SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PreRenderPass;

#[derive(bevy::SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InnerRenderPass;

#[derive(bevy::SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostRenderPass;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

//TODO make RenderTarget into a component (or other) to support multiple draw surfaces, and separate render into its own file under /rendering/

#[derive(bevy::Resource)]
pub struct RenderTarget<'a> {
	pub surface: Surface<'a>,
	pub size: ScreenSize,
	pub capabilities: SurfaceCapabilities,
	pub config: SurfaceConfiguration,

	pub command_queue: Vec<CommandBuffer>,

	current_texture: Option<SurfaceTexture>,
	pub current_view: Option<TextureView>,
}

impl<'a> RenderTarget<'a> {
	fn from_window(window: Arc<Window>, gpu: &Gpu) -> Self {
		// Window is passed as arc so that the surface creation can be done safely

		let size = window.inner_size().convert();

		// Create the rendering surface on which wgpu will render, from a raw_window_handle
		let surface = gpu
			.instance
			.create_surface(window)
			.expect("Couldn't create surface from window");

		// Describes what the surface is compatible with on the given adapter
		let capabilities = surface.get_capabilities(&gpu.adapter);

		// According to the docs, the first format is normally the preferred one
		let surface_format = capabilities.formats[0];

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

fn is_render_pass_valid(render_target: Res<RenderTarget<'static>>) -> bool {
	render_target.current_view.is_some()
}

fn prepare_render_pass(mut render_target: ResMut<RenderTarget<'static>>) {
	// trace!("Preparing render pass");

	// Get the output texture to render to and create a view for it.
	// A texture view is essentially like a "pointer" to the texture data
	let output = render_target.surface.get_current_texture().ok();
	let view = output
		.as_ref()
		.map(|output| output.texture.create_view(&TextureViewDescriptor::default()));

	render_target.current_texture = output;
	render_target.current_view = view;
}

fn finish_render_pass(mut render_target: ResMut<RenderTarget<'static>>, gpu: Res<Gpu>) {
	// trace!("Finishing render pass");

	// Submit the encoded command buffer to the queue
	// And clear queue at the same time
	gpu.queue.submit(render_target.command_queue.drain(..));

	// Swap the draw buffers and show what we rendered to the screen
	if let Some(output) = render_target.current_texture.take() {
		output.present();
	}
}

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
