use bevy_tasks::{AsyncComputeTaskPool, TaskPool};
use brainrot::bevy::App;

pub fn run() {
	AsyncComputeTaskPool::get_or_init(TaskPool::new);

	App::new()
		// .add_plugin(GameloopPlugin)
		// .add_plugin(EventPlugin)
		// .add_plugin(DisplayPlugin)p
		// // Rendering plugins
		// .add_plugin(WindowRenderTargetPlugin)
		// .add_plugin(TerrainRendererPlugin)
		// .add_plugin(ChunkRendererPlugin)
		// .add_plugin(DebugRendererPlugin)
		// .add_plugin(DebugGuiPlugin)
		// // Universe plugins
		// .add_plugin(CameraPlugin)
		// .add_plugin(TerrainPlugin)
		// // Configure Renderpass order
		// .configure_sets(
		// 	Render,
		// 	((
		// 		PreRenderPass,
		// 		(
		// 			TerrainRenderPass,
		// 			(PreDebugRenderPass, DebugGuiRenderPass, PostDebugRenderPass)
		// 				.chain()
		// 				.in_set(DebugRenderPass),
		// 		)
		// 			.chain()
		// 			.in_set(InnerRenderPass),
		// 		PostRenderPass,
		// 	)
		// 		.chain()
		// 		.in_set(RenderPass),),
		// )
		.run();
}
