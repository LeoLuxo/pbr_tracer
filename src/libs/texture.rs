#![allow(dead_code)]

use brainrot::vek::{Extent2, Extent3};
use image::GenericImageView;
use wgpu::{
	AddressMode, AstcBlock, AstcChannel, CompareFunction, Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout,
	Origin3d, Sampler, SamplerBorderColor, SamplerDescriptor, StorageTextureAccess, TextureAspect, TextureDescriptor,
	TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor,
	TextureViewDimension,
};

use crate::core::gpu::Gpu;

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TextureAssetDimensions {
	D1(u32),
	D2(Extent2<u32>),
	D2Array(Extent2<u32>, u32),
	D3(Extent3<u32>),
	Cube(Extent2<u32>),
	CubeArray(Extent2<u32>, u32),
}

impl TextureAssetDimensions {
	pub fn get_size(&self) -> Extent3d {
		match self {
			TextureAssetDimensions::D1(size) => Extent3d {
				width: *size,
				..Default::default()
			},
			TextureAssetDimensions::D2(size) => Extent3d {
				width: size.w,
				height: size.h,
				..Default::default()
			},
			TextureAssetDimensions::D2Array(size, length) => Extent3d {
				width: size.w,
				height: size.h,
				depth_or_array_layers: *length,
			},
			TextureAssetDimensions::D3(size) => Extent3d {
				width: size.w,
				height: size.h,
				depth_or_array_layers: size.d,
			},
			TextureAssetDimensions::Cube(size) => Extent3d {
				width: size.w,
				height: size.h,
				depth_or_array_layers: 6,
			},
			TextureAssetDimensions::CubeArray(size, length) => Extent3d {
				width: size.w,
				height: size.h,
				depth_or_array_layers: 6 * length,
			},
		}
	}

