
@group(0) @binding(0) var out_texture: texture_2d<f32>;
@group(0) @binding(1) var out_sampler: sampler;

@group(1) @binding(0) var<uniform> viewport_size: vec2u;


@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
	var x = -1.0 + f32((vertex_index & 1) * 2);
	var y = -1.0 + f32(vertex_index & 2);

	return vec4(x, y, 0, 1);
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4f) -> @location(0) vec4f {
	let coord = frag_coord.xy / vec2f(viewport_size);

	// return vec4(frag_coord.xy / vec2f(viewport_size), 0, 1);
	return textureSample(out_texture, out_sampler, coord);
	// return vec4f(0);
}
