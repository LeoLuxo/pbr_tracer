
@group(0) @binding(0) var out_texture: texture_storage_2d<rgba32float, read_write>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
	let coord = vec3f(gid).xy / f32(num_workgroups.y) * 2.0 - 1.0;
	
	let color = render_pixel(coord);
	
	textureStore(out_texture, gid.xy, color);
}