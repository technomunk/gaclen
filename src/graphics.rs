pub mod context;
pub mod device;
pub mod pass;

pub use vulkano::instance::Version;

const REQUIRED_VULKAN_VERSION: Version = Version { major: 1, minor: 0, patch: 0 };
const ENGINE_NAME: &str = "gaclen";
const ENGINE_VERSION: Version = Version { major: 0, minor: 0, patch: 0 };

#[derive(Debug)]
pub enum ResizeError {
	Swapchain(vulkano::swapchain::SwapchainCreationError), // Error during recreation of the device swapchian
	UnsizedWindow, // The provided window has no size
}

impl From<vulkano::swapchain::SwapchainCreationError> for ResizeError {
	fn from(err: vulkano::swapchain::SwapchainCreationError) -> ResizeError { ResizeError::Swapchain(err) }
}