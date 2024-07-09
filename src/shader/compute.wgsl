

@compute
@workgroup_size(WORKGROUP_X, WORKGROUP_Y, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
	let resolution = textureDimensions(output_color);
	
	if gid.x >= resolution.x || gid.y >= resolution.y {
		return;
	}
	
	render_pixel(gid.xy, resolution);
}