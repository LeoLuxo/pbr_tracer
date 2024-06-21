use std::{
	cmp::min,
	time::{Duration, Instant},
};

use bevy_ecs::{schedule::ScheduleLabel, world::World};
use brainrot::{
	bevy::{self, App, Plugin, PluginsState},
	Converter,
};
use log::trace;
use winit::event::{DeviceEvent, Event, KeyEvent, WindowEvent};

use crate::{
	core::{
		display::AppWindow,
		events::{
			KeyboardInputEvent, MouseInputEvent, MouseMotionEvent, MouseWheelEvent, WindowResizedEvent,
			WinitWindowEvent,
		},
	},
	EventLoop,
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct GameloopPlugin;

impl Plugin for GameloopPlugin {
	fn build(&self, app: &mut App) {
		let time = Time {
			target_ups: 60,
			target_fps: None,

			start_time: Instant::now(),
			last_iteration_time: Instant::now(),
			last_update_time: Instant::now(),
			last_render_time: Instant::now(),

			..Default::default()
		};

		app.world.insert_resource(time);

		app.set_runner(run);
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

/// A schedule that runs at every iteration of the event loop, but before
/// [`IterStep`]. It is meant to be used in by the core of the app, to keep
/// track of when the event queues should be cleared.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EventsCore;

/// The schedule that runs at every iteration of the event loop.
/// (To be more precise every time a redraw is requested, which might be several
/// thousand times per second)
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct IterStep;

/// The schedule that runs at a fixed timestep, meant for game logic, physic
/// updates, etc
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Update;

/// The schedule that runs at a semi-fixed timestep, meant for things that
/// should happen right before rendering
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PreRender;

/// The schedule that runs at a semi-fixed timestep, meant for rendering
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Render;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(bevy::Resource, Debug, Copy, Clone)]
pub struct Time {
	start_time: Instant,
	pub current_time: Duration,

	pub target_ups: u32,
	pub target_fps: Option<u32>,

	last_iteration_time: Instant,
	last_update_time: Instant,
	last_render_time: Instant,

	pub dt_u: Duration,
	pub dt_f: Duration,

	update_accumulator: Duration,
	render_accumulator: Duration,

	pub counter_update: u64,
	pub counter_frame: u64,

	pub ups: f32,
	pub fps: f32,
	pub smooth_ups: f32,
	pub smooth_fps: f32,
}

impl Time {
	const SMOOTH_RESPONSIVENESS: f32 = 0.05;

	pub fn smooth(&self, smoothed: &mut f32, raw: f32) {
		*smoothed = self.smoothed(*smoothed, raw);
	}

	pub fn smoothed(&self, smoothed: f32, raw: f32) -> f32 {
		let response = Self::SMOOTH_RESPONSIVENESS * self.target_ups as f32 * self.dt_u.as_secs_f32();
		(1.0 - response) * smoothed + response * raw
	}
}

impl Default for Time {
	fn default() -> Self {
		Self {
			start_time: Instant::now(),
			current_time: Default::default(),
			target_ups: Default::default(),
			target_fps: Default::default(),
			last_iteration_time: Instant::now(),
			last_update_time: Instant::now(),
			last_render_time: Instant::now(),
			dt_u: Default::default(),
			dt_f: Default::default(),
			update_accumulator: Default::default(),
			render_accumulator: Default::default(),
			counter_update: Default::default(),
			counter_frame: Default::default(),
			ups: Default::default(),
			fps: Default::default(),
			smooth_ups: Default::default(),
			smooth_fps: Default::default(),
		}
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub fn run(mut app: App) {
	wait_for_plugins(&mut app);
	start_event_loop(&mut app.world);
}

fn wait_for_plugins(app: &mut App) {
	// Wait for all plugins to be ready
	while app.plugins_state() != PluginsState::Ready {
		// Yield to the OS scheduler, because we have to wait anyway
		std::thread::yield_now();
	}

	// Finish plugin building process and cleanup
	app.finish();
	app.cleanup();
}

fn start_event_loop(world: &mut World) {
	trace!("Starting event loop");

	let event_loop = world
		.remove_non_send_resource::<EventLoop>()
		.expect("Tried starting the gameloop without a winit eventloop available");

	let _ = event_loop.run(move |event, target| match event {
		Event::DeviceEvent { event, .. } => match event {
			DeviceEvent::MouseMotion { delta } => {
				let event_out = MouseMotionEvent {
					motion_delta: delta.into(),
				};
				// trace!("Winit event: Event::DeviceEvent::MouseMotion");
				// trace!("Event out: {event_out:#?}");
				world.send_event(event_out);
			}
			DeviceEvent::MouseWheel { delta } => {
				let event_out = MouseWheelEvent { wheel_delta: delta };
				trace!("Winit event: Event::DeviceEvent::MouseWheel");
				trace!("Event out: {event_out:#?}");
				world.send_event(event_out);
			}
			_ => {}
		},

		Event::AboutToWait => {
			// trace!("Winit event: Event::AboutToWait");
		}

		Event::WindowEvent { event, .. } => {
			world.send_event(WinitWindowEvent(event.clone()));

			match event {
				WindowEvent::CloseRequested => {
					trace!("Winit event: Event::WindowEvent::CloseRequested");
					target.exit();
				}

				WindowEvent::KeyboardInput {
					event: KeyEvent {
						state,
						logical_key,
						physical_key,
						..
					},
					..
				} => {
					let event_out = KeyboardInputEvent {
						state,
						logical_key,
						physical_key,
					};
					trace!("Winit event: Event::WindowEvent::KeyboardInput");
					trace!("Event out: {event_out:#?}");
					world.send_event(event_out);
				}

				WindowEvent::MouseInput { state, button, .. } => {
					let event_out = MouseInputEvent { state, button };
					trace!("Winit event: Event::WindowEvent::MouseInput");
					trace!("Event out: {event_out:#?}");
					world.send_event(event_out);
				}

				WindowEvent::Resized(physical_size) if physical_size.width > 0 && physical_size.height > 0 => {
					let event_out = WindowResizedEvent {
						size: physical_size.convert(),
					};
					trace!("Winit event: Event::WindowEvent::Resized");
					trace!("Event out: {event_out:#?}");
					world.send_event(event_out);
				}

				WindowEvent::RedrawRequested => {
					// trace!("Winit event: Event::WindowEvent::RedrawRequested");
					schedule_game_iteration(world);
					world.resource::<AppWindow>().winit_window.request_redraw();
				}

				_ => {}
			}
		}
		_ => {}
	});
}

fn schedule_game_iteration(world: &mut World) {
	// Inspired by https://gafferongames.com/post/fix_your_timestep/

	// Call the fast-looping schedules at the beginning, so they don't delay
	// delta_iteration in case they take longer than they should
	let _ = world.try_run_schedule(EventsCore);
	let _ = world.try_run_schedule(IterStep);

	// Due to mut borrows clashing with time when running schedules, I clone here
	// and then re-insert time before running schedules
	let mut time = *world.resource::<Time>();
	let now = Instant::now();

	let delta_iteration = now - time.last_iteration_time;

	// Update deltaTimes and update the accumulators that update() and render() can
	// consume from dt_f is either "constant" or basically equal to delta_iteration
	if let Some(target_fps) = time.target_fps {
		time.dt_f = Duration::from_secs_f32(1. / target_fps as f32);
		time.render_accumulator += delta_iteration;
	} else {
		time.dt_f = now - time.last_render_time;
	}

	// dt_u is "constant"
	time.dt_u = Duration::from_secs_f32(1. / time.target_ups as f32);
	time.update_accumulator += delta_iteration;

	// Run update systems
	let num_updates = time.update_accumulator.as_nanos() / time.dt_u.as_nanos();
	for _ in 0..num_updates {
		world.insert_resource(time);
		let _ = world.try_run_schedule(Update);

		// Update current time by one step so that the update systems see it correctly
		time.current_time += time.dt_u;

		time.update_accumulator -= time.dt_u;
		time.counter_update += 1;

		// TODO Somehow handle the case where update takes consistently longer
		// than 16ms to run
	}

	if num_updates >= 1 {
		// Update UPS info; technically the UPS should be equal to target_ups, but since
		// that's useless info, predict the number of updates for the next second
		// (very jittery), and let smooth_ups do its thing
		time.ups = 1. / (now - time.last_update_time).as_secs_f32() * num_updates as f32;
		time.smooth_ups = time.smoothed(time.smooth_ups, time.ups);

		time.last_update_time = now;
	}

	// Update current_time to the true value
	time.current_time = now - time.start_time;

	// TODO Should interpolate here
	// Why and how to do interpolation despite the potential 1-frame delay:
	// https://gamedev.stackexchange.com/questions/187660/fixed-timestep-game-loop-why-interpolation
	// https://gamedev.stackexchange.com/questions/147908/using-an-interpolated-game-loop-such-as-gaffers-final-game-loop-will-the-fra/147913#147913

	// Run render systems
	let mut should_render = false;
	if time.target_fps.is_some() {
		// If there is a certain target fps, then check that the accumulator has
		// produced enough time that render() can consume
		if time.render_accumulator >= time.dt_f {
			should_render = true;

			time.render_accumulator -= time.dt_f;

			// In case FPS is consistently under the target, make sure that
			// render_accumulator doesn't accumulate too much (which would compensate when
			// FPS start raising again)
			time.render_accumulator = min(time.render_accumulator, time.dt_f * 2);
		}
	} else {
		// Otherwise, just render as often as possible
		should_render = true;
	}

	if should_render {
		world.insert_resource(time);
		let _ = world.try_run_schedule(PreRender);
		let _ = world.try_run_schedule(Render);

		// Update FPS info; above comment about UPS also applies here
		time.fps = 1. / (now - time.last_render_time).as_secs_f32();
		time.smooth_fps = time.smoothed(time.smooth_fps, time.fps);

		time.last_render_time = now;
		time.counter_frame += 1;
	}

	// Finish up
	time.last_iteration_time = now;
	world.insert_resource(time);
}
