
@group(0) @binding(0) var out_texture: texture_storage_2d<rgba32float, read_write>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
	let pixel_coord = vec2f(gid.xy);
	let pixel_size = vec2f(textureDimensions(out_texture));
	
	let color = render_pixel(pixel_coord, pixel_size);
	
	textureStore(out_texture, gid.xy, color);
}