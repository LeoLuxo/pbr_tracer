use bevy_ecs::{
	schedule::IntoSystemConfigs,
	system::{Res, ResMut},
};
use brainrot::{
	bevy,
	bevy::{App, Plugin},
	src,
};
use wgpu::{
	include_wgsl, BlendState, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, FragmentState, FrontFace,
	LoadOp, MultisampleState, Operations, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPassColorAttachment,
	RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, StoreOp, VertexState,
};

use crate::core::{display::Gpu, gameloop::Render, render_target::RenderTarget};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct ComposeRendererPlugin;

impl Plugin for ComposeRendererPlugin {
	fn build(&self, app: &mut App) {
		let gpu = app.world.resource::<Gpu>();
		let render_target = app.world.resource::<RenderTarget>();

		let compose_renderer = ComposeRenderer::new(gpu, render_target);

		app.world.insert_resource(compose_renderer);

		app.add_systems(Render, (render).in_set(ComposeRenderPass).chain());
	}
}

#[derive(bevy::SystemSet, Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ComposeRenderPass;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(bevy::Resource)]
pub struct ComposeRenderer {
	pipeline: RenderPipeline,
}

impl ComposeRenderer {
	pub fn new(gpu: &Gpu, render_target: &RenderTarget) -> Self {
		// Statically include the shader in the executable
		let shader = gpu
			.device
			.create_shader_module(include_wgsl!(src!("shader/compose.wgsl")));

		// Create the render pipeline. Specify shader stages, primitive type, stencil/depth information, and some more stuff.
		let pipeline = gpu.device.create_render_pipeline(&RenderPipelineDescriptor {
			label: Some("Basic Render Pipeline"),
			layout: None,
			// No vertex buffers, we'll render 2 fullscreen triangles
			// and set their positions in the shader
			vertex: VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[],
			},
			fragment: Some(FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[Some(ColorTargetState {
					format: render_target.config.format,
					blend: Some(BlendState::REPLACE),
					write_mask: ColorWrites::ALL,
				})],
			}),
			// The point is to draw 2 triangles using 4 vertices.
			// 1 -- 2
			// | /  |
			// 3 -- 4
			primitive: PrimitiveState {
				topology: PrimitiveTopology::TriangleStrip,
				strip_index_format: None,
				front_face: FrontFace::Ccw,
				cull_mode: None,
				polygon_mode: PolygonMode::Fill,
				unclipped_depth: false,
				conservative: true,
			},
			// Don't worry about the depth buffer for now
			depth_stencil: None,
			multisample: MultisampleState {
				count: 1,
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			multiview: None,
		});

		Self { pipeline }
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

fn render(compose_renderer: Res<ComposeRenderer>, mut render_target: ResMut<RenderTarget<'static>>, gpu: Res<Gpu>) {
	// trace!("Rendering terrain");

	// A command encoder takes multiple draw/compute commands that can then be encoded into a command buffer to be submitted to the queue
	let mut encoder = gpu.device.create_command_encoder(&CommandEncoderDescriptor {
		label: Some("ComposeRenderer Command Encoder"),
	});

	{
		let render_view = &render_target
			.current_view
			.as_ref()
			.expect("Attempt to encode renderpass while RenderTarget view is unavailable");

		// A render pass records a single pass of a pipeline
		let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
			label: Some("ComposeRenderer Render Pass"),
			color_attachments: &[Some(RenderPassColorAttachment {
				view: render_view,
				resolve_target: None,
				ops: Operations {
					load: LoadOp::Clear(Color {
						r: 0.0,
						g: 0.0,
						b: 0.0,
						a: 1.0,
					}),
					store: StoreOp::Store,
				},
			})],
			depth_stencil_attachment: None,
			occlusion_query_set: None,
			timestamp_writes: None,
		});

		render_pass.set_pipeline(&compose_renderer.pipeline);

		// Draw 2 fullscreen triangles
		// 1 -- 2
		// | /  |
		// 3 -- 4
		render_pass.draw(0..4, 0..1);
	}
	// Extra scope here to make sure render_pass is dropped, otherwise encoder.finish() can't be called

	render_target.command_queue.push(encoder.finish());
}
