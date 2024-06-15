use log::LevelFilter;

fn main() {
	env_logger::Builder::new()
		.filter_level(LevelFilter::Error)
		.filter_module("pbr_tracer", LevelFilter::Debug)
		.init();

	pbr_tracer::run();
}
