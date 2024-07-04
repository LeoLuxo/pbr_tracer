
fn render_pixel(pixel_coord: vec2u, pixel_size: vec2u) {
	let color = vec4f(pixel_coord / pixel_size, 0, 1);
	textureStore(output_color, pixel_coord, color);
}