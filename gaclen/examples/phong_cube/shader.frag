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
layout(set = 1, binding = 1) uniform sampler2D u_texture;

layout(location = 0) out vec4 out_color;

void main() {
	vec3 light_dir = u_light.position.xyz - in_position;
	float light_dist = length(light_dir);
	light_dist = light_dist * light_dist;
	light_dir = normalize(light_dir);
	vec3 normal = normalize(in_normal);

	float lambertian = max(dot(light_dir, normal), 0.0);
	float specular = 0.0;

	// TODO: specularity

	vec3 color =
		u_light.ambient.xyz +
		u_light.diffuse.xyz * lambertian * u_light.diffuse.w / light_dist +
		u_light.specular.xyz * specular / light_dist;
	out_color = vec4(color * texture(u_texture, in_uv).xyz, 1);
}