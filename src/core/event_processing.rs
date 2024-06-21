use std::{marker::PhantomData, slice::Iter};

use bevy_ecs::{
	event::{Event, EventReader, Events},
	schedule::{IntoSystemConfigs, ScheduleLabel},
	system::{Res, ResMut},
	world::World,
};
use brainrot::{
	bevy::{self, App, Plugin},
	vec2, MouseMotionDelta, ScreenSize,
};
use events::{KeyboardInputEvent, MouseInputEvent, MouseMotionEvent, WindowResizedEvent};
use winit::{
	event::{ElementState, MouseButton},
	keyboard::{KeyCode, PhysicalKey},
};

use crate::core::{
	events,
	gameloop::{EventsCore, IterStep, Render, Update},
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct EventProcessingPlugin;

impl Plugin for EventProcessingPlugin {
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
		// * clear_events (for each event type)
		// *	reset_signals + clear_trackers
		app.add_systems(EventsCore, (check_signals, reset_signals).chain());
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

/// Checks whether all schedule signals are set (thus all schedules were run),
/// if yes signal the events to be cleared
pub fn check_signals(world: &mut World) {
	let signal_iterstep = world.resource::<ScheduleSignal<IterStep>>();
	let signal_update = world.resource::<ScheduleSignal<Update>>();
	let signal_render = world.resource::<ScheduleSignal<Render>>();

	// Only clear events etc if ALL the schedules have been called AT LEAST TWICE
	// since last time If it was only for the event queue clearing, checking that
	// they were called only ONCE would be enough, since the event queues are
	// double-buffered. For the world trackers though, if a change were to happen
	// at the end of an iteration (in render for example) where all three schedules
	// were run, then the trackers would be cleared before the next iteration and
	// none of the other schedules would have been able to detect the change.
	if signal_iterstep.counter >= 2 && signal_update.counter >= 2 && signal_render.counter >= 2 {
		// Signal that events should be cleared
		// (done in a second step because the generic clearing function needs to be
		// statically called for each event type). Signals will be reset after all the
		// events are cleared.
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
		// Using the update() function makes it double-buffer instead of truly
		// "clearing" them
		events.update();
	}
}

/// After clearing events, reset the schedule signals + `ClearEvents` signal
/// This needs to be done in two steps to make sure ALL events types (Events<T>)
/// get cleared before resetting the counter. Also clear trackers here to avoid
/// doing it multiple times.
pub fn reset_signals(world: &mut World) {
	let mut clear_events = world.resource_mut::<ClearEvents>();

	if clear_events.clear {
		clear_events.clear = false;

		// Reset the counters
		world.resource_mut::<ScheduleSignal<IterStep>>().counter = 0;
		world.resource_mut::<ScheduleSignal<Update>>().counter = 0;
		world.resource_mut::<ScheduleSignal<Render>>().counter = 0;

		// Calling clear_trackers here makes sure that all my schedules will have had a
		// chance to detect changes, BUT with the drawback that they might detect a
		// change for multiple frames, making it unreliable-ish. So change-detection
		// should be used sparingly, in favor of events (since those include automatic
		// per-system read tracking).
		world.clear_trackers();
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

// Some traits to make events less boilerplate-y

pub struct ProcessedEventReader<E: Event> {
	events: Vec<E>,
}

impl<E: Event> ProcessedEventReader<E> {
	#[allow(dead_code)]
	fn iter(&self) -> Iter<E> {
		self.events.iter()
	}
}

pub trait EventReaderProcessor<E: Event> {
	// Taking ownership of self on purpose here; since process() will read() and
	// thus consume the iterator behind EventReader, making sure I can't use the
	// EventReader anymore afterwards should hopefully prevent some dumb bugs
	fn process(self) -> ProcessedEventReader<E>;
}

impl<E: Event + Clone> EventReaderProcessor<E> for EventReader<'_, '_, E> {
	fn process(mut self) -> ProcessedEventReader<E> {
		ProcessedEventReader {
			events: self.read().cloned().collect(),
		}
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub trait ProcessedInputEvents {
	type KeyType;

	fn has_pressed(&self, key: Self::KeyType) -> bool;
	fn has_released(&self, key: Self::KeyType) -> bool;
	fn has_interacted(&self, key: Self::KeyType) -> bool;

	fn states(&self, key: Self::KeyType) -> impl Iterator<Item = ElementState>;
	fn latest_state(&self, key: Self::KeyType) -> Option<ElementState>;

	fn pressed_keys(&self) -> impl Iterator<Item = Self::KeyType>;
	fn released_keys(&self) -> impl Iterator<Item = Self::KeyType>;
	fn interacted_keys(&self) -> impl Iterator<Item = Self::KeyType>;
}

impl ProcessedInputEvents for ProcessedEventReader<KeyboardInputEvent> {
	type KeyType = KeyCode;

	fn has_pressed(&self, keycode: KeyCode) -> bool {
		self.events
			.iter()
			.any(|kb| kb.state.is_pressed() && kb.physical_key == PhysicalKey::Code(keycode))
	}

	fn has_released(&self, keycode: KeyCode) -> bool {
		self.events
			.iter()
			.any(|kb| !kb.state.is_pressed() && kb.physical_key == PhysicalKey::Code(keycode))
	}

	fn has_interacted(&self, keycode: KeyCode) -> bool {
		self.events
			.iter()
			.any(|kb| kb.physical_key == PhysicalKey::Code(keycode))
	}

	fn states(&self, keycode: KeyCode) -> impl Iterator<Item = ElementState> {
		self.events
			.iter()
			.filter(move |kb| kb.physical_key == PhysicalKey::Code(keycode))
			.map(|kb| kb.state)
	}

	fn latest_state(&self, keycode: Self::KeyType) -> Option<ElementState> {
		self.events
			.iter()
			.filter(|kb| kb.physical_key == PhysicalKey::Code(keycode))
			.last()
			.map(|kb| kb.state)
	}

	fn pressed_keys(&self) -> impl Iterator<Item = KeyCode> {
		self.events.iter().filter_map(|kb| match (kb.state, kb.physical_key) {
			(ElementState::Pressed, PhysicalKey::Code(key)) => Some(key),
			_ => None,
		})
	}

	fn released_keys(&self) -> impl Iterator<Item = KeyCode> {
		self.events.iter().filter_map(|kb| match (kb.state, kb.physical_key) {
			(ElementState::Released, PhysicalKey::Code(key)) => Some(key),
			_ => None,
		})
	}

	fn interacted_keys(&self) -> impl Iterator<Item = KeyCode> {
		self.events.iter().filter_map(|kb| match kb.physical_key {
			PhysicalKey::Code(key) => Some(key),
			_ => None,
		})
	}
}

impl ProcessedInputEvents for ProcessedEventReader<MouseInputEvent> {
	type KeyType = MouseButton;

	fn has_pressed(&self, button: MouseButton) -> bool {
		self.events.iter().any(|b| b.state.is_pressed() && b.button == button)
	}

	fn has_released(&self, button: MouseButton) -> bool {
		self.events.iter().any(|b| !b.state.is_pressed() && b.button == button)
	}

	fn has_interacted(&self, button: MouseButton) -> bool {
		self.events.iter().any(|b| b.button == button)
	}

	fn states(&self, button: MouseButton) -> impl Iterator<Item = ElementState> {
		self.events.iter().filter(move |b| b.button == button).map(|b| b.state)
	}

	fn latest_state(&self, button: Self::KeyType) -> Option<ElementState> {
		self.events
			.iter()
			.filter(|b| b.button == button)
			.last()
			.map(|b| b.state)
	}

	fn pressed_keys(&self) -> impl Iterator<Item = MouseButton> {
		self.events
			.iter()
			.filter(|b| b.state == ElementState::Pressed)
			.map(|b| b.button)
	}

	fn released_keys(&self) -> impl Iterator<Item = MouseButton> {
		self.events
			.iter()
			.filter(|b| b.state == ElementState::Released)
			.map(|b| b.button)
	}

	fn interacted_keys(&self) -> impl Iterator<Item = MouseButton> {
		self.events.iter().map(|b| b.button)
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub trait ProcessedChangeEvents {
	type ItemType;

	fn latest(&self) -> Option<Self::ItemType>;
}

impl ProcessedChangeEvents for ProcessedEventReader<WindowResizedEvent> {
	type ItemType = ScreenSize;

	fn latest(&self) -> Option<Self::ItemType> {
		self.events.last().map(|w| w.size)
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub trait ProcessedMotionEvents {
	type MotionType;

	fn delta_sum(&self) -> Self::MotionType;
}

impl ProcessedMotionEvents for ProcessedEventReader<MouseMotionEvent> {
	type MotionType = MouseMotionDelta;

	fn delta_sum(&self) -> Self::MotionType {
		self.iter().fold(vec2!(0.0), |acc, m| acc + m.motion_delta)
	}
}
