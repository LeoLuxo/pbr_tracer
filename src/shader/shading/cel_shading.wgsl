fn shade(intersection: Intersection) -> vec4f {
	if !intersection.has_hit {
		return vec4f(0.0, 0.6, 1.0, 1.0);
	}

	let object = intersection.object;

	let full_diffuse = dot(intersection.normal, -sun_direction) * 0.5 + 0.5;
	let cel_diffuse = get_gradient_value(full_diffuse);
	
	let color = object.color * cel_diffuse;
	
	return vec4f(color, 1.0);
}

fn get_gradient_value(diffuse: f32) -> vec3f {
	let coords = vec2f(diffuse, 0.5);
	let fitted_coords = coords * vec2f(textureDimensions(cel_gradient));
	return textureLoad(cel_gradient, vec2u(fitted_coords)).rgb;
}