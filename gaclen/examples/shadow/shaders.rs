pub mod shadow {
	pub mod vertex {
		gaclen_shader::shader!{
			ty: "vertex",
			path: "examples/shadow/shadow.vert",
		}
	}
	pub mod fragment {
		gaclen_shader::shader!{
			ty: "fragment",
			path: "examples/shadow/shadow.frag",
		}
	}
}
pub mod albedo {
	pub mod vertex {
		gaclen_shader::shader!{
			ty: "vertex",
			path: "examples/shadow/albedo.vert",
		}
	}
	pub mod fragment {
		gaclen_shader::shader!{
			ty: "fragment",
			path: "examples/shadow/albedo.frag",
		}
	}
}
