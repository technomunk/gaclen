pub mod shadow {
	pub mod vertex {
		gaclen_shader::shader!{
			ty: "vertex",
			path: "examples/scene/graphics/shadow.vert",
		}
	}
	pub mod fragment {
		gaclen_shader::shader!{
			ty: "fragment",
			path: "examples/scene/graphics/shadow.frag",
		}
	}
}
pub mod albedo {
	pub mod vertex {
		gaclen_shader::shader!{
			ty: "vertex",
			path: "examples/scene/graphics/albedo.vert",
		}
	}
	pub mod fragment {
		gaclen_shader::shader!{
			ty: "fragment",
			path: "examples/scene/graphics/albedo.frag",
		}
	}
}
