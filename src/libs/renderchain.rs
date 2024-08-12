pub mod compute_step;

use anyhow::Result;
use bevy_ecs::{
	entity::Entity,
	query::{QueryData, QueryFilter, ROQueryItem, With},
	schedule::IntoSystemConfigs,
	system::{Commands, Query, Res, ResMut},
	world::World,
};
use brainrot::{
	bevy::{self, App, Plugin},
	vec2,
	vek::Vec2,
	ScreenSize,
};
use wgpu::{
	Buffer, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, FilterMode,
	SamplerBorderColor, ShaderStages, StorageTextureAccess, TextureViewDescriptor,
};

use super::{
	buffer::{ShaderBufferDescriptor, ShaderBufferResource},
	shader::Shader,
	shader_fragment::ShaderFragment,
};
use crate::{
	core::{
		camera::Camera,
		gameloop::{PreRender, Render},
		gpu::Gpu,
		render_target::RenderTarget,
	},
	libs::{
		buffer::{
			storage_texture_buffer::StorageTexture, uniform_buffer::UniformBufferDescriptor, BufferMappingApplicable,
		},
		shader::{CompiledShader, ShaderBuilder},
		shader_fragment::Renderer,
		smart_arc::Sarc,
		texture::{SamplerEdges, Tex, TexSamplerDescriptor},
	},
	ShaderAssets,
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(bevy::Component)]
pub struct RenderChain {
	pub buffers: Vec<&'static (dyn ShaderBufferDescriptor + Sync + Send)>,
	pub steps: Vec<&'static (dyn RenderStep + Sync + Send)>,
}

#[derive(bevy::Component)]
pub struct CompiledRenderChain {
	buffers: Vec<Box<dyn ShaderBufferResource + Sync + Send>>,
	steps: Vec<Box<dyn CompiledRenderStep + Sync + Send>>,
}

pub trait RenderStep {
	fn compile(&self, gpu: &Gpu, extras: &mut Extras) -> Box<dyn CompiledRenderStep + Sync + Send>;
}

pub trait CompiledRenderStep {
	fn render(&self);
}

impl RenderChain {
	pub fn compile(self, gpu: &Gpu) -> CompiledRenderChain {
		let buffers = self.buffers.into_iter().map(|b| b.as_resource(gpu)).collect();
		let steps = self.steps.into_iter().map(|s| s.compile(gpu, extras));

		CompiledRenderChain { buffers, steps }
	}
}

impl CompiledRenderChain {
	// TODO: add support for renderchains without a rendertarget (where the compute buffer gets copied back to RAM)
	pub fn render(&self, gpu: &Gpu, target: &RenderTarget) -> Result<()> {
		// Get the output texture to render to and create a view for it.
		let output = target.surface.get_current_texture()?;
		let view = output.texture.create_view(&TextureViewDescriptor::default());

		for step in self.steps {
			todo!()
		}

		// Submit the encoded command buffer to the queue
		// And clear queue at the same time
		gpu.queue.submit(target.command_queue.drain(..));

		// Swap the draw buffers and show what we rendered to the screen
		Ok(output.present())
	}
}
/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

struct RenderChainPlugin;

impl Plugin for RenderChainPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(PreRender, render_targets_and_chains);
		app.add_systems(Render, render_targets_and_chains);
	}
}

fn compile_render_chains(render_chains: Query<Entity, With<RenderChain>>, mut commands: Commands) {
	for entity_id in render_chains.iter() {
		commands.add(move |world: &mut World| {
			let gpu = world.resource::<Gpu>();

			// Can't cache `let entity = world.entity_mut(..)` because of borrow-checker reasons
			let chain = world.entity_mut(entity_id).take::<RenderChain>().unwrap();
			let compiled_chain = chain.compile(gpu);
			world.entity_mut(entity_id).insert(compiled_chain);
		});
	}
}

fn render_targets_and_chains(
	gpu: Res<Gpu>,
	render_targets: Query<&RenderTarget>,
	render_chains: Query<&CompiledRenderChain>,
) {
	for target in render_targets.iter() {
		for chain in render_chains.iter() {
			chain.render(&gpu, target);
		}
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct Extras<'w> {
	world: &'w mut World,
}

impl<'w> Extras<'w> {
	pub fn get<D: QueryData, F: QueryFilter>(&'w mut self) -> ROQueryItem<'w, D> {
		self.world.query_filtered::<D, F>().single(self.world)
	}
}
