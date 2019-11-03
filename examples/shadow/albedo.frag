#version 450

// Input
layout(location = 0) in vec4 in_pos_lightspace;
layout(location = 1) in vec4 in_color;

// Uniform
layout(set = 2, binding = 0) uniform sampler2D u_shadow;

// Output
layout(location = 0) out vec4 out_color;

float shadow(vec4 lightspace) {
	vec3 proj = lightspace.xyz / lightspace.w;
	proj = proj * 0.5 + 0.5;

	float light_depth = texture(u_shadow, proj.xy).r;
	float frag_depth = proj.z;

	return float(frag_depth > light_depth);
}

void main() {
	out_color = in_color * (1.0 - shadow(in_pos_lightspace));
}