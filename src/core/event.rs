use std::marker::PhantomData;

use bevy_ecs::{
	event::{Event, Events},
	schedule::{IntoSystemConfigs, ScheduleLabel},
	system::{Res, ResMut},
	world::World,
};
use brainrot::{
	bevy::{self, App, Plugin},
	MouseMotionDelta, ScreenSize,
};

use super::gameloop::{EventsCore, IterStep, Render, Update};

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

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct EventPlugin;

impl Plugin for EventPlugin {
	fn build(&self, app: &mut App) {
		// Signal into the dedicated resource when the schedule was run
		app.init_resource::<ScheduleSignal<IterStep>>();
		app.init_resource::<ScheduleSignal<Update>>();
		app.init_resource::<ScheduleSignal<Render>>();
		app.add_systems(IterStep, signal_schedule::<IterStep>);
		app.add_systems(Update, signal_schedule::<Update>);
		app.add_systems(Render, signal_schedule::<Render>);

		// Signal that events should be cleared
		app.init_resource::<ClearEvents>();

		// Order of operation:
		// * check_signals
		// * if check passed
		// * 	clear_events (for each event type)
		// *	reset_signals + clear_trackers
		app.add_systems(EventsCore, (check_signals, reset_signals).chain());

		// Register all events
		add_event::<KeyboardInputEvent>(app);
		add_event::<MouseMotionEvent>(app);
		add_event::<MouseWheelEvent>(app);
		add_event::<WindowResizedEvent>(app);
		add_event::<WinitWindowEvent>(app);
	}
}

#[derive(bevy::Resource)]
pub struct ScheduleSignal<S> {
	counter: usize,
	_marker: PhantomData<S>,
}

#[derive(bevy::Resource, Default)]
pub struct ClearEvents {
	clear: bool,
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

impl<S> Default for ScheduleSignal<S> {
	fn default() -> Self {
		Self {
			counter: 0,
			_marker: Default::default(),
		}
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

/// Setup the world to manage events of type `E`.
///
/// This is done by adding a [`Resource`] of type [`Events::<E>`].
/// See [`Events`] for defining events.
pub fn add_event<E: Event>(app: &mut App) {
	if !app.world.contains_resource::<Events<E>>() {
		app.world.init_resource::<Events<E>>();
		app.add_systems(
			EventsCore,
			try_clear_events::<E>
				.after(check_signals)
				.before(reset_signals)
				.run_if(bevy_ecs::event::event_update_condition::<E>),
		);
	}
}

/// This function should be run from a specific schedule that matches type `S`.
/// This signals that schedule `S` was indeed called
pub fn signal_schedule<S: ScheduleLabel>(mut schedule_signal: ResMut<ScheduleSignal<S>>) {
	schedule_signal.counter += 1;
}

/// Checks whether all schedule signals are set (thus all schedules were run), if yes signal the events to be cleared
pub fn check_signals(world: &mut World) {
	let signal_iterstep = world.resource::<ScheduleSignal<IterStep>>();
	let signal_update = world.resource::<ScheduleSignal<Update>>();
	let signal_render = world.resource::<ScheduleSignal<Render>>();

	// Only clear events etc if ALL the schedules have been called AT LEAST TWICE since last time
	// If it was only for the event queue clearing, checking that they were called only ONCE would be enough, since the event queues are double-buffered.
	// For the world trackers though, if a change were to happen at the end of an iteration (in render for example) where all three schedules were run,
	// then the trackers would be cleared before the next iteration and none of the other schedules would have been able to detect the change.
	if signal_iterstep.counter >= 2 && signal_update.counter >= 2 && signal_render.counter >= 2 {
		// Signal that events should be cleared
		// (done in a second step because the generic clearing function needs to be statically called for each event type).
		// Signals will be reset after all the events are cleared.
		world.resource_mut::<ClearEvents>().clear = true;
	}
}

/// Clear the specific event type `E` if the `ClearEvents` signal was set
pub fn try_clear_events<E: 'static + Send + Sync + Event>(
	clear_events: Res<ClearEvents>,
	mut events: ResMut<Events<E>>,
) {
	if clear_events.clear {
		// Clear the event queues from the world for event type E
		// Using the update() function makes it double-buffer instead of truly "clearing" them
		events.update();
	}
}

/// After clearing events, reset the schedule signals + `ClearEvents` signal
/// This needs to be done in two steps to make sure ALL events types (Events<T>) get cleared before resetting the counter.
/// Also clear trackers here to avoid doing it multiple times.
pub fn reset_signals(world: &mut World) {
	let mut clear_events = world.resource_mut::<ClearEvents>();

	if clear_events.clear {
		clear_events.clear = false;

		// Reset the counters
		world.resource_mut::<ScheduleSignal<IterStep>>().counter = 0;
		world.resource_mut::<ScheduleSignal<Update>>().counter = 0;
		world.resource_mut::<ScheduleSignal<Render>>().counter = 0;

		// Calling clear_trackers here makes sure that all my schedules will have had a chance to detect changes,
		// BUT with the drawback that they might detect a change for multiple frames, making it unreliable-ish.
		// So change-detection should be used sparingly, in favor of events (since those include automatic per-system read tracking).
		world.clear_trackers();
	}
}
