use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_ecs::system::{Res, ResMut};
use brainrot::bevy::{App, Plugin};
use brainrot::engine_3d::TextureAsset;
use brainrot::{bevy, src};
use wgpu::{
	include_wgsl, BlendState, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, CompareFunction,
	DepthBiasState, DepthStencilState, FragmentState, FrontFace, LoadOp, MultisampleState, Operations,
	PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPassColorAttachment,
	RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, StencilState,
	StoreOp, VertexState,
};

use crate::core::display::Gpu;
use crate::core::gameloop::Render;

use super::render_target::RenderTarget;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct BasicRendererPlugin;

impl Plugin for BasicRendererPlugin {
	fn build(&self, app: &mut App) {
		let gpu = app.world.resource::<Gpu>();
		let render_target = app.world.resource::<RenderTarget>();

		let basic_renderer = BasicRenderer::new(gpu, render_target);

		app.world.insert_resource(basic_renderer);

		app.add_systems(Render, (render).in_set(BasicRenderPass).chain());
	}
}

#[derive(bevy::SystemSet, Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct BasicRenderPass;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(bevy::Resource)]
pub struct BasicRenderer {
	render_pipeline: RenderPipeline,
}

impl BasicRenderer {
	pub fn new(gpu: &Gpu, render_target: &RenderTarget) -> Self {
		// Statically include the shader in the executable
		let shader = gpu
			.device
			.create_shader_module(include_wgsl!(src!("shader/compose.wgsl")));

		// Contains the bind group layouts that are needed in the pipeline
		let render_pipeline_layout = gpu.device.create_pipeline_layout(&PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[],
			push_constant_ranges: &[],
		});

		// Create the render pipeline. Specify shader stages, primitive type, stencil/depth information, and some more stuff.
		let render_pipeline = {
			gpu.device.create_render_pipeline(&RenderPipelineDescriptor {
				label: Some("Basic Render Pipeline"),
				layout: Some(&render_pipeline_layout),
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
					front_face: FrontFace::Cw,
					cull_mode: None,
					polygon_mode: PolygonMode::Fill,
					unclipped_depth: false,
					conservative: true,
				},
				// Don't worry about the depth buffer for now
				depth_stencil: Some(DepthStencilState {
					format: TextureAsset::DEPTH_FORMAT,
					depth_write_enabled: false,
					depth_compare: CompareFunction::Always,
					stencil: StencilState::default(),
					bias: DepthBiasState::default(),
				}),
				multisample: MultisampleState {
					count: 1,
					mask: !0,
					alpha_to_coverage_enabled: false,
				},
				multiview: None,
			})
		};

		Self { render_pipeline }
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

fn render(basic_renderer: Res<BasicRenderer>, mut render_target: ResMut<RenderTarget<'static>>, gpu: Res<Gpu>) {
	// trace!("Rendering terrain");

	// A command encoder takes multiple draw/compute commands that can then be encoded into a command buffer to be submitted to the queue
	let mut encoder = gpu.device.create_command_encoder(&CommandEncoderDescriptor {
		label: Some("BasicRenderer Command Encoder"),
	});

	{
		let render_view = &render_target
			.current_view
			.as_ref()
			.expect("Attempt to encode renderpass while RenderTarget view is unavailable");

		// A render pass records a single pass of a pipeline
		let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
			label: Some("Render Pass"),
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
			depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
				view: &render_target.depth_texture.view,
				depth_ops: Some(Operations {
					load: LoadOp::Clear(1.0),
					store: StoreOp::Store,
				}),
				stencil_ops: None,
			}),
			occlusion_query_set: None,
			timestamp_writes: None,
		});

		render_pass.set_pipeline(&basic_renderer.render_pipeline);

		// Draw 2 fullscreen triangles
		// 1 -- 2
		// | /  |
		// 3 -- 4
		render_pass.draw(0..4, 0..1);
	}
	// Extra scope here to make sure render_pass is dropped, otherwise encoder.finish() can't be called

	render_target.command_queue.push(encoder.finish());
}
