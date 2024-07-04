
struct Intersection {
	has_hit: bool,
	object: Object,
	position: vec3f,
	normal: vec3f,
	outgoing: vec3f,
}

struct Object {
	color: vec3f,
}

fn render_pixel(pixel_coord: vec2u, pixel_size: vec2u) {
	let coord = (vec2f(pixel_coord) - vec2f(pixel_size) / 2.0) / f32(pixel_size.y);
	
	let color = render(coord);
	
	textureStore(output_color, pixel_coord, color);
}

fn render(coord: vec2f) -> vec4f {
	let ray_origin = vec3f(0, 0, -5);
	let ray_target = vec3f(0);
	
	let view_mat = camera_look_at(ray_origin, ray_target);
	
	let focal_length = 1.0 / 2.0 / 0.5; // tan(45° / 2) = 0.5
	
	let ray_dir = view_mat * normalize(vec3f(coord, focal_length));
	
	let intersection = intersect_scene(ray_origin, ray_dir);
	
	var color = shade(intersection);
	
	color = post_processing_pipeline(coord, color);

	return color;
}

fn camera_look_at(position: vec3f, target_position: vec3f) -> mat3x3f {
	let fwd = normalize(target_position - position);
	let right = normalize(cross(fwd, vec3f(0, 1, 0)));
	let up = cross(right, fwd);
	return mat3x3f(right, up, fwd);
}

