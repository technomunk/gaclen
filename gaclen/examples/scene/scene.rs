//! Logic related to managing multiple objects that are drawn in a non-trivial way.

pub use cgmath::{Matrix4, Quaternion, Vector3, One, Zero};

/// Full transformation that may be applied to an object.
#[derive(Debug)]
pub struct Transform {
	pub rotation: Quaternion<f32>,
	pub scaling: Vector3<f32>,
	pub position: Vector3<f32>,
}

impl Transform {
	pub fn compute_matrix(self) -> Matrix4<f32> {
		let position_scale_matrix = Matrix4::new(
			self.scaling.x, 0.0, 0.0, self.position.x,
			0.0, self.scaling.y, 0.0, self.position.y,
			0.0, 0.0, self.scaling.z, self.position.z,
			0.0, 0.0, 0.0, 1.0
		);
		let rotation_matrix: Matrix4<f32> = self.rotation.into();
		position_scale_matrix * rotation_matrix
	}
}

impl std::default::Default for Transform {
	fn default() -> Self {
		Self {
			rotation: Quaternion::one(),
			scaling: Vector3{ x: 1.0, y: 1.0, z: 1.0 },
			position: Vector3::zero(),
		}
	}
}

impl std::convert::Into<[[f32; 4]; 4]> for Transform {
	fn into(self) -> [[f32; 4]; 4] {
		// TODO: try to circumvent matrix construction and construct the arrays directly
		self.compute_matrix().into()
	}
}

pub struct Object {
	// TODO: figure out referencing a model

}
