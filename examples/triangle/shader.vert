#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;

layout(push_constant) uniform PushConstantData {
	mat4 MVP;
} pc;

layout(location = 0) out vec4 out_color;

void main() {
	gl_Position = MVP * vec4(position, 0.0);

	out_color = color;
}