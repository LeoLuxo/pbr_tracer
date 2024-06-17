f32 sphere(vec3f p, f32 radius)
{
  return length(p) - radius;
}

f32 floor(vec3f p, f32 height)
{
	return p.y - height;
}

f32 bbox(vec3f p, vec3f bounds)
{
  vec3f q = abs(p) - bounds / 2.0;
  return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

f32 octahedron(vec3f p, f32 size)
{
	p = abs(p);
	f32 m = p.x+p.y+p.z-size;
	vec3f q;
			if( 3.0*p.x < m ) q = p.xyz;
	else if( 3.0*p.y < m ) q = p.yzx;
	else if( 3.0*p.z < m ) q = p.zxy;
	else return m*0.57735027;
		
	f32 k = clamp(0.5*(q.z-q.y+size),0.0,size); 
	return length(vec3f(q.x,q.y-size+k,q.z-k)); 
}

f32 torus(vec3f p, f32 radius, f32 thickness)
{
  vec2f q = vec2f(length(p.xz) - radius, p.y);
  return length(q) - thickness;
}

f32 Capsule(vec3f p, vec3f a, vec3f b, f32 radius)
{
  vec3f pa = p - a;
  vec3f ba = b - a;
  f32 h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
  return length(pa - ba*h) - radius;
}