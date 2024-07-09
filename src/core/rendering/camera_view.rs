use bevy_ecs::{
	entity::Entity,
	query::With,
	schedule::IntoSystemConfigs,
	system::{Query, Res},
};
use brainrot::{
	bevy::{self, App, Plugin},
	calc_projection_matrix, calc_view_matrix,
	vek::Mat4,
	Direction, Frustum, Position, ScreenSize,
};
use pbr_tracer_derive::ShaderStruct;

use crate::{
	core::{
		camera::{Camera, CameraControl},
		gameloop::Update,
		gpu::Gpu,
		render_target::RenderTarget,
	},
	libs::{
		buffer::{self, uniform_buffer::UniformBuffer, ShaderType},
		smart_arc::Sarc,
	},
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct CameraViewPlugin;

impl Plugin for CameraViewPlugin {
	fn build(&self, app: &mut App) {
		let gpu = app.world.resource::<Gpu>();

		let camera_view_buffer = Sarc::new(UniformBuffer::raw_buffer_from_type::<CameraView>(gpu, None));

		let camera_entity = app
			.world
			.query_filtered::<Entity, With<Camera>>()
			.single_mut(&mut app.world);

		app.world
			.entity_mut(camera_entity)
			.insert(CameraView::default())
			.insert(camera_view_buffer);

		buffer::register_auto_update::<CameraView>(app);

		app.add_systems(Update, (update_view).after(CameraControl));
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[repr(C)]
#[derive(ShaderStruct, bytemuck::Pod, bytemuck::Zeroable, bevy::Component, Copy, Clone, Debug, Default, PartialEq)]
pub struct CameraView {
	// vek::Mat4 supports bytemuck
	pub view_mat: Mat4<f32>,
	pub inverse_view_mat: Mat4<f32>,
	pub proj_mat: Mat4<f32>,
}

impl CameraView {
	pub fn new(position: Position, direction: Direction, frustum: Frustum, size: ScreenSize) -> Self {
		Self {
			view_mat: calc_view_matrix(position, direction),
			inverse_view_mat: calc_view_matrix(position, direction).inverted(),
			proj_mat: calc_projection_matrix(frustum, size),
		}
	}
}

fn update_view(
	render_target: Res<RenderTarget<'static>>,
	mut q: Query<(&Position, &Direction, &Frustum, &mut CameraView), With<Camera>>,
) {
	for (position, direction, frustum, mut view) in q.iter_mut() {
		*view = CameraView::new(*position, *direction, *frustum, render_target.size);
	}
}
