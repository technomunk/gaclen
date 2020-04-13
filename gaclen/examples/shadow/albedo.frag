#version 450

// Input
layout(location = 0) in vec4 in_pos_lightspace;
layout(location = 1) in vec4 in_color;

// Uniform
layout(set = 2, binding = 0) uniform sampler2DShadow u_shadow;

// Output
layout(location = 0) out vec4 out_color;

const float DepthBias = 2e-3;

float in_shadow(vec4 lightspace) {
	vec3 proj = lightspace.xyz / lightspace.w;
	proj.z += DepthBias;
	return texture(u_shadow, proj.xyz);
}

void main() {
	// out_color = in_color;
	out_color = in_color * (1.0 - 0.5 * in_shadow(in_pos_lightspace));
}
