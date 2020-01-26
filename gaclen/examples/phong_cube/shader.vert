#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(set = 0, binding = 0) uniform TransformData {
	mat4 model;
	mat4 view;
	mat4 proj;
} u_transform;

layout(location = 0) out vec3 out_position;
layout(location = 1) out vec3 out_normal;
layout(location = 2) out vec2 out_uv;

void main() {
	gl_Position = u_transform.proj * u_transform.view * u_transform.model * vec4(position, 1.0);

	out_position = (u_transform.model * vec4(position, 1.0)).xyz;
	out_normal = (u_transform.model * vec4(normal, 0.0)).xyz;
	out_uv = uv;
}
