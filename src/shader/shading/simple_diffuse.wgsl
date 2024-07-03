fn shade(intersection: Intersection) -> vec4f {
	if !intersection.has_hit {
		return vec4f(0.0, 0.6, 1.0, 1.0);
	}

	let object = intersection.object;

	var diffuse = dot(intersection.normal, -sun_direction);
	diffuse = max(diffuse, 0.0);
	
	let color = object.color * diffuse;
	
	return vec4f(color, 1.0);
}