	pub fn get_dimension(&self) -> TextureViewDimension {
		match self {
			TextureAssetDimensions::D1(_) => TextureViewDimension::D1,
			TextureAssetDimensions::D2(_) => TextureViewDimension::D2,
			TextureAssetDimensions::D2Array(_, _) => TextureViewDimension::D2Array,
			TextureAssetDimensions::D3(_) => TextureViewDimension::D3,
			TextureAssetDimensions::Cube(_) => TextureViewDimension::Cube,
			TextureAssetDimensions::CubeArray(_, _) => TextureViewDimension::CubeArray,
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TextureAssetDescriptor<'a> {
	pub label: &'a str,
	pub dimensions: TextureAssetDimensions,
	pub format: TextureFormat,
	pub usage: Option<TextureUsages>,
	pub aspect: TextureAspect,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TextureAssetSamplerDescriptor {
	pub filter: FilterMode,
	pub edges: Edges,
	pub compare: Option<CompareFunction>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Edges {
	ClampToEdge,
	Repeat,
	MirrorRepeat,
	ClampToColor(SamplerBorderColor),
}

impl Edges {
	pub fn as_address_mode(&self) -> AddressMode {
		match self {
			Edges::ClampToEdge => AddressMode::ClampToEdge,
			Edges::Repeat => AddressMode::Repeat,
			Edges::MirrorRepeat => AddressMode::MirrorRepeat,
			Edges::ClampToColor(_) => AddressMode::ClampToBorder,
		}
	}

	pub fn get_border_color(&self) -> Option<SamplerBorderColor> {
		match self {
			Edges::ClampToColor(color) => Some(*color),
			_ => None,
		}
	}
}

#[derive(Debug)]
pub struct TextureAsset {
	view_dimension: TextureViewDimension,
	aspect: TextureAspect,
	pub texture: wgpu::Texture,
	pub view: TextureView,
	pub sampler: Option<Sampler>,
}

impl TextureAsset {
	pub const DEFAULT_DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

	pub fn from_image_bytes_with_sampler(
		gpu: &Gpu,
		bytes: &[u8],
		format: TextureFormat,
		filter: FilterMode,
		edges: Edges,
		usage: Option<TextureUsages>,
		label: &str,
	) -> Self {
		let img = image::load_from_memory(bytes).expect("Couldn't load image bytes from memory");
		Self::from_image_with_sampler(gpu, label, &img, format, usage, filter, edges)
	}

	pub fn from_image_with_sampler(
		gpu: &Gpu,
		label: &str,
		img: &image::DynamicImage,
		format: TextureFormat,
		usage: Option<TextureUsages>,
		filter: FilterMode,
		edges: Edges,
	) -> Self {
		let texture = Self::create_with_sampler(
			gpu,
			TextureAssetDescriptor {
				label,
				dimensions: TextureAssetDimensions::D2(img.dimensions().into()),
				format,
				usage,
				aspect: TextureAspect::All,
			},
			TextureAssetSamplerDescriptor {
				edges,
				filter,
				compare: None,
			},
		);

		texture.upload_image(gpu, img);
		texture
	}

	pub fn from_image_storage(
		gpu: &Gpu,
		label: &str,
		img: &image::DynamicImage,
		format: TextureFormat,
		usage: Option<TextureUsages>,
	) -> Self {
		let texture = Self::create(
			gpu,
			TextureAssetDescriptor {
				label,
				dimensions: TextureAssetDimensions::D2(img.dimensions().into()),
				format,
				usage,
				aspect: TextureAspect::All,
			},
		);

		texture.upload_image(gpu, img);
		texture
	}

	pub fn create_depth_texture(gpu: &Gpu, size: Extent2<u32>, label: &str) -> Self {
		Self::create_with_sampler(
			gpu,
			TextureAssetDescriptor {
				label,
				dimensions: TextureAssetDimensions::D2(size),
				format: Self::DEFAULT_DEPTH_FORMAT,
				usage: Some(TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING),
				aspect: TextureAspect::DepthOnly,
			},
			TextureAssetSamplerDescriptor {
				edges: Edges::ClampToEdge,
				filter: FilterMode::Linear,
				compare: Some(CompareFunction::LessEqual),
			},
		)
	}

	pub fn create(gpu: &Gpu, desc: TextureAssetDescriptor) -> Self {
		let view_dimension = desc.dimensions.get_dimension();
		let aspect = desc.aspect;

		let texture = gpu.device.create_texture(&TextureDescriptor {
			label: Some(&format!("{} Texture", desc.label)),
			size: desc.dimensions.get_size(),
			mip_level_count: 1,
			sample_count: 1,
			dimension: view_dimension.compatible_texture_dimension(),
			format: desc.format,
			usage: desc.usage.unwrap_or(TextureUsages::empty())
				| TextureUsages::COPY_DST
				| TextureUsages::STORAGE_BINDING,
			// TODO: Clean up usages
			view_formats: &[],
		});

		let view = texture.create_view(&TextureViewDescriptor {
			label: Some(&format!("{} Texture View", desc.label)),
			format: Some(desc.format),
			dimension: Some(view_dimension),
			aspect,
			..Default::default()
		});

		Self {
			view_dimension,
			aspect,
			texture,
			view,
			sampler: None,
		}
	}

	pub fn create_with_sampler(
		gpu: &Gpu,
		mut desc: TextureAssetDescriptor,
		sampler_desc: TextureAssetSamplerDescriptor,
	) -> Self {
		// If the texture is gonna be sampled, it needs to be bound with TEXTURE_BINDING
		// anyway
		desc.usage = desc.usage.map(|u| u | TextureUsages::TEXTURE_BINDING);

		let sampler = Some(gpu.device.create_sampler(&SamplerDescriptor {
			label: Some(&format!("{} Sampler", desc.label)),
			address_mode_u: sampler_desc.edges.as_address_mode(),
			address_mode_v: sampler_desc.edges.as_address_mode(),
			address_mode_w: sampler_desc.edges.as_address_mode(),
			mag_filter: sampler_desc.filter,
			min_filter: sampler_desc.filter,
			mipmap_filter: sampler_desc.filter,
			border_color: sampler_desc.edges.get_border_color(),
			compare: sampler_desc.compare,
			..Default::default()
		}));

		Self {
			sampler,
			..Self::create(gpu, desc)
		}
	}

	pub fn upload_bytes(&self, gpu: &Gpu, bytes: &[u8]) {
		self.upload_bytes_layer(gpu, bytes, 0)
	}

	pub fn upload_bytes_layer(&self, gpu: &Gpu, bytes: &[u8], layer: u32) {
		let img = image::load_from_memory(bytes).expect("Couldn't load image bytes from memory");
		self.upload_image_layer(gpu, &img, layer)
	}

	pub fn upload_image(&self, gpu: &Gpu, img: &image::DynamicImage) {
		self.upload_image_layer(gpu, img, 0)
	}

	pub fn upload_image_layer(&self, gpu: &Gpu, img: &image::DynamicImage, layer: u32) {
		let rgba = img.to_rgba8();
		let dimensions = img.dimensions();

		// Panic to avoid dumb errors in the long run
		assert!(layer < self.size().depth_or_array_layers);
		assert!(dimensions.0 == self.size().width);
		assert!(dimensions.1 == self.size().height);

		gpu.queue.write_texture(
			ImageCopyTexture {
				aspect: self.aspect,
				texture: &self.texture,
				mip_level: 0,
				origin: Origin3d::ZERO,
			},
			&rgba,
			ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4 * dimensions.0),
				rows_per_image: Some(dimensions.1),
			},
			self.size(),
		);
	}

	pub fn view_dimension(&self) -> TextureViewDimension {
		self.view_dimension
	}

	pub fn dimension(&self) -> TextureDimension {
		self.view_dimension.compatible_texture_dimension()
	}

	pub fn size(&self) -> Extent3d {
		self.texture.size()
	}

	pub fn format(&self) -> TextureFormat {
		self.texture.format()
	}
}

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

// TODO
// pub struct TextureArray {
// 	pub textures: Vec<TextureAsset>,
// }

// impl TextureArray {
// 	pub fn from_bytes(gpu: &Gpu, array: Vec<(&[u8], Option<&str>)>, filter_mode:
// FilterMode) -> Result<Self> { 		let mut textures = Vec::new();

// 		for (bytes, label) in array {
// 			textures.push(TextureAsset::from_image_bytes(gpu, bytes, filter_mode,
// label)?); 		}

// 		Ok(TextureArray { textures })
// 	}

// 	#[allow(dead_code)]
// 	pub fn from_images(
// 		gpu: &Gpu,
// 		filter_mode: FilterMode,
// 		array: Vec<(&image::DynamicImage, Option<&str>)>,
// 	) -> Result<Self> {
// 		let mut textures = Vec::new();

// 		for (image, label) in array {
// 			textures.push(TextureAsset::from_image(gpu, image, filter_mode, label)?);
// 		}

// 		Ok(TextureArray { textures })
// 	}

// 	pub fn get_samplers(&self) -> Vec<&Sampler> {
// 		self.textures.iter().filter_map(|t| (&t.sampler).as_ref()).collect()
// 	}

// 	pub fn get_views(&self) -> Vec<&TextureView> {
// 		self.textures.iter().map(|t| &t.view).collect()
// 	}

// 	pub fn len(&self) -> usize {
// 		self.textures.len()
// 	}

// 	#[must_use]
// 	pub fn is_empty(&self) -> bool {
// 		self.len() == 0
// 	}
// }

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub fn dimension_to_string(dimension: TextureDimension) -> String {
	match dimension {
		TextureDimension::D1 => "1d",
		TextureDimension::D2 => "2d",
		TextureDimension::D3 => "3d",
	}
	.to_string()
}

#[rustfmt::skip]
pub fn access_to_string(access: StorageTextureAccess) -> String {
	match access {
		StorageTextureAccess::WriteOnly => "write",
		StorageTextureAccess::ReadOnly  => "read",
		StorageTextureAccess::ReadWrite => "read_write",
	}
	.to_string()
}

#[rustfmt::skip]
pub fn view_dimension_to_string(dimension: TextureViewDimension) -> String {
	match dimension {
		TextureViewDimension::D1 =>        "texture_1d",
		TextureViewDimension::D2 =>        "texture_2d",
		TextureViewDimension::D2Array =>   "texture_2d_array",
		TextureViewDimension::Cube =>      "texture_cube",
		TextureViewDimension::CubeArray => "texture_cube_array",
		TextureViewDimension::D3 =>        "texture_3d",
	}
	.to_string()
}

#[rustfmt::skip]
pub fn format_to_string(format: TextureFormat) -> String {
	match format {
		TextureFormat::R8Unorm              => "r8unorm".to_string(),
		TextureFormat::R8Snorm              => "r8snorm".to_string(),
		TextureFormat::R8Uint               => "r8uint".to_string(),
		TextureFormat::R8Sint               => "r8sint".to_string(),
		TextureFormat::R16Uint              => "r16uint".to_string(),
		TextureFormat::R16Sint              => "r16sint".to_string(),
		TextureFormat::R16Unorm             => "r16unorm".to_string(),
		TextureFormat::R16Snorm             => "r16snorm".to_string(),
		TextureFormat::R16Float             => "r16float".to_string(),
		TextureFormat::Rg8Unorm             => "rg8unorm".to_string(),
		TextureFormat::Rg8Snorm             => "rg8snorm".to_string(),
		TextureFormat::Rg8Uint              => "rg8uint".to_string(),
		TextureFormat::Rg8Sint              => "rg8sint".to_string(),
		TextureFormat::R32Uint              => "r32uint".to_string(),
		TextureFormat::R32Sint              => "r32sint".to_string(),
		TextureFormat::R32Float             => "r32float".to_string(),
		TextureFormat::Rg16Uint             => "rg16uint".to_string(),
		TextureFormat::Rg16Sint             => "rg16sint".to_string(),
		TextureFormat::Rg16Unorm            => "rg16unorm".to_string(),
		TextureFormat::Rg16Snorm            => "rg16snorm".to_string(),
		TextureFormat::Rg16Float            => "rg16float".to_string(),
		TextureFormat::Rgba8Unorm           => "rgba8unorm".to_string(),
		TextureFormat::Rgba8UnormSrgb       => "rgba8unorm-srgb".to_string(),
		TextureFormat::Rgba8Snorm           => "rgba8snorm".to_string(),
		TextureFormat::Rgba8Uint            => "rgba8uint".to_string(),
		TextureFormat::Rgba8Sint            => "rgba8sint".to_string(),
		TextureFormat::Bgra8Unorm           => "bgra8unorm".to_string(),
		TextureFormat::Bgra8UnormSrgb       => "bgra8unorm-srgb".to_string(),
		TextureFormat::Rgb10a2Uint          => "rgb10a2uint".to_string(),
		TextureFormat::Rgb10a2Unorm         => "rgb10a2unorm".to_string(),
		TextureFormat::Rg11b10Float         => "rg11b10ufloat".to_string(),
		TextureFormat::Rg32Uint             => "rg32uint".to_string(),
		TextureFormat::Rg32Sint             => "rg32sint".to_string(),
		TextureFormat::Rg32Float            => "rg32float".to_string(),
		TextureFormat::Rgba16Uint           => "rgba16uint".to_string(),
		TextureFormat::Rgba16Sint           => "rgba16sint".to_string(),
		TextureFormat::Rgba16Unorm          => "rgba16unorm".to_string(),
		TextureFormat::Rgba16Snorm          => "rgba16snorm".to_string(),
		TextureFormat::Rgba16Float          => "rgba16float".to_string(),
		TextureFormat::Rgba32Uint           => "rgba32uint".to_string(),
		TextureFormat::Rgba32Sint           => "rgba32sint".to_string(),
		TextureFormat::Rgba32Float          => "rgba32float".to_string(),
		TextureFormat::Stencil8             => "stencil8".to_string(),
		TextureFormat::Depth32Float         => "depth32float".to_string(),
		TextureFormat::Depth16Unorm         => "depth16unorm".to_string(),
		TextureFormat::Depth32FloatStencil8 => "depth32float-stencil8".to_string(),
		TextureFormat::Depth24Plus          => "depth24plus".to_string(),
		TextureFormat::Depth24PlusStencil8  => "depth24plus-stencil8".to_string(),
		TextureFormat::NV12                 => "nv12".to_string(),
		TextureFormat::Rgb9e5Ufloat         => "rgb9e5ufloat".to_string(),
		TextureFormat::Bc1RgbaUnorm         => "bc1-rgba-unorm".to_string(),
		TextureFormat::Bc1RgbaUnormSrgb     => "bc1-rgba-unorm-srgb".to_string(),
		TextureFormat::Bc2RgbaUnorm         => "bc2-rgba-unorm".to_string(),
		TextureFormat::Bc2RgbaUnormSrgb     => "bc2-rgba-unorm-srgb".to_string(),
		TextureFormat::Bc3RgbaUnorm         => "bc3-rgba-unorm".to_string(),
		TextureFormat::Bc3RgbaUnormSrgb     => "bc3-rgba-unorm-srgb".to_string(),
		TextureFormat::Bc4RUnorm            => "bc4-r-unorm".to_string(),
		TextureFormat::Bc4RSnorm            => "bc4-r-snorm".to_string(),
		TextureFormat::Bc5RgUnorm           => "bc5-rg-unorm".to_string(),
		TextureFormat::Bc5RgSnorm           => "bc5-rg-snorm".to_string(),
		TextureFormat::Bc6hRgbUfloat        => "bc6h-rgb-ufloat".to_string(),
		TextureFormat::Bc6hRgbFloat         => "bc6h-rgb-float".to_string(),
		TextureFormat::Bc7RgbaUnorm         => "bc7-rgba-unorm".to_string(),
		TextureFormat::Bc7RgbaUnormSrgb     => "bc7-rgba-unorm-srgb".to_string(),
		TextureFormat::Etc2Rgb8Unorm        => "etc2-rgb8unorm".to_string(),
		TextureFormat::Etc2Rgb8UnormSrgb    => "etc2-rgb8unorm-srgb".to_string(),
		TextureFormat::Etc2Rgb8A1Unorm      => "etc2-rgb8a1unorm".to_string(),
		TextureFormat::Etc2Rgb8A1UnormSrgb  => "etc2-rgb8a1unorm-srgb".to_string(),
		TextureFormat::Etc2Rgba8Unorm       => "etc2-rgba8unorm".to_string(),
		TextureFormat::Etc2Rgba8UnormSrgb   => "etc2-rgba8unorm-srgb".to_string(),
		TextureFormat::EacR11Unorm          => "eac-r11unorm".to_string(),
		TextureFormat::EacR11Snorm          => "eac-r11snorm".to_string(),
		TextureFormat::EacRg11Unorm         => "eac-rg11unorm".to_string(),
		TextureFormat::EacRg11Snorm         => "eac-rg11snorm".to_string(),
		TextureFormat::Astc { block, channel } => {
			let block = match block {
				AstcBlock::B4x4   => "4x4",
				AstcBlock::B5x4   => "5x4",
				AstcBlock::B5x5   => "5x5",
				AstcBlock::B6x5   => "6x5",
				AstcBlock::B6x6   => "6x6",
				AstcBlock::B8x5   => "8x5",
				AstcBlock::B8x6   => "8x6",
				AstcBlock::B8x8   => "8x8",
				AstcBlock::B10x5  => "10x5",
				AstcBlock::B10x6  => "10x6",
				AstcBlock::B10x8  => "10x8",
				AstcBlock::B10x10 => "10x10",
				AstcBlock::B12x10 => "12x10",
				AstcBlock::B12x12 => "12x12",
			};

			let channel = match channel {
				AstcChannel::Unorm     => "unorm",
				AstcChannel::UnormSrgb => "unorm-srgb",
				AstcChannel::Hdr       => "hdr",
			};

			format!("astc-{block}-{channel}")
		}
	}
	
}

pub fn format_to_type_string(format: TextureFormat) -> String {
	match format.sample_type(None, None) {
		Some(TextureSampleType::Float { .. }) => "f32",
		Some(TextureSampleType::Sint { .. }) => "i32",
		Some(TextureSampleType::Uint { .. }) => "u32",
		_ => unimplemented!(),
	}
	.to_string()
}
