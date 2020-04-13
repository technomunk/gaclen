use gaclen::graphics::device::Device;

use gaclen::graphics::buffer::{BufferUsage, CpuAccessibleBuffer};

use std::sync::Arc;

#[derive(Default, Debug, Clone)]
pub struct Vertex {
	pos: [f32; 3],
	col: [f32; 4],
}
gaclen::graphics::impl_vertex!(Vertex, pos, col);


/// Generate a buffer with quad geometry.
pub fn generate_quad(device: &Device) -> Arc<CpuAccessibleBuffer<[Vertex]>> {
	CpuAccessibleBuffer::from_iter(device.logical_device(), BufferUsage::all(), false, [
		Vertex { pos: [-0.5, 0.5, 0.0 ], col: [ 0.75, 0.75, 0.75, 1.0 ] },
		Vertex { pos: [ 0.5, 0.5, 0.0 ], col: [ 0.75, 0.75, 0.75, 1.0 ] },
		Vertex { pos: [ 0.5,-0.5, 0.0 ], col: [ 0.75, 0.75, 0.75, 1.0 ] },
		Vertex { pos: [-0.5,-0.5, 0.0 ], col: [ 0.75, 0.75, 0.75, 1.0 ] },
		Vertex { pos: [-0.5, 0.5, 0.0 ], col: [ 0.75, 0.75, 0.75, 1.0 ] },
		Vertex { pos: [ 0.5,-0.5, 0.0 ], col: [ 0.75, 0.75, 0.75, 1.0 ] },
	].iter().cloned()).unwrap()
}

/// Generate a buffer with cube geometry.
pub fn generate_cube(device: &Device) -> Arc<CpuAccessibleBuffer<[Vertex]>> {
	CpuAccessibleBuffer::from_iter(device.logical_device(), BufferUsage::all(), false, [
			// X-
			Vertex { pos: [ -0.5, -0.5, -0.5 ], col: [ 0.6, 0.7, 0.7, 1.0 ] },
			Vertex { pos: [ -0.5,  0.5, -0.5 ], col: [ 0.6, 0.7, 0.7, 1.0 ] },
			Vertex { pos: [ -0.5,  0.5,  0.5 ], col: [ 0.6, 0.7, 0.7, 1.0 ] },
			Vertex { pos: [ -0.5, -0.5, -0.5 ], col: [ 0.6, 0.7, 0.7, 1.0 ] },
			Vertex { pos: [ -0.5,  0.5,  0.5 ], col: [ 0.6, 0.7, 0.7, 1.0 ] },
			Vertex { pos: [ -0.5, -0.5,  0.5 ], col: [ 0.6, 0.7, 0.7, 1.0 ] },
			
			// X+
			Vertex { pos: [  0.5,  0.5, -0.5 ], col: [ 0.8, 0.7, 0.7, 1.0 ] },
			Vertex { pos: [  0.5, -0.5, -0.5 ], col: [ 0.8, 0.7, 0.7, 1.0 ] },
			Vertex { pos: [  0.5,  0.5,  0.5 ], col: [ 0.8, 0.7, 0.7, 1.0 ] },
			Vertex { pos: [  0.5, -0.5,  0.5 ], col: [ 0.8, 0.7, 0.7, 1.0 ] },
			Vertex { pos: [  0.5,  0.5,  0.5 ], col: [ 0.8, 0.7, 0.7, 1.0 ] },
			Vertex { pos: [  0.5, -0.5, -0.5 ], col: [ 0.8, 0.7, 0.7, 1.0 ] },
	
			// Y-
			Vertex { pos: [  0.5, -0.5, -0.5 ], col: [ 0.7, 0.6, 0.7, 1.0 ] },
			Vertex { pos: [ -0.5, -0.5, -0.5 ], col: [ 0.7, 0.6, 0.7, 1.0 ] },
			Vertex { pos: [  0.5, -0.5,  0.5 ], col: [ 0.7, 0.6, 0.7, 1.0 ] },
			Vertex { pos: [ -0.5, -0.5,  0.5 ], col: [ 0.7, 0.6, 0.7, 1.0 ] },
			Vertex { pos: [  0.5, -0.5,  0.5 ], col: [ 0.7, 0.6, 0.7, 1.0 ] },
			Vertex { pos: [ -0.5, -0.5, -0.5 ], col: [ 0.7, 0.6, 0.7, 1.0 ] },
	
			// Y+
			Vertex { pos: [  0.5, 0.5,  0.5 ], col: [ 0.7, 0.8, 0.7, 1.0 ] },
			Vertex { pos: [ -0.5, 0.5, -0.5 ], col: [ 0.7, 0.8, 0.7, 1.0 ] },
			Vertex { pos: [  0.5, 0.5, -0.5 ], col: [ 0.7, 0.8, 0.7, 1.0 ] },
			Vertex { pos: [ -0.5, 0.5, -0.5 ], col: [ 0.7, 0.8, 0.7, 1.0 ] },
			Vertex { pos: [  0.5, 0.5,  0.5 ], col: [ 0.7, 0.8, 0.7, 1.0 ] },
			Vertex { pos: [ -0.5, 0.5,  0.5 ], col: [ 0.7, 0.8, 0.7, 1.0 ] },
			
			// Z-
			Vertex { pos: [ -0.5,  0.5, -0.5 ], col: [ 0.7, 0.7, 0.6, 1.0 ] },
			Vertex { pos: [ -0.5, -0.5, -0.5 ], col: [ 0.7, 0.7, 0.6, 1.0 ] },
			Vertex { pos: [  0.5,  0.5, -0.5 ], col: [ 0.7, 0.7, 0.6, 1.0 ] },
			Vertex { pos: [  0.5, -0.5, -0.5 ], col: [ 0.7, 0.7, 0.6, 1.0 ] },
			Vertex { pos: [  0.5,  0.5, -0.5 ], col: [ 0.7, 0.7, 0.6, 1.0 ] },
			Vertex { pos: [ -0.5, -0.5, -0.5 ], col: [ 0.7, 0.7, 0.6, 1.0 ] },
	
			// Z+
			Vertex { pos: [  0.5,  0.5,  0.5 ], col: [ 0.7, 0.7, 0.8, 1.0 ] },
			Vertex { pos: [ -0.5, -0.5,  0.5 ], col: [ 0.7, 0.7, 0.8, 1.0 ] },
			Vertex { pos: [ -0.5,  0.5,  0.5 ], col: [ 0.7, 0.7, 0.8, 1.0 ] },
			Vertex { pos: [ -0.5, -0.5,  0.5 ], col: [ 0.7, 0.7, 0.8, 1.0 ] },
			Vertex { pos: [  0.5,  0.5,  0.5 ], col: [ 0.7, 0.7, 0.8, 1.0 ] },
			Vertex { pos: [  0.5, -0.5,  0.5 ], col: [ 0.7, 0.7, 0.8, 1.0 ] },
	].iter().cloned()).unwrap()
}
