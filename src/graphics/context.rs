use vulkano_win;

use std::sync::Arc;

// A Vulkan driver-talking context instance.
pub struct Context {
	pub(super) instance: Arc<vulkano::instance::Instance>
}

impl Context {

	// Create a new instance of Context with all extensions required by gaclen.
	// Will propogate underlying vulkano::instance::InstanceCreationError.
	pub fn new() -> Result<Context, vulkano::instance::InstanceCreationError> {
		let extensions = vulkano_win::required_extensions();
		let instance = vulkano::instance::Instance::new(None, &extensions, None)?;
		let context = Context{ instance };
		Ok(context)
	}
	// TODO: add a version with custom extensions
}

impl Context {

}