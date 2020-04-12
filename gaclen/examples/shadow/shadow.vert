#version 450

// #include "light.glsl"

// per vertex input
layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;

// uniform
layout(set = 0) uniform Model { mat4 model_matrix; } u_model;
layout(set = 1) uniform Light { mat4 view_projection_matrix; } u_light;

void main() {
	gl_Position = u_light.view_projection_matrix * (u_model.model_matrix * vec4(position, 1.0));
}
