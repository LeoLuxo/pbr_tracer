

#include "primitives.wgsl"


fn intersect_scene(ray_origin: vec3f, ray_dir: vec3f) -> Intersection {
	let object = Object(vec3f(1, 0, 0));
	var intersection = Intersection(false, object, vec3f(0), vec3f(0), -ray_dir);
	
	var iters: u32;
	var t = settings.min_march;
	var p = ray_origin;
	
	for (iters = 0u; iters < settings.max_march_steps && t < settings.max_march; iters++) {
		p = ray_origin + ray_dir * t;
		
		let distance = sdf(p);
		
		if (distance < settings.epsilon) {
			intersection.position = p;
			break;
		}
		
		t += distance;
	}
	
	if (t >= settings.max_march) {
		// Marched too far away, we didn't hit anything
		return intersection;
	}
	
	// Marched too often or marched too close, we "hit" something
	
	intersection.has_hit = true;
	intersection.position = p;
	intersection.normal = calc_normal(p);
	
	return intersection;
}

fn calc_normal(p: vec3f) -> vec3f {
	let h = 0.0001; // replace by an appropriate value
	let k = vec2f(1, -1);
	return normalize(k.xyy * sdf(p + k.xyy * h) + 
						  k.yyx * sdf(p + k.yyx * h) + 
						  k.yxy * sdf(p + k.yxy * h) + 
						  k.xxx * sdf(p + k.xxx * h));
}

fn sdf(p: vec3f) -> f32 {
	return sphere(p, 1.0);
}