fn main() {
	use std::{
		env,
		fs::{read_to_string, File},
		io::{BufWriter, Write},
		path::Path,
	};

	let dir = "src/shader";
	let full_dir = &format!("{}/{}/", env!("CARGO_MANIFEST_DIR"), dir);
	let destination = "shader_dir.rs";

	// Tell Cargo that if the directory changes, to rerun this build script.
	println!("cargo::rerun-if-changed={}", dir);

	let mut map = phf_codegen::Map::<Path>::new();
	// Set the path that will be printed in the resulting source to the phf re-export (so that it isn't needed in the destination lib)
	map.phf_path("brainrot::lib_crates::phf");

	let shader_files = glob::glob(&format!("{}**/*", full_dir)).unwrap();
	for entry in shader_files {
		let path_buf = if let Ok(path) = entry { path } else { continue };
		if !path_buf.is_file() {
			continue;
		}

		let source = read_to_string(&path_buf).unwrap();
		let path: String = path_buf.strip_prefix(full_dir).unwrap().to_string_lossy().into();

		// The program source needs to be quoted, as the value of the map is printed *as-is*
		map.entry(path, &format!("r#\"{}\"#", &source));
	}

	let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join(destination);
	let mut out_file = BufWriter::new(File::create(out_path).unwrap());

	write!(&mut out_file, "{}", map.build()).unwrap();
}
