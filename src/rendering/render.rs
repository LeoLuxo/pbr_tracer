use bevy_ecs::{
	schedule::{IntoSystemConfigs, IntoSystemSetConfigs},
	system::{Res, ResMut},
};
use brainrot::bevy::{self, App, Plugin};
use wgpu::TextureViewDescriptor;

use crate::core::{display::Gpu, gameloop::Render, render_target::RenderTarget};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
	fn build(&self, app: &mut App) {
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
