

fn post_processing_pipeline(coord: vec2f, color_in: vec4f) -> vec4f {
	var color = color_in;
	
	CALL_EFFECTS
	
	return color;
}