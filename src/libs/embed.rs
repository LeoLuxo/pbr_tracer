/// A trait-object-safe version of rust_embed::Embed
pub trait Assets {
	fn get(&self, file_path: &str) -> Option<rust_embed::EmbeddedFile>;
	fn iter(&self) -> rust_embed::Filenames;
}

impl<T: rust_embed::Embed> Assets for T {
	fn get(&self, file_path: &str) -> Option<rust_embed::EmbeddedFile> {
		<Self as rust_embed::Embed>::get(file_path)
	}

	fn iter(&self) -> rust_embed::Filenames {
		<Self as rust_embed::Embed>::iter()
	}
}
