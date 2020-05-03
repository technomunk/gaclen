//! Buffers are a region of mapped memory.
//!
//! Gaclen exports vulkano buffers:
//! 
//! - [`CpuAccessibleBuffer`](struct.CpuAccessibleBuffer.html) : simple buffer for quick prototyping.
//! - [`CpuBufferPool`](struct.CpuBufferPool.html) : ring buffer for per-frame data.
//! - [`ImmutableBuffer`](struct.ImmutableBuffer.html) : an idiomatic device-local buffer for read-only data.
//! 
//! # Usage
//! 
//! [`CpuBufferPool`](struct.CpuBufferPool.html) and [`CpuAccessibleBuffer`](struct.CpuAccessibleBuffer.html) can be created directly, using [`Device::logical_device()`](struct.Device.html#method.logical_device):
//! ```
//! let device : gaclen::graphics::Device;
//! let usage = gaclen::graphics::buffer::BufferUsage::all();
//! let buffer = gaclen::graphics::buffer::CpuAccessibleBuffer::uninitialized(device.logical_device(), usage, false);
//! // buffer is ready to use.
//! ```
//! 
//! Device-local buffers (currently only [`ImmutableBuffer`](struct.ImmutableBuffer.html)) require additional initialization (uploading data to the GPU) and can thus be created through helper methods:
//! ```
//! let data : Sized + Send + Sync + 'static;
//! let device : gaclen::graphics::Device;
//! let usage = gaclen::graphics::buffer::BufferUsage::vertex_buffer();
//! let buffer = gaclen::graphics::buffer::create_immutable_buffer_from_data(device, data, usage);
//! // buffer is ready to use.
//! ```

use super::device::Device;

use std::sync::Arc;

use vulkano::buffer::{TypedBufferAccess};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::sync::GpuFuture;
use vulkano::memory::DeviceMemoryAllocError;

pub use vulkano::buffer::{BufferAccess, BufferSlice, BufferUsage, CpuAccessibleBuffer, CpuBufferPool, DeviceLocalBuffer, ImmutableBuffer};

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

/// Create an uninitialized device-local buffer for sized data.
#[inline]
pub fn create_device_local_buffer<T>(device: &Device, usage: BufferUsage) -> Result<Arc<DeviceLocalBuffer<T>>, DeviceMemoryAllocError> {
	DeviceLocalBuffer::new(device.logical_device(), usage, device.device.active_queue_families())
}

/// Create an uninitialized device-local buffer for an array of data.
#[inline]
pub fn create_device_local_array_buffer<T>(device: &Device, len: usize, usage: BufferUsage) -> Result<Arc<DeviceLocalBuffer<[T]>>, DeviceMemoryAllocError> {
	DeviceLocalBuffer::array(device.logical_device(), len, usage, device.device.active_queue_families())
}

/// Write data to a buffer.
/// 
/// Builds a command buffer for writing the data to the buffer and executes it.
/// 
/// # Panic
/// 
/// - Panics if fails to create the command buffer.
/// - Panics if fails to submit the command buffer.
pub fn update<B, D>(device: &Device, buffer: B, data: D)
where
	B : TypedBufferAccess<Content = D> + Send + Sync + 'static,
	D : Send + Sync + 'static,
{
	let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.logical_device(), device.transfer_queue.family()).unwrap()
		.update_buffer(buffer, data).unwrap()
		.build().unwrap();
	
	vulkano::sync::now(device.logical_device())
		.then_execute(device.transfer_queue.clone(), command_buffer).unwrap()
		.flush().unwrap();
}

/// Copies data from one buffer to another.
/// 
/// Builds a command buffer for copying the data and executes it.
/// 
/// # Notes
/// 
/// - The source buffer should have `BufferUsage::transfer_source` set to true.
/// - The destination buffer should have `BufferUsage::transfer_destination` set to true.
/// - If the sizes of buffers are not equal the amount of data copies will be equal to the smaller of the two sizes.
/// 
/// # Panic
/// 
/// - Panics if fails to create the command buffer.
/// - Panics if fails to submit the command buffer.
pub fn copy<S, D, T>(device: &Device, source: S, destination: D)
where
	S : TypedBufferAccess<Content = T> + Send + Sync + 'static,
	D : TypedBufferAccess<Content = T> + Send + Sync + 'static,
	T : ?Sized,
{
	let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.logical_device(), device.transfer_queue.family()).unwrap()
		.copy_buffer(source, destination).unwrap()
		.build().unwrap();
	
	vulkano::sync::now(device.logical_device())
		.then_execute(device.transfer_queue.clone(), command_buffer).unwrap()
		.flush().unwrap();
}
