#version 450

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec2 in_uv;

layout(set = 0, binding = 1) uniform LightData {
	vec3 position;
	vec3 direct;
	vec3 ambient;
} u_light;

layout(location = 0) out vec4 out_color;

void main() {
	vec3 color = u_light.ambient + u_light.direct * clamp(dot(normalize(u_light.position - in_position), in_normal), 0, 1);
	out_color = vec4(color, 1);
}