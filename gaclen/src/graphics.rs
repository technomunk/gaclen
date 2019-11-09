//! Graphics provide hardware accelerated rendering.
//! 
//! This is a major part of [gaclen](index.html), since rendering is exclusive to clients.
//! 
//! The graphical workflow is extensive, please refer to [examples](https://github.com/Griffone/gaclen/tree/master/examples) for help.

pub mod context;
pub mod device;
pub mod pass;

/// used for hardware acceleration.
pub use vulkano;
pub use vulkano::impl_vertex;
pub use vulkano::instance::Version;

const REQUIRED_VULKAN_VERSION: Version = Version { major: 1, minor: 0, patch: 0 };
const ENGINE_NAME: &str = "gaclen";
// Graphical engine version. Is allowed to differ from gaclen cargo version.
const ENGINE_VERSION: Version = Version { major: 0, minor: 0, patch: 0 };

/// Error during resizing of viewports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResizeError {
	/// Error during recreation of the [Device](struct.Device) swapchain.
	Swapchain(vulkano::swapchain::SwapchainCreationError),
	/// Error during recreation of depth image of the [Device](struct.Device) swapchain.
	Image(vulkano::image::ImageCreationError),
	/// The window provided has no apparent size.
	UnsizedWindow,
}

impl From<vulkano::swapchain::SwapchainCreationError> for ResizeError {
	fn from(err: vulkano::swapchain::SwapchainCreationError) -> ResizeError { ResizeError::Swapchain(err) }
}
impl From<vulkano::image::ImageCreationError> for ResizeError {
	fn from(err: vulkano::image::ImageCreationError) -> ResizeError { ResizeError::Image(err) }
}