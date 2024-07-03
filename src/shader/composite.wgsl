
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
	
	var tex_coord = get_texture_coordinates(frag_coord.xy, texture_size, screen_size);
	
	// Invert the y coordinate since texture.y is from top to bottom.
	tex_coord.y = 1.0 - tex_coord.y;

	return textureSample(out_texture, out_sampler, tex_coord);
}

fn get_texture_coordinates(frag_coord: vec2f, texture_size: vec2f, screen_size: vec2f) -> vec2f {
	if texture_size.x / texture_size.y < screen_size.x / screen_size.y {
		// texture is TALLER than the screen
		let x = frag_coord.x / screen_size.x;
		let size_y = screen_size.y / screen_size.x / texture_size.y * texture_size.x;
		let y = frag_coord.y / screen_size.y * size_y + (1.0 - size_y) * 0.5;
		return vec2f(x, y);
	} else {
		// texture is WIDER than the screen
		let y = frag_coord.y / screen_size.y;
		let size_x = screen_size.x / screen_size.y / texture_size.x * texture_size.y;
		let x = frag_coord.x / screen_size.x * size_x + (1.0 - size_x) * 0.5;
		return vec2f(x, y);
	}
}
