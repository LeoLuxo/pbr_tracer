use log::LevelFilter;

fn main() {
	env_logger::Builder::new()
		.filter_level(LevelFilter::Error)
		.filter_module("pathtracer", LevelFilter::Debug)
		.init();

	pathtracer::run();
}
