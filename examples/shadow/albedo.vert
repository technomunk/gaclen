#version 450

// Per-vertex input
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec4 in_color;

// Per-instance input
// NOTE: mat4 takes up 4 location blocks, this will be caught by shaderc, but the error message is just about the overlap.
layout(location = 2) in mat4 in_model_matrix;
layout(location = 6) in mat4 in_viewprojection_matrix;
layout(location = 10) in mat4 in_light_matrix; // view-projection matrix for the light source

// Ouput
layout(location = 0) out vec4 out_pos_lightspace;
layout(location = 1) out vec4 out_color;

void main() {
	vec4 world_pos = in_model_matrix * vec4(in_position, 1.0);
	
	out_pos_lightspace = in_light_matrix * world_pos;
	out_color = in_color;

	gl_Position = in_viewprojection_matrix * world_pos;
}