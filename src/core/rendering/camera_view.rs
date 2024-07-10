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
	Direction, Frustum, Position,
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
	pub z_near: f32,
	pub z_far: f32,

	pub y_fov: f32,
	pub focal_length: f32,

	pub view_mat: Mat4<f32>,
	pub inverse_view_mat: Mat4<f32>,
	pub proj_mat: Mat4<f32>,
}

fn update_view(
	render_target: Res<RenderTarget<'static>>,
	mut q: Query<(&Position, &Direction, &Frustum, &mut CameraView)>,
) {
	for (position, direction, frustum, mut view) in q.iter_mut() {
		let position = *position;
		let direction = *direction;
		let size = render_target.size;
		let z_near = frustum.z_near;
		let z_far = frustum.z_far;
		let y_fov = frustum.y_fov;

		let focal_length = (size.h as f32) / 2.0 / (y_fov / 2.0).tan();
		let view_mat = calc_view_matrix(position, direction);
		let inverse_view_mat = calc_view_matrix(position, direction).inverted();
		let proj_mat = calc_projection_matrix(*frustum, size);

		*view = CameraView {
			z_near,
			z_far,
			y_fov,
			focal_length,
			view_mat,
			inverse_view_mat,
			proj_mat,
		}
	}
}
