pub mod vertex {
	gaclen_shader::shader!{
		ty: "vertex",
		path: "examples/phong_cube/shader.vert",
	}

	// force recompilation on changes in shader source
	const _: &'static [u8] = include_bytes!("shader.vert");
}
pub mod fragment {
	gaclen_shader::shader!{
		ty: "fragment",
		path: "examples/phong_cube/shader.frag",
	}

	// force recompilation on changes in shader source
	const _: &'static [u8] = include_bytes!("shader.frag");
}
