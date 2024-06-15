use bevy_ecs::event::Event;
use brainrot::{
	bevy::{App, Plugin},
	MouseMotionDelta, ScreenSize,
};

use super::event_processing::add_event;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct EventsPlugin;

impl Plugin for EventsPlugin {
	fn build(&self, app: &mut App) {
		// Register all events
		add_event::<KeyboardInputEvent>(app);
		add_event::<MouseMotionEvent>(app);
		add_event::<MouseWheelEvent>(app);
		add_event::<WindowResizedEvent>(app);
		add_event::<WinitWindowEvent>(app);
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

/// Event for keyboard inputs.
///
/// The fields are taken directly from the corresponding [`winit::event::WindowEvent::KeyboardInput`].
/// [`Self::logical_key`] is how the OS interprets the key, so with keyboard layout and other stuff applied.
/// [`Self::physical_key`] is how the physical keyboard sent the key, so without any keyboard layout information.
/// Keys are named after their physical position on a US keyboard layout.
#[allow(dead_code)]
#[derive(Event, Clone, Debug, PartialEq, Eq)]
pub struct KeyboardInputEvent {
	pub state: winit::event::ElementState,
	pub logical_key: winit::keyboard::Key,
	pub physical_key: winit::keyboard::PhysicalKey,
}

// TODO add special Resource that takes in these raw events and then keeps track of continuous keyboard presses and clicks to make it easier to work with

/// Event for mouse motion.
///
/// The [`Self::motion_delta`] field is taken directly from the corresponding [`winit::event::DeviceEvent::MouseMotion`].
/// Since the [`winit::event::DeviceEvent`] version of the mouse motion event is used, it is suitable for 3D camera movement.
/// If UI cursor movement is required though, [`winit::event::WindowEvent::CursorMoved`] should be used instead.

#[derive(Event, Clone, Debug)]
pub struct MouseMotionEvent {
	pub motion_delta: MouseMotionDelta,
}

// TODO Missing event for UI cursor movement (using winit::event::WindowEvent::CursorMoved)

/// Event for mouse wheel motion.
///
/// The [`Self::wheel_delta`] field is taken directly from the corresponding [`winit::event::DeviceEvent::MouseWheel`].
/// I don't know thge difference between the [`winit::event::DeviceEvent`] version of the mouse wheel event and the [`winit::event::WindowEvent`] version.
#[allow(dead_code)]
#[derive(Event, Clone, Debug)]
pub struct MouseWheelEvent {
	pub wheel_delta: winit::event::MouseScrollDelta,
}

/// Event for mouse wheel motion.
///
/// The [`Self::wheel_delta`] field is taken directly from the corresponding [`winit::event::DeviceEvent::MouseWheel`].
/// I don't know thge difference between the [`winit::event::DeviceEvent`] version of the mouse wheel event and the [`winit::event::WindowEvent`] version.
#[allow(dead_code)]
#[derive(Event, Clone, Debug, PartialEq, Eq)]
pub struct MouseInputEvent {
	pub state: winit::event::ElementState,
	pub button: winit::event::MouseButton,
}

/// Event for when the window was resized.
///
/// Careful, might fire many times in a row when the window is currently being drag-resized.
/// Corresponds to [`winit::event::WindowEvent::Resized`].

#[derive(Event, Clone, Debug, PartialEq, Eq)]
pub struct WindowResizedEvent {
	pub size: ScreenSize,
}

/// Event for *any* [`winit`] window event that might have been fired.
/// Newtype wrapper around [`winit::event::WindowEvent`].
/// Mostly useful for passing along the raw [`winit`] events (to [`egui`] for example).
/// For any specialized cases, corresponding events should be used.

#[derive(Event, Clone, Debug)]
pub struct WinitWindowEvent(pub winit::event::WindowEvent);
