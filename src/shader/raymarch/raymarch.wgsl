

#include "primitives.wgsl"


fn intersect_scene(ray_origin: vec3f, ray_dir: vec3f) -> Intersection {
	// struct Intersection {
	// 	has_hit: bool,
	// 	object: Object,
	// 	distance: f32,
	// 	position: vec3f,
	// 	normal: vec3f,
	// 	outgoing: vec3f,
	// }
	let object = Object(vec3f(1, 0, 0));
	var intersection = Intersection(false, object, 0.0, vec3f(0), vec3f(0), -ray_dir);
	
	var iters: u32;
	var t = settings.min_march;
	var p = ray_origin;
	
	for (iters = 0u; iters < settings.max_march_steps && t < camera.z_far; iters++) {
		p = ray_origin + ray_dir * t;
		
		let distance = sdf(p);
		
		if (distance < settings.epsilon) {
			break;
		}
		
		t += distance;
	}
	
	if (t >= camera.z_far) {
		// Marched too far away, we didn't hit anything
		intersection.distance = camera.z_far;
		return intersection;
	}
	
	// Marched too often or marched too close, we "hit" something
	
	intersection.has_hit = true;
	intersection.distance = t;
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
	return min(sphere(p, 1.0), sphere(p - vec3f(2, 3, 1), 2.0));
	// return sphere(p, 1.0);
}