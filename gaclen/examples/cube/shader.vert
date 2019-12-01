#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(push_constant) uniform PushConstantData {
	mat4 MVP;
} pc;

layout(location = 0) out vec3 out_position;
layout(location = 1) out vec3 out_normal;
layout(location = 2) out vec2 out_uv;

void main() {
	gl_Position = pc.MVP * vec4(position, 1.0);
	
	out_position = gl_Position.xyz;
	out_normal = normal;//(pc.MVP * vec4(normal, 0.0)).xyz;
	out_uv = uv;
}