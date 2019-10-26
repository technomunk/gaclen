pub mod vertex {
	vulkano_shaders::shader!{
		ty: "vertex",
		path: "shader.vert",
	}
}
pub mod fragment {
	vulkano_shaders::shader!{
		ty: "fragment",
		path: "shader.frag",
	}
}
