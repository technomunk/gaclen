//! Buffers are a region of mapped memory.
//!
//! Gaclen exports vulkano buffers:
//! 
//! - [`CpuAccessibleBuffer`](CpuAccessibleBuffer) : simple buffer for quick prototyping.
//! - [`CpuBufferPool`](CpuBufferPool) : ring buffer for per-frame data.
//! - [`ImmutableBuffer`](ImmutableBuffer) : an idiomatic device-local buffer for read-only data.
//! 
//! # Usage
//! 
//! [`CpuBufferPool`](CpuBufferPool) and [`CpuAccessibleBuffer`](CpuAccessibleBuffer) can be created directly, using [`Device::logical_device()`](Device::logical_device):
//! ```
//! let device : gaclen::graphics::Device;
//! let usage = gaclen::graphics::buffer::BufferUsage::all();
//! let buffer = gaclen::graphics::buffer::CpuAccessibleBuffer::uninitialized(device.logical_device(), usage, false);
//! // buffer is ready to use.
//! ```
//! 
//! Device-local buffers (currently only [`ImmutableBuffer`](ImmutableBuffer)) require additional initialization (uploading data to the GPU) and can thus be created through helper methods:
//! ```
//! let data : Sized + Send + Sync + 'static;
//! let device : gaclen::graphics::Device;
//! let usage = gaclen::graphics::buffer::BufferUsage::vertex_buffer();
//! let buffer = gaclen::graphics::buffer::create_immutable_buffer_from_data(device, data, usage);
//! // buffer is ready to use.
//! ```

use super::device::Device;

use std::sync::Arc;

use vulkano::sync::GpuFuture;
use vulkano::memory::DeviceMemoryAllocError;

pub use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool, ImmutableBuffer};

/// Create a device-local immutable buffer from some data.
/// 
/// Builds an intermediate memory-mapped buffer, writes data to it, builds a copy (upload) command buffer and executes it.
/// 
/// # Panic.
/// 
/// - Panics if fails to submit the copy command buffer.
pub fn create_immutable_buffer_from_data<T>(device: &Device, data: T, usage: BufferUsage) -> Result<Arc<ImmutableBuffer<T>>, DeviceMemoryAllocError>
where
	T : Send + Sync + Sized + 'static,
{
	let (buffer, future) = ImmutableBuffer::from_data(data, usage, device.transfer_queue.clone())?;

	// TODO: handle synchronization between separate queues in a performant way
	future.flush().unwrap();

	Ok(buffer)
}

/// Create a device-local immutable buffer from some data iterator.
/// 
/// Builds an intermediate memory-mapped buffer, writes data to it, builds a copy (upload) command buffer and executes it.
/// 
/// # Panic.
/// 
/// - Panics if fails to submit the copy command buffer.
pub fn create_immutable_buffer_from_iter<T>(device: &Device, data_iterator: impl ExactSizeIterator<Item = T>, usage: BufferUsage) -> Result<Arc<ImmutableBuffer<[T]>>, DeviceMemoryAllocError>
where
	T : Send + Sync + Sized + 'static,
{
	let (buffer, future) = ImmutableBuffer::from_iter(data_iterator, usage, device.transfer_queue.clone())?;

	// TODO: handle synchronization between separate queues in a performant way
	future.flush().unwrap();

	Ok(buffer)
}
