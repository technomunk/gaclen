pub mod vertex {
	vulkano_shaders::shader!{
		ty: "vertex",
		src: "
#version 450

layout(location = 0) in vec2 position;

layout(push_constant) uniform PushConstantData {
	vec4 color_rotation;
} pc;

layout(location = 0) out vec4 out_color;

void main() {
	gl_Position = vec4(position, 0.0, 1.0);

	float sinw = sin(pc.color_rotation.w);
	float cosw = cos(pc.color_rotation.w);

	mat2 rotMatrix;
	rotMatrix[0] = vec2(cosw, sinw);
	rotMatrix[1] = vec2(-sinw, cosw);
	
	gl_Position.xy = rotMatrix * gl_Position.xy;

	out_color = vec4(pc.color_rotation.xyz, 1.0);
}"
	}
}

pub mod fragment {
	vulkano_shaders::shader!{
		ty: "fragment",
		src: "
#version 450

layout(location = 0) in vec4 in_color;

layout(location = 0) out vec4 out_color;

void main() {
	out_color = in_color;
}"
	}
}