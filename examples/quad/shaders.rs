pub mod vertex {
	vulkano_shaders::shader!{
		ty: "vertex",
		path: "examples/quad/shader.vert",
	}
}
pub mod fragment {
	vulkano_shaders::shader!{
		ty: "fragment",
		path: "examples/quad/shader.frag",
	}
}
