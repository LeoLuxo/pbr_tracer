
@group(0) @binding(0) var output_tex: texture_storage_2d<rgba32float, read_write>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
	let coord = vec3f(gid) / vec3f(num_workgroups);
	
	textureStore(output_tex, gid.xy, vec4(coord, 1));
}