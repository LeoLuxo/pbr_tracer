

fn post_processing_effect(coord: vec2f, color: vec4f) -> vec4f {
	return pow(color, vec4f(1.0 / gamma));
}