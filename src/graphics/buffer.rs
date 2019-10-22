use super::device::Device;

use std::sync::Arc;

use vulkano::buffer::{CpuAccessibleBuffer};

#[derive(Default, Debug, Clone)]
pub struct Vertex2D {
	position: [f32; 2]
}

vulkano::impl_vertex!(Vertex2D, position);

pub fn triangle(device: &Device) -> Arc<CpuAccessibleBuffer<[Vertex2D]>> {
	CpuAccessibleBuffer::from_iter(device.device.clone(), vulkano::buffer::BufferUsage::all(), [
		Vertex2D { position: [-0.5, 0.5] },
		Vertex2D { position: [0.0, -0.5] },
		Vertex2D { position: [0.5, 0.5] }
	].iter().cloned()).unwrap()
}