use std::sync::Arc;
use std::borrow::Cow;

use vulkano::instance::{Instance, InstanceCreationError, ApplicationInfo, Version};
use vulkano::instance::InstanceExtensions;

use super::{ENGINE_NAME, ENGINE_VERSION};

// A Vulkan driver-talking context instance.
pub struct Context {
	pub(super) instance: Arc<Instance>
}

impl Context {

	// Create a new instance of Context with all extensions required by gaclen.
	// Will propogate underlying vulkano::instance::InstanceCreationError.
	pub fn new() -> Result<Context, InstanceCreationError> { Context::create(None, None, vulkano_win::required_extensions()) }

	// Provide a custom application name and a version to the Instance.
	// This will allow for driver-side optimizations specific to your application.
	pub fn with_app_info(name: &str, version: Version) -> Result<Context, InstanceCreationError> { Context::create(Some(name), Some(version), vulkano_win::required_extensions()) }

	// TODO: add a version with custom extensions
}

impl Context {
	fn create(
		application_name: Option<&str>,
		application_version: Option<Version>,
		extenshions: InstanceExtensions
	) -> Result<Context, InstanceCreationError> {
		let application_name: Option<Cow<str>> = match application_name {
			Some(name) => Some(Cow::from(name)),
			None => None,
		};
		let app_info = ApplicationInfo {
			application_name,
			application_version,
			engine_name: Some(Cow::from(ENGINE_NAME)),
			engine_version: Some(ENGINE_VERSION),
		};
		let instance = Instance::new(Some(&app_info), &extenshions, None)?;
		Ok(Context { instance })
	}
}