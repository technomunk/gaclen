use gaclen::graphics::device::Device as GaclenDevice;
use gaclen::graphics::vulkano::memory::DeviceMemoryAllocError;
use gaclen::graphics::buffer::{BufferUsage, CpuAccessibleBuffer};

use std::sync::Arc;

#[derive(Clone, Default)]
pub struct Vertex {
	position: [f32; 3],
	normal: [f32; 3],
	uv: [f32; 2],
}

gaclen::graphics::impl_vertex!(Vertex, position, normal, uv);

pub fn generate_cube(device: &GaclenDevice) -> Result<Arc<CpuAccessibleBuffer<[Vertex]>>, DeviceMemoryAllocError> {
	CpuAccessibleBuffer::from_iter(device.logical_device(), BufferUsage::all(), false, [
		// X-
		Vertex { position: [ -0.5,  0.5, -0.5 ], normal: [ -1.0, 0.0, 0.0 ], uv: [ 1.0, 1.0 ] },
		Vertex { position: [ -0.5, -0.5, -0.5 ], normal: [ -1.0, 0.0, 0.0 ], uv: [ 0.0, 1.0 ] },
		Vertex { position: [ -0.5,  0.5,  0.5 ], normal: [ -1.0, 0.0, 0.0 ], uv: [ 1.0, 0.0 ] },
		Vertex { position: [ -0.5,  0.5,  0.5 ], normal: [ -1.0, 0.0, 0.0 ], uv: [ 1.0, 0.0 ] },
		Vertex { position: [ -0.5, -0.5, -0.5 ], normal: [ -1.0, 0.0, 0.0 ], uv: [ 0.0, 1.0 ] },
		Vertex { position: [ -0.5, -0.5,  0.5 ], normal: [ -1.0, 0.0, 0.0 ], uv: [ 0.0, 0.0 ] },
		
		// X+
		Vertex { position: [  0.5, -0.5, -0.5 ], normal: [ 1.0, 0.0, 0.0 ], uv: [ 1.0, 1.0 ] },
		Vertex { position: [  0.5,  0.5, -0.5 ], normal: [ 1.0, 0.0, 0.0 ], uv: [ 0.0, 1.0 ] },
		Vertex { position: [  0.5,  0.5,  0.5 ], normal: [ 1.0, 0.0, 0.0 ], uv: [ 0.0, 0.0 ] },
		Vertex { position: [  0.5,  0.5,  0.5 ], normal: [ 1.0, 0.0, 0.0 ], uv: [ 0.0, 0.0 ] },
		Vertex { position: [  0.5, -0.5,  0.5 ], normal: [ 1.0, 0.0, 0.0 ], uv: [ 1.0, 0.0 ] },
		Vertex { position: [  0.5, -0.5, -0.5 ], normal: [ 1.0, 0.0, 0.0 ], uv: [ 1.0, 1.0 ] },

		// Y-
		Vertex { position: [ -0.5, -0.5, -0.5 ], normal: [ 0.0, -1.0, 0.0 ], uv: [ 0.0, 1.0 ] },
		Vertex { position: [  0.5, -0.5, -0.5 ], normal: [ 0.0, -1.0, 0.0 ], uv: [ 1.0, 1.0 ] },
		Vertex { position: [  0.5, -0.5,  0.5 ], normal: [ 0.0, -1.0, 0.0 ], uv: [ 1.0, 0.0 ] },
		Vertex { position: [  0.5, -0.5,  0.5 ], normal: [ 0.0, -1.0, 0.0 ], uv: [ 1.0, 0.0 ] },
		Vertex { position: [ -0.5, -0.5,  0.5 ], normal: [ 0.0, -1.0, 0.0 ], uv: [ 0.0, 0.0 ] },
		Vertex { position: [ -0.5, -0.5, -0.5 ], normal: [ 0.0, -1.0, 0.0 ], uv: [ 0.0, 1.0 ] },

		// Y+
		Vertex { position: [ -0.5, 0.5, -0.5 ], normal: [ 0.0, 1.0, 0.0 ], uv: [ 1.0, 1.0 ] },
		Vertex { position: [  0.5, 0.5,  0.5 ], normal: [ 0.0, 1.0, 0.0 ], uv: [ 0.0, 0.0 ] },
		Vertex { position: [  0.5, 0.5, -0.5 ], normal: [ 0.0, 1.0, 0.0 ], uv: [ 0.0, 1.0 ] },
		Vertex { position: [  0.5, 0.5,  0.5 ], normal: [ 0.0, 1.0, 0.0 ], uv: [ 0.0, 0.0 ] },
		Vertex { position: [ -0.5, 0.5, -0.5 ], normal: [ 0.0, 1.0, 0.0 ], uv: [ 1.0, 1.0 ] },
		Vertex { position: [ -0.5, 0.5,  0.5 ], normal: [ 0.0, 1.0, 0.0 ], uv: [ 1.0, 0.0 ] },
		
		// Z-
		Vertex { position: [ -0.5, -0.5, -0.5 ], normal: [ 0.0, 0.0, -1.0 ], uv: [ 1.0, 1.0 ] },
		Vertex { position: [ -0.5,  0.5, -0.5 ], normal: [ 0.0, 0.0, -1.0 ], uv: [ 1.0, 0.0 ] },
		Vertex { position: [  0.5,  0.5, -0.5 ], normal: [ 0.0, 0.0, -1.0 ], uv: [ 0.0, 0.0 ] },
		Vertex { position: [  0.5,  0.5, -0.5 ], normal: [ 0.0, 0.0, -1.0 ], uv: [ 0.0, 0.0 ] },
		Vertex { position: [  0.5, -0.5, -0.5 ], normal: [ 0.0, 0.0, -1.0 ], uv: [ 0.0, 1.0 ] },
		Vertex { position: [ -0.5, -0.5, -0.5 ], normal: [ 0.0, 0.0, -1.0 ], uv: [ 1.0, 1.0 ] },

		// Z+
		Vertex { position: [ -0.5, -0.5,  0.5 ], normal: [ 0.0, 0.0, 1.0 ], uv: [ 1.0, 1.0 ] },
		Vertex { position: [  0.5,  0.5,  0.5 ], normal: [ 0.0, 0.0, 1.0 ], uv: [ 0.0, 0.0 ] },
		Vertex { position: [ -0.5,  0.5,  0.5 ], normal: [ 0.0, 0.0, 1.0 ], uv: [ 1.0, 0.0 ] },
		Vertex { position: [  0.5,  0.5,  0.5 ], normal: [ 0.0, 0.0, 1.0 ], uv: [ 0.0, 0.0 ] },
		Vertex { position: [ -0.5, -0.5,  0.5 ], normal: [ 0.0, 0.0, 1.0 ], uv: [ 1.0, 1.0 ] },
		Vertex { position: [  0.5, -0.5,  0.5 ], normal: [ 0.0, 0.0, 1.0 ], uv: [ 0.0, 1.0 ] },
	].iter().cloned())
}
