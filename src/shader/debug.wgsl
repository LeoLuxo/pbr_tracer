
fn render_pixel(pixel_coord: vec2f, pixel_size: vec2f) -> vec4f {
	let color = vec4f(pixel_coord / pixel_size, 0, 1);
	return color;
}