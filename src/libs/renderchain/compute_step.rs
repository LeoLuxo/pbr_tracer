use std::fmt::format;

use anyhow::Result;
use brainrot::vek::Vec2;
use wgpu::{ComputePipelineDescriptor, ShaderStages};

use super::RenderStep;
use crate::{
	core::gpu::Gpu,
	libs::{
		shader::{CompiledShader, Shader, ShaderBuilder},
		shader_fragment::ShaderFragment,
	},
	ShaderAssets,
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct ComputeStep<S>
where
	S: ShaderFragment,
{
	pub label: String,
	pub workgroup_size: Vec2<u32>,
	pub shader: S,
}

impl<S> RenderStep for ComputeStep<S>
where
	S: ShaderFragment,
{
	fn pipeline(&self, gpu: &Gpu, extras: &mut Extras) -> Pipeline {
		let shader = ShaderBuilder::new()
			.include_path("compute.wgsl")
			.define("WORKGROUP_X", format!("{}", self.workgroup_size.x))
			.define("WORKGROUP_Y", format!("{}", self.workgroup_size.y))
			.include(self.shader.shader())
			// .include_buffer(UniformBufferDescriptor::FromBuffer::<CameraView, _> {
			// 	var_name: "camera",
			// 	buffer: camera_buffer,
			// })
			.build(
				gpu,
				&format!("Compute shader '{}'", self.label),
				&ShaderAssets,
				ShaderStages::COMPUTE,
				0,
			)
			.expect(&format!("Couldn't build compute shader '{}'", self.label));

		let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Compute pipeline Layout"),
			bind_group_layouts: &shader.layouts(),
			push_constant_ranges: &[],
		});

		let pipeline = gpu.device.create_compute_pipeline(&ComputePipelineDescriptor {
			label: Some("Compute pipeline"),
			layout: Some(&pipeline_layout),
			module: &shader.shader_module,
			entry_point: "main",
		});
	}
}
