use std::sync::Arc;

use bevy_ecs::{
	change_detection::DetectChanges,
	event::EventReader,
	system::{ResMut, Resource},
};

use brainrot::{
	bevy::{App, Plugin},
	math::Converter,
	size, ScreenSize,
};
use wgpu::{
	Adapter, Backends, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, InstanceFlags, Limits,
	PowerPreference, Queue, RequestAdapterOptions, Surface,
};
use winit::{
	dpi::{PhysicalPosition, PhysicalSize},
	event::WindowEvent,
	keyboard::KeyCode,
	window::{CursorGrabMode, Window, WindowBuilder},
};

use crate::{
	core::{
		event::{KeyboardInputEvent, WinitWindowEvent},
		gameloop::Update,
	},
	util::event_processing::EventReaderProcessor,
};
use crate::{util::event_processing::ProcessedInputEvents, EventLoop};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct DisplayPlugin;

impl Plugin for DisplayPlugin {
	fn build(&self, app: &mut App) {
		let window_settings = WindowSettings {
			title: "Pew Pew Ray Thingie",
			size: size!(1600, 900),
		};

		let event_loop = EventLoop::new().expect("Couldn't create winit event_loop");
		let app_window = AppWindow::new(&event_loop, &window_settings);
		let gpu = pollster::block_on(Gpu::new(None));

		app.world.insert_resource(window_settings);
		app.world.insert_non_send_resource(event_loop);
		app.world.insert_resource(app_window);
		app.world.insert_resource(gpu);

		app.add_systems(Update, toggle_cursor_attached);
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(Resource, Copy, Clone, Debug, Default)]
pub struct WindowSettings {
	pub title: &'static str,
	pub size: ScreenSize,
} // TODO either update this on resize or delete or make immutable or something

#[derive(Resource)]
pub struct AppWindow {
	// Window needs to be an arc so that a surface can be created from it safely
	pub winit_window: Arc<winit::window::Window>,

	pub cursor_attached: bool,
}

#[derive(Resource)]
pub struct Gpu {
	pub instance: Instance,
	pub adapter: Adapter,
	pub device: Device,
	pub queue: Queue,
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

impl AppWindow {
	#[must_use]
	pub fn new(event_loop: &EventLoop, settings: &WindowSettings) -> Self {
		let window = WindowBuilder::new()
			.with_title(settings.title)
			.with_inner_size(Converter::<PhysicalSize<u32>>::convert(settings.size))
			.build(event_loop)
			.expect("Couldn't build winit window from event loop");

		// Center the window
		if let Some(monitor) = window.current_monitor() {
			let screen_size = monitor.size();
			let window_size = window.outer_size();

			window.set_outer_position(winit::dpi::PhysicalPosition {
				x: f64::from(screen_size.width.saturating_sub(window_size.width)) / 2.
					+ f64::from(monitor.position().x),
				y: f64::from(screen_size.height.saturating_sub(window_size.height)) / 2.
					+ f64::from(monitor.position().y),
			});
		}

		Self {
			winit_window: Arc::new(window),
			cursor_attached: true,
		}
	}
}

impl Gpu {
	async fn new(compatible_surface: Option<&Surface<'_>>) -> Self {
		// Instance is the instance of wgpu which serves as entrypoint for everything wgpu-related
		#[cfg(debug_assertions)]
		// Not running in --release mode, activate validation and debug info for wgpu
		let instance = Instance::new(InstanceDescriptor {
			backends: Backends::PRIMARY,
			flags: InstanceFlags::VALIDATION | InstanceFlags::DEBUG,
			..Default::default()
		});

		#[cfg(not(debug_assertions))]
		// Running in --release mode, don't activate debugging infos for wgpu
		let instance = Instance::new(InstanceDescriptor {
			backends: Backends::PRIMARY,
			..Default::default()
		});

		// Adapter essentially represents the physical GPU + the Backend, e.g. GTX1080_VK; GTX1080_DX12; etc
		let adapter = instance
			.request_adapter(&RequestAdapterOptions {
				power_preference: PowerPreference::HighPerformance,
				compatible_surface,
				force_fallback_adapter: false,
			})
			.await
			.expect("Coudln't request compatible adapter");

		// Device esentially acts like a logical connection to the selected adapter in an application-isolated way. The device is selected based on a descriptor that describes the required features.
		// Queue is the message queue / command buffer for the GPU, anything that the GPU needs to do should be requested into that queue (i.e. rendering, uploading buffer data, etc)
		let (device, queue) = adapter
			.request_device(
				&(DeviceDescriptor {
					required_features: Features::empty()
						| Features::TEXTURE_BINDING_ARRAY
						| Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
						| Features::CONSERVATIVE_RASTERIZATION,
					required_limits: Limits::default(),
					label: None,
				}),
				None,
			)
			.await
			.expect("Couldn't request device");

		Self {
			instance,
			adapter,
			device,
			queue,
		}
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

fn toggle_cursor_attached(
	mut app_window: ResMut<AppWindow>,
	keyboard_events: EventReader<KeyboardInputEvent>,
	mut winit_events: EventReader<WinitWindowEvent>,
) {
	let mut needs_update = false;
	let mut needs_reset = false;

	if keyboard_events.process().has_pressed(KeyCode::Escape) {
		app_window.cursor_attached = !app_window.cursor_attached;
		needs_update = true;
		needs_reset = true;
	}

	for WinitWindowEvent(winit_event) in winit_events.read() {
		match winit_event {
			WindowEvent::Focused(..)
			| WindowEvent::CursorEntered { .. }
			| WindowEvent::CursorLeft { .. }
			| WindowEvent::MouseInput { .. } => needs_update = true,
			_ => {}
		}
	}

	// If the display was externally changed, or its cursor_attached was just changed
	needs_update = needs_update || app_window.is_changed();

	// Then detach or attach the cursor
	if needs_update {
		if app_window.cursor_attached {
			attach_cursor(&app_window.winit_window);
		} else {
			// Might lead to some problems down the line, if the cursor_attached value was changed externally, then needs_reset wouldn't be set
			detach_cursor(&app_window.winit_window, needs_reset);
		}
	}
}

fn attach_cursor(window: &Window) {
	// game is focused: hide the cursor and lock it in place
	window.set_cursor_visible(false);

	// [`Locked`] keeps the cursor stuck in the middle of the window (not implemented in windows), [`Confined`] keeps the cursor within the bounds of the window
	window
		.set_cursor_grab(CursorGrabMode::Locked)
		.or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
		.unwrap();
}

fn detach_cursor(window: &Window, reset: bool) {
	// menu is focused: show and unlock the cursor
	window.set_cursor_visible(true);
	window.set_cursor_grab(CursorGrabMode::None).unwrap();

	// to make sure the cursor appears to "spawn" in the middle of the window when it appears, force it back to the middle of the window
	if reset {
		window
			.set_cursor_position(PhysicalPosition::<u32> {
				x: window.inner_size().width / 2,
				y: window.inner_size().height / 2,
			})
			.unwrap();
	}
}
