use std::slice::Iter;

use bevy_ecs::event::{Event, EventReader};
use brainrot::{vec2, MouseMotionDelta, ScreenSize};
use event::{KeyboardInputEvent, MouseInputEvent, MouseMotionEvent, WindowResizedEvent};
use winit::{
	event::{ElementState, MouseButton},
	keyboard::{KeyCode, PhysicalKey},
};

use crate::core::event;

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
	// Taking ownership of self on purpose here; since process() will read() and thus consume the iterator behind EventReader,
	// making sure I can't use the EventReader anymore afterwards should hopefully prevent some dumb bugs
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
