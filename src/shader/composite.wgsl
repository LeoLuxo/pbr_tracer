
// @group(0) @binding(0) var out_texture: texture_2d<f32>;
// @group(0) @binding(1) var out_sampler: sampler;

// @group(0) @binding(0) var<uniform> viewport_size: vec2u;


@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
	var x = -1.0 + f32((vertex_index & 1) * 2);
	var y = -1.0 + f32(vertex_index & 2);

	return vec4(x, y, 0, 1);
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4f) -> @location(0) vec4f {
	let texture_size = vec2f(textureDimensions(out_texture));
	let screen_size = vec2f(viewport_size);
	
	var tex_coord: vec2f;
	
	if texture_size.x / texture_size.y < screen_size.x / screen_size.y {
		// texture is TALLER than the screen
		tex_coord.x = frag_coord.x / screen_size.x;
		let size_y = screen_size.y / screen_size.x / texture_size.y * texture_size.x;
		tex_coord.y = frag_coord.y / screen_size.y * size_y + (1.0 - size_y) * 0.5;
	} else {
		// texture is WIDER than the screen
		tex_coord.y = frag_coord.y / screen_size.y;
		let size_x = screen_size.x / screen_size.y / texture_size.x * texture_size.y;
		tex_coord.x = frag_coord.x / screen_size.x * size_x + (1.0 - size_x) * 0.5;
	}

	return textureSample(out_texture, out_sampler, tex_coord);
}
