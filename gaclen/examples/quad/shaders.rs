pub mod vertex {
	gaclen_shader::shader!{
		ty: "vertex",
		path: "examples/quad/shader.vert",
	}
}
pub mod fragment {
	gaclen_shader::shader!{
		ty: "fragment",
		path: "examples/quad/shader.frag",
	}
}
