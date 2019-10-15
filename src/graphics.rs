pub mod context;
pub mod device;
pub mod buffer;
pub mod pipeline;
pub mod shader;

const REQUIRED_VULKAN_VERSION: vulkano::instance::Version = vulkano::instance::Version { major: 1, minor: 0, patch: 0 };