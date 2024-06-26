

#include "primitives.wgsl"


fn send_ray(ray_origin: vec3f, ray_dir: vec3f) -> vec4f {
	var iters: u32;
	var t = settings.min_march;
	
	for (iters = 0u; iters < settings.max_march_steps && t < settings.max_march; iters++) {
		let p = ray_origin + ray_dir * t;
		let distance = sdf(p);
		
		if (distance < settings.epsilon) {break;}
		
		t += distance;
	}
	
	if (t >= settings.max_march) {
		t = -1.0;
	}
	
	var color = vec4f(vec3f(f32(iters) / f32(settings.max_march_steps)), 1);
	
	return color;
}

fn sdf(p: vec3f) -> f32 {
	return sphere(p, 1.0);
}