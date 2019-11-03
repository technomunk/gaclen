pub mod shadow {
	pub mod vertex {
		vulkano_shaders::shader!{
			ty: "vertex",
			path: "examples/shadow/shadow.vert",
		}
	}
	pub mod fragment {
		vulkano_shaders::shader!{
			ty: "fragment",
			path: "examples/shadow/shadow.frag",
		}
	}
}
pub mod albedo {
	pub mod vertex {
		vulkano_shaders::shader!{
			ty: "vertex",
			path: "examples/shadow/albedo.vert",
		}
	}
	pub mod fragment {
		vulkano_shaders::shader!{
			ty: "fragment",
			path: "examples/shadow/albedo.frag",
		}
	}
}