
#define EPSILON 0.00001
#define MIN_MARCH 0.1
#define MAX_MARCH 1000.0
#define MAX_MARCH_STEPS 1024

#include "primitives.wgsl"

fn send_ray(ray_origin: vec3f, ray_dir: vec3f) -> vec4f {
	var iters: u32;
	var t = MIN_MARCH;
	
	for (iters = 0u; iters < MAX_MARCH_STEPS && t < MAX_MARCH; iters++) {
		let p = ray_origin + ray_dir * t;
		let distance = sdf(p);
		
		if (distance < EPSILON) {break;}
		
		t += distance;
	}
	
	if (t >= MAX_MARCH) {
		t = -1.0;
	}
	
	var color = vec4f(vec3f(f32(iters) / f32(MAX_MARCH_STEPS)), 1);
	
	return color;
}

fn sdf(p: vec3f) -> f32 {
	return sphere(p, 1.0);
}