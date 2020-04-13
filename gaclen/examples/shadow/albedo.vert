#version 450

// per vertex input
layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;

// uniforms
layout(set = 0) uniform Model { mat4 model_matrix; } u_model;
layout(set = 1) uniform Light { mat4 view_projection_matrix; } u_light;

layout(push_constant) uniform PushConstantData {
	mat4 view_projection_matrix; // Camera view-projection matrix
} pc;

// ouput
layout(location = 0) out vec4 out_pos_lightspace;
layout(location = 1) out vec4 out_color;

void main() {
	vec4 world_pos = u_model.model_matrix * vec4(position, 1.0);

	out_pos_lightspace = (u_light.view_projection_matrix * world_pos) * vec4(0.5, 0.5, 1, 1) + vec4(0.5, 0.5, 0, 0);
	out_color = color;

	gl_Position = pc.view_projection_matrix * world_pos;
}
