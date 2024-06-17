use std::path::PathBuf;

use brainrot::{native_pathbuf, path};

fn main() {
	use std::{
		env,
		fs::{read_to_string, File},
		io::{BufWriter, Write},
	};

	let dir = path!("src/shader");
	let absolute_dir = path!(env!("CARGO_MANIFEST_DIR")).join(&dir);
	let destination = path!("shader_dir.rs");

	// Tell Cargo that if the directory changes, to rerun this build script.
	println!("cargo::rerun-if-changed={}", dir);

	let mut map = phf_codegen::Map::<String>::new();
	// Set the path that will be printed in the resulting source to the phf re-export (so that it isn't needed in the destination lib)
	map.phf_path("brainrot::lib_crates::phf");

	let shader_files = glob::glob(absolute_dir.join("**/*").as_str()).unwrap();

	for entry in shader_files {
		let path_buf = if let Ok(path) = entry {
			path
		} else {
			continue;
		};
		if !path_buf.is_file() {
			continue;
		}

		let source = read_to_string(&path_buf).unwrap();

		// Convert path_buf to a typed_path
		let path_buf = path!(&path_buf.to_string_lossy());

		// Make the path relative from the shader dir, and set the root
		let shader_path_relative = path_buf.strip_prefix(&absolute_dir).unwrap().with_unix_encoding();
		let shader_path_rooted = path!("/").join(shader_path_relative);
		let shader_path_str = shader_path_rooted.into_string();

		// The program source needs to be quoted, as the value of the map is printed *as-is*
		map.entry(shader_path_str, &format!("r#\"{}\"#", &source));
	}

	let out_path = path!(&env::var("OUT_DIR").unwrap()).join(destination);
	let out_file = File::create(native_pathbuf!(out_path).unwrap()).unwrap();
	println!("cargo::warning={:?}", out_file);
	let mut out_writer = BufWriter::new(out_file);

	write!(&mut out_writer, "{}", map.build()).unwrap();
}
