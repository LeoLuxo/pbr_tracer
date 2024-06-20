fn sphere(p: vec3f, radius: f32) -> f32
{
	return length(p) - radius;
}

fn floor(p: vec3f, height: f32) -> f32
{
	return p.y - height;
}

fn bbox(p: vec3f, bounds: vec3f) -> f32
{
	let q = abs(p) - bounds / 2.0;
	return length(max(q, vec3f(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

fn octahedron(p: vec3f, size: f32) -> f32
{
	let p2 = abs(p);
	let m = p2.x+p2.y+p2.z-size;
	var q: vec3f;
	     if( 3.0*p2.x < m ) {q = p2.xyz;}
	else if( 3.0*p2.y < m ) {q = p2.yzx;}
	else if( 3.0*p2.z < m ) {q = p2.zxy;}
	else {return m*0.57735027;}
		
	let k = clamp(0.5*(q.z-q.y+size),0.0,size); 
	return length(vec3f(q.x,q.y-size+k,q.z-k)); 
}

fn torus(p: vec3f, radius: f32, thickness: f32) -> f32
{
	let q = vec2f(length(p.xz) - radius, p.y);
	return length(q) - thickness;
}

fn capsule(p: vec3f, a: vec3f, b: vec3f, radius: f32) -> f32
{
	let pa = p - a;
	let ba = b - a;
	let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
	return length(pa - ba*h) - radius;
}