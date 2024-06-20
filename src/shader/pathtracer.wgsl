

fn render_pixel(coord: vec2f) -> vec4f {
	if (coord.x < 0.01 && coord.x > -0.01 && coord.y < 0.01 && coord.y > -0.01) {
		return vec4f(1, 0, 0, 1);
	}

	let ray_origin = vec3f(0, 0, -5);
	let ray_target = vec3f(0);
	
	let view_mat = camera_look_at(ray_origin, ray_target);
	
	let focal_length = 1.0 / 2.0 / 0.5; // tan(45° / 2) = 0.5
	
	let ray_dir = view_mat * normalize(vec3f(coord, focal_length));
	
	var color = send_ray(ray_origin, ray_dir);
	// color = pow(color, vec3(0.4545));

	return color;
}

fn camera_look_at(position: vec3f, target_position: vec3f) -> mat3x3f {
	let fwd = normalize(target_position - position);
	let right = normalize(cross(fwd, vec3f(0, 1, 0)));
	let up = cross(right, fwd);
	return mat3x3f(right, up, fwd);
}

