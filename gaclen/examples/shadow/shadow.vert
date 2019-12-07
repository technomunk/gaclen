#version 450

// #include "light.glsl"

// per vertex input
layout(location = 0) in vec3 in_position;

// uniform
layout(binding = 0) uniform Model { mat4 model_matrix; } u_model;
layout(binding = 1) uniform Light { mat4 view_projection_matrix; } u_light;

void main() {
	gl_Position = u_light.view_projection_matrix * u_model.model_matrix * vec4(in_position, 1.0);
}