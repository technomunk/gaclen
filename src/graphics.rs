pub mod context;
pub mod device;
pub mod buffer;
pub mod pipeline;
pub mod shader;

const REQUIRED_VULKAN_VERSION: vulkano::instance::Version = vulkano::instance::Version { major: 1, minor: 0, patch: 0 };

#[derive(Debug)]
pub enum ResizeError {
	Swapchain(vulkano::swapchain::SwapchainCreationError), // Error during recreation of the device swapchian
	UnsizedWindow, // The provided window has no size
}

impl From<vulkano::swapchain::SwapchainCreationError> for ResizeError {
	fn from(err: vulkano::swapchain::SwapchainCreationError) -> ResizeError { ResizeError::Swapchain(err) }
}