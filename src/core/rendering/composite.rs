use bevy_ecs::{
	event::EventReader,
	schedule::IntoSystemConfigs,
	system::{Query, Res, ResMut},
};
use brainrot::{
	bevy::{self, App, Plugin},
	vek::Vec2,
	ScreenSize,
};
use pbr_tracer_derive::ShaderStruct;
use velcro::vec;
use wgpu::{
	BlendState, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, FragmentState, FrontFace, LoadOp,
	MultisampleState, Operations, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
	RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStages, StoreOp,
	VertexState,
};

use super::compute::ComputeRenderer;
use crate::{
	core::{
		event_processing::{EventReaderProcessor, ProcessedChangeEvents},
		events::WindowResizedEvent,
		gameloop::{Render, Update},
		gpu::Gpu,
		render_target::RenderTarget,
	},
	libs::{
		buffer::{
			self, texture_sampler_buffer::TextureSamplerBuffer, uniform_buffer::UniformBuffer, BufferMappingApplicable,
			GenericDataBuffer, ShaderType,
		},
		shader::{CompiledShader, ShaderBuilder},
	},
	ShaderAssets,
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct CompositeRendererPlugin;

impl Plugin for CompositeRendererPlugin {
	fn build(&self, app: &mut App) {
		let gpu = app.world.resource::<Gpu>();
		let render_target = app.world.resource::<RenderTarget>();
		let computer_renderer = app.world.resource::<ComputeRenderer>();

		let viewport_info = ViewportInfo {
			size: render_target.size,
		};

		let viewport_buffer = GenericDataBuffer::new(
			gpu,
			ShaderStages::FRAGMENT,
			&UniformBuffer::from_data("viewport_size", viewport_info),
		);

		let composite_renderer = CompositeRenderer::new(gpu, render_target, computer_renderer, viewport_buffer.clone());

		buffer::register_uniform_auto_update::<ViewportInfo>(app);
		app.world.spawn((viewport_info, viewport_buffer));

		app.world.insert_resource(composite_renderer);

		app.add_systems(Update, resize);
		app.add_systems(Render, (render).in_set(CompositeRenderPass).chain());
	}
}

#[derive(bevy::SystemSet, Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct CompositeRenderPass;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[repr(C)]
#[derive(ShaderStruct, bevy::Component, bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
pub struct ViewportInfo {
	pub size: ScreenSize,
}

#[derive(bevy::Resource)]
pub struct CompositeRenderer {
	pipeline: RenderPipeline,
	shader: CompiledShader,
}

impl CompositeRenderer {
	pub fn new(
		gpu: &Gpu,
		render_target: &RenderTarget,
		compute_renderer: &ComputeRenderer,
		viewport_buffer: GenericDataBuffer,
	) -> Self {
		let shader = ShaderBuilder::new()
			.include_path("composite.wgsl")
			.include_texture(TextureSamplerBuffer::new(
				"out_texture",
				"out_sampler",
				buffer::texture_sampler_buffer::TextureBufferBacking::From(compute_renderer.output_texture.clone()),
			))
			.include_data_buffer(UniformBuffer::<Vec2<u32>>::from_buffer(
				"viewport_size",
				viewport_buffer.buffer,
			))
			.build(gpu, &ShaderAssets, ShaderStages::FRAGMENT, 0)
			.expect("Couldn't build shader");

		// Contains the bind group layouts that are needed in the pipeline
		let render_pipeline_layout = gpu.device.create_pipeline_layout(&PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &shader.buffers.layouts(),
			push_constant_ranges: &[],
		});

		// Create the render pipeline. Specify shader stages, primitive type,
		// stencil/depth information, and some more stuff.
		let pipeline = gpu.device.create_render_pipeline(&RenderPipelineDescriptor {
			label: Some("Basic Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			// No vertex buffers, we'll render 2 fullscreen triangles
			// and set their positions in the shader
			vertex: VertexState {
				module: &shader.shader_module,
				entry_point: "vs_main",
				buffers: &[],
			},
			fragment: Some(FragmentState {
				module: &shader.shader_module,
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

		Self { pipeline, shader }
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

fn resize(window_events: EventReader<WindowResizedEvent>, mut q: Query<&mut ViewportInfo>) {
	if let Some(size) = window_events.process().latest() {
		for mut viewport_info in q.iter_mut() {
			viewport_info.size = size;
		}
	}
}

fn render(composite_renderer: Res<CompositeRenderer>, mut render_target: ResMut<RenderTarget<'static>>, gpu: Res<Gpu>) {
	// trace!("Rendering terrain");

	// A command encoder takes multiple draw/compute commands that can then be
	// encoded into a command buffer to be submitted to the queue
	let mut encoder = gpu.device.create_command_encoder(&CommandEncoderDescriptor {
		label: Some("CompositeRenderer Command Encoder"),
	});

	{
		let render_view = &render_target
			.current_view
			.as_ref()
			.expect("Attempt to encode renderpass while RenderTarget view is unavailable");

		// A render pass records a single pass of a pipeline
		let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
			label: Some("CompositeRenderer Render Pass"),
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

		render_pass.set_pipeline(&composite_renderer.pipeline);

		render_pass.apply_buffer_mapping(&composite_renderer.shader.buffers);

		// Draw 2 fullscreen triangles
		// 2 - 3
		// | \ |
		// 0 - 1
		render_pass.draw(0..4, 0..1);
	}
	// Extra scope here to make sure render_pass is dropped, otherwise
	// encoder.finish() can't be called

	render_target.command_queue.push(encoder.finish());
}
