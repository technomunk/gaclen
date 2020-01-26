//! Context holds global Vulkan API state information.

use std::sync::Arc;
use std::borrow::Cow;

use vulkano::instance::{Instance, InstanceCreationError, ApplicationInfo, Version};
use vulkano::instance::InstanceExtensions;

use super::{ENGINE_NAME, ENGINE_VERSION};

/// An instance of graphical context.
/// 
/// It holds global Vulkan API state information.
pub struct Context {
	pub(super) instance: Arc<Instance>
}

impl Context {
	/// Create a new instance of Context.
	/// 
	/// Will use blank application name and version.
	pub fn new() -> Result<Context, InstanceCreationError> { Context::create(None, None, vulkano_win::required_extensions()) }

	/// Create a new instance of Context with an application name and version.
	/// 
	/// This will allow for potential driver-side optimizations specific to your application.
	pub fn with_app_info(name: &str, version: Version) -> Result<Context, InstanceCreationError> { Context::create(Some(name), Some(version), vulkano_win::required_extensions()) }

	// TODO: add a version with custom extensions
}

#[cfg(feature = "expose-underlying-vulkano")]
impl Context {
	/// Get the underlying [vulkano instance](Instance).
	#[inline(always)]
	pub fn instance(&self) -> &Arc<Instance> { self.instance }
}

impl Context {
	fn create(
		application_name: Option<&str>,
		application_version: Option<Version>,
		extensions: InstanceExtensions
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
		let instance = Instance::new(Some(&app_info), &extensions, None)?;
		Ok(Context { instance })
	}
}
