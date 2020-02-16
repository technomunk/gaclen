// TODO/rel: explain buffers

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
