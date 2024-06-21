use brainrot::bevy::{self, App, Plugin};
use wgpu::{
	Adapter, Backends, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, InstanceFlags, Limits,
	PowerPreference, Queue, RequestAdapterOptions, Surface,
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct GpuPlugin;

impl Plugin for GpuPlugin {
	fn build(&self, app: &mut App) {
		let gpu = pollster::block_on(Gpu::new(None));
		app.world.insert_resource(gpu);
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(bevy::Resource)]
pub struct Gpu {
	pub instance: Instance,
	pub adapter: Adapter,
	pub device: Device,
	pub queue: Queue,
}

impl Gpu {
	async fn new(compatible_surface: Option<&Surface<'_>>) -> Self {
		// Instance is the instance of wgpu which serves as entrypoint for everything
		// wgpu-related
		#[cfg(debug_assertions)]
		// Not running in --release mode, activate validation and debug info for wgpu
		let instance = Instance::new(InstanceDescriptor {
			backends: Backends::PRIMARY,
			flags: InstanceFlags::VALIDATION | InstanceFlags::DEBUG,
			..Default::default()
		});

		#[cfg(not(debug_assertions))]
		// Running in --release mode, don't activate debugging infos for wgpu
		let instance = Instance::new(InstanceDescriptor {
			backends: Backends::PRIMARY,
			..Default::default()
		});

		// Adapter essentially represents the physical GPU + the Backend, e.g.
		// GTX1080_VK; GTX1080_DX12; etc
		let adapter = instance
			.request_adapter(&RequestAdapterOptions {
				power_preference: PowerPreference::HighPerformance,
				compatible_surface,
				force_fallback_adapter: false,
			})
			.await
			.expect("Coudln't request compatible adapter");

		// Device esentially acts like a logical connection to the selected adapter in
		// an application-isolated way. The device is selected based on a descriptor
		// that describes the required features. Queue is the message queue / command
		// buffer for the GPU, anything that the GPU needs to do should be requested
		// into that queue (i.e. rendering, uploading buffer data, etc)
		let (device, queue) = adapter
			.request_device(
				&(DeviceDescriptor {
					required_features: Features::empty()
						// | Features::TEXTURE_BINDING_ARRAY
						// | Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
						| Features::CONSERVATIVE_RASTERIZATION
						| Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
						| Features::FLOAT32_FILTERABLE,
					required_limits: Limits::default(),
					label: None,
				}),
				None,
			)
			.await
			.expect("Couldn't request device");

		Self {
			instance,
			adapter,
			device,
			queue,
		}
	}
}
