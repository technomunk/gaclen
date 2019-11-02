//! Graphics provide hardware accelerated rendering.
//! 
//! This is a major part of [gaclen](index.html), since rendering is exclusive to clients.
//! 
//! The graphical workflow is extensive, please refer to [examples](https://github.com/Griffone/gaclen/tree/master/examples) for help.

pub mod context;
pub mod device;
pub mod pass;

pub use vulkano::instance::Version;

const REQUIRED_VULKAN_VERSION: Version = Version { major: 1, minor: 0, patch: 0 };
const ENGINE_NAME: &str = "gaclen";
const ENGINE_VERSION: Version = Version { major: 0, minor: 0, patch: 0 };

/// Error during resizing of viewports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResizeError {
	/// Error during recreation of the [Device](struct.Device) swapchain
	Swapchain(vulkano::swapchain::SwapchainCreationError),
	/// The window provided has no apparent size
	UnsizedWindow,
}

impl From<vulkano::swapchain::SwapchainCreationError> for ResizeError {
	fn from(err: vulkano::swapchain::SwapchainCreationError) -> ResizeError { ResizeError::Swapchain(err) }
}