
struct Intersection {
	has_hit: bool,
	object: Object,
	distance: f32,
	position: vec3f,
	normal: vec3f,
	outgoing: vec3f,
}

struct Object {
	color: vec3f,
}

fn render_pixel(pixel_coord: vec2u, pixel_size: vec2u) {
	let coord = (vec2f(pixel_coord) - vec2f(pixel_size) / 2.0) / f32(pixel_size.y);
	
	let ray_origin = (camera.inverse_view_mat * vec4f(0, 0, 0, 1)).xyz;
	
	let focal_length = 1.0 / 2.0 / 0.5; // tan(45° / 2) = 0.5
	let ray_dir_raw = normalize(vec3f(coord, focal_length));
	let ray_dir = (camera.inverse_view_mat * vec4f(ray_dir_raw, 0.0)).xyz;
	
	let intersection = intersect_scene(ray_origin, ray_dir);
	
	var color = shade(intersection);
	
	color = post_processing_pipeline(coord, color);
	
	// TODO: Set the max distance dynamically
	let depth = vec4f(vec3f(intersection.distance / 100.0), 1.0);
	let normal = vec4f(intersection.normal, 1.0) * 0.5 + vec4f(0.5);

	textureStore(output_color, pixel_coord, color);
	textureStore(output_depth, pixel_coord, depth);
	textureStore(output_normal, pixel_coord, normal);
}

fn camera_look_at(position: vec3f, target_position: vec3f) -> mat3x3f {
	let fwd = normalize(target_position - position);
	let right = normalize(cross(fwd, vec3f(0, 1, 0)));
	let up = cross(right, fwd);
	return mat3x3f(right, up, fwd);
}

