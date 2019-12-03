#version 450

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec2 in_uv;

layout(set = 1, binding = 0) uniform LightData {
	vec4 position;
	vec4 ambient;
	vec4 diffuse;
	vec4 specular;
} u_light;

layout(location = 0) out vec4 out_color;

void main() {
	vec3 color = u_light.ambient.xyz + u_light.diffuse.xyz * clamp(dot(normalize(u_light.position.xyz - in_position), in_normal), 0, 1);
	out_color = vec4(color, 1);
}