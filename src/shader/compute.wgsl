
@group(0) @binding(0) var out_texture: texture_storage_2d<rgba32float, read_write>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
	let size = vec2f(textureDimensions(out_texture));
	let coord = (vec2f(gid.xy) - size / 2.0) / size.y;
	
	// let color = render_pixel(coord);
	let color = vec4f(vec2f(gid.xy) / size, 0, 1);
	
	textureStore(out_texture, gid.xy, color);
}