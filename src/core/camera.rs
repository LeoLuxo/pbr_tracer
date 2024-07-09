use bevy_ecs::{
	event::EventReader,
	query::With,
	schedule::{IntoSystemConfigs, SystemSet},
	system::{Local, Query, Res},
};
use brainrot::{
	bevy::{self, App, Plugin},
	calc_forward_horizontal_vector, calc_right_vector, deg, rad, spd, Angle, Direction, Frustum, Position, Speed,
	SAFE_FRAC_PI_2,
};
use derive_more::{Deref, Display, From};
use winit::{
	event::ElementState,
	keyboard::{KeyCode, PhysicalKey},
};

use super::{
	display::AppWindow,
	event_processing::{EventReaderProcessor, ProcessedInputEvents, ProcessedMotionEvents},
	events::{KeyboardInputEvent, MouseMotionEvent},
	gameloop::{Time, Update},
};
use crate::EntityLabel;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			(process_keyboard, process_mouse, process_sprint, update_camera)
				.in_set(CameraControl)
				.run_if(is_cursor_attached),
		);

		app.world.spawn((
			CameraBundle {
				label: Camera,
				frustum: Frustum {
					y_fov: 45_f32.to_radians(),
					z_near: 0.3,
					z_far: 1000.0,
				},
				position: Default::default(),
				direction: Default::default(),
			},
			CameraControlBundle {
				speed: spd!(5.0),
				sensitivity: spd!(deg!(0.1)),
				controller: Default::default(),
			},
			Sprint {
				starting_speed: spd!(1.0),
				acceleration: spd!(spd!(20.)),
			},
		));
	}
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CameraControl;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(bevy::Component)]
pub struct Camera;
impl EntityLabel for Camera {}

#[derive(bevy::Bundle)]
struct CameraBundle {
	label: Camera,
	position: Position,
	direction: Direction,
	frustum: Frustum,
}

#[derive(bevy::Bundle)]
struct CameraControlBundle {
	controller: CameraController,
	speed: MovementSpeed,
	sensitivity: Sensitivity,
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(bevy::Component, Deref, From, Display, Copy, Clone, Debug, Default, PartialEq)]
pub struct MovementSpeed(pub Speed);

#[derive(bevy::Component, Deref, From, Display, Copy, Clone, Debug, Default, PartialEq)]
pub struct Sensitivity(pub Speed<Angle>);

#[derive(bevy::Component, Copy, Clone, Debug, Default, PartialEq)]
pub struct Sprint {
	pub starting_speed: Speed,
	pub acceleration: Speed<Speed>,
}

#[derive(bevy::Component, Copy, Clone, Debug, Default, PartialEq)]
pub struct CameraController {
	moving_left: bool,
	moving_right: bool,
	moving_forward: bool,
	moving_backward: bool,
	moving_up: bool,
	moving_down: bool,

	direction_yaw_accu: f32,
	direction_pitch_accu: f32,
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

fn is_cursor_attached(app_window: Res<AppWindow>) -> bool {
	app_window.cursor_attached
}

fn process_keyboard(
	mut q: Query<&mut CameraController, With<Camera>>,
	mut keyboard_events: EventReader<KeyboardInputEvent>,
) {
	let mut controller = q.single_mut();

	for KeyboardInputEvent {
		state, physical_key, ..
	} in keyboard_events.read()
	{
		let pressed = *state == ElementState::Pressed;

		if let PhysicalKey::Code(key) = physical_key {
			match key {
				KeyCode::KeyW | KeyCode::ArrowUp => {
					controller.moving_forward = pressed;
				}
				KeyCode::KeyS | KeyCode::ArrowDown => {
					controller.moving_backward = pressed;
				}
				KeyCode::KeyA | KeyCode::ArrowLeft => {
					controller.moving_left = pressed;
				}
				KeyCode::KeyD | KeyCode::ArrowRight => {
					controller.moving_right = pressed;
				}
				KeyCode::Space => {
					controller.moving_up = pressed;
				}
				KeyCode::ControlLeft => {
					controller.moving_down = pressed;
				}
				_ => {}
			};
		}
	}
}

fn process_mouse(mut q: Query<&mut CameraController, With<Camera>>, mouse_events: EventReader<MouseMotionEvent>) {
	let mut controller = q.single_mut();
	let motion_delta = mouse_events.process().delta_sum();

	controller.direction_yaw_accu += motion_delta.x as f32;
	controller.direction_pitch_accu += motion_delta.y as f32;
}

fn process_sprint(
	mut q: Query<(&mut MovementSpeed, &mut Sprint), With<Camera>>,
	keyboard_events: EventReader<KeyboardInputEvent>,
	mut normal_speed_backup: Local<Option<Speed>>,
	time: Res<Time>,
) {
	// Turns out winit events are not very reliable, and multiple Pressed events might be fired successively (and vice-versa for Released)

	let (mut speed, sprint) = q.single_mut();

	for state in keyboard_events.process().states(KeyCode::ShiftLeft) {
		match state {
			ElementState::Pressed => {
				if normal_speed_backup.is_none() {
					*normal_speed_backup = Some(speed.0);
					speed.0 = sprint.starting_speed;
				}
			}
			ElementState::Released => {
				if let Some(old_speed) = (*normal_speed_backup).take() {
					speed.0 = old_speed;
				}
			}
		}
	}

	// Speed is being controlled by sprint, so accelerate it
	if normal_speed_backup.is_some() {
		speed.0 += sprint.acceleration * time.dt_u;
	}
}

fn update_camera(
	mut q: Query<
		(
			&mut CameraController,
			&mut Position,
			&mut Direction,
			&MovementSpeed,
			&Sensitivity,
		),
		With<Camera>,
	>,
	time: Res<Time>,
) {
	let (mut controller, mut position, mut direction, movement_speed, sensitivity) = q.single_mut();

	// Move forward/backward and left/right
	let forward = calc_forward_horizontal_vector(*direction);
	let right = calc_right_vector(*direction);

	let movement = movement_speed.0 * time.dt_u;

	if controller.moving_forward {
		position.0 += forward * movement;
	}
	if controller.moving_backward {
		position.0 -= forward * movement;
	}
	if controller.moving_right {
		position.0 += right * movement;
	}
	if controller.moving_left {
		position.0 -= right * movement;
	}
	if controller.moving_up {
		position.0.y += movement;
	}
	if controller.moving_down {
		position.0.y -= movement;
	}

	// Rotate
	// Need to divide by dt_u since the accumulators can be updated multiple times per tick
	// Looks stupid but I swear semantically it makes sense (I hope, I've tried everything to fix this shit)
	direction.yaw += sensitivity.0 * time.dt_u * controller.direction_yaw_accu / time.dt_u.as_secs_f32();
	direction.pitch -= sensitivity.0 * time.dt_u * controller.direction_pitch_accu / time.dt_u.as_secs_f32();

	controller.direction_yaw_accu = 0.0;
	controller.direction_pitch_accu = 0.0;

	// Keep the camera's angle from going too high/low.
	direction.pitch.clamp(rad!(-SAFE_FRAC_PI_2), rad!(SAFE_FRAC_PI_2));
}
