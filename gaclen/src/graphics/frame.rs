// TODO/rel: Explain how frames are drawn.

use super::device::Device;
use super::pass::GraphicalPass;
use super::swapchain::Swapchain;

use crate::window::Window;

use std::sync::Arc;

use vulkano::buffer::{BufferAccess, TypedBufferAccess};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferExecError, DynamicState};
use vulkano::descriptor::descriptor_set::DescriptorSetsCollection;
use vulkano::framebuffer::FramebufferAbstract;
use vulkano::sync::{GpuFuture, FlushError};
use vulkano::swapchain::{Swapchain as VlkSwapchain};
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::pipeline::input_assembly::Index;
use vulkano::pipeline::vertex::VertexSource;

/// A frame in the process of being drawn.
pub struct Frame {
	pub(super) device: Device,
	pub(super) swapchain: Arc<VlkSwapchain<Arc<Window>>>,
	pub(super) time: Box<dyn GpuFuture>,
	pub(super) dynamic_state: DynamicState,
	pub(super) commands: AutoCommandBufferBuilder,
	// index of the frame in the swapchain
	pub(super) swapchain_index: usize,
}

/// A frame in the process of being drawn using a given [GraphicalPass].
pub struct PassInFrame<'a, P : ?Sized> {
	pub(super) frame: Frame,
	pub(super) pass: &'a GraphicalPass<P>,
}

/// Error finishing the frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameFinishError {
	/// Error during flushing commands to the GPU.
	Flush(FlushError),
	/// Error during attempted execution of GPU commands.
	Commands(CommandBufferExecError),
}

impl Frame {
	/// Begin drawing a frame.
	/// 
	/// - Locks down the Device for the drawing process (consuming it for the duration of the frame).
	/// - Acquires the swapchain image to draw to.
	/// - Creates a CommandBuffer to be recorded for the frame.
	/// NOTE: that to actually draw, [Frame::begin_pass] needs to be called.
	pub fn begin(
		mut device: Device,
		swapchain: &Swapchain,
	) -> Result<Frame, (Device, vulkano::swapchain::AcquireError)>
	{
		let used_swapchain = swapchain.swapchain.clone();

		let (swapchain_index, image_acquire_time) = match vulkano::swapchain::acquire_next_image(used_swapchain.clone(), None) {
			Ok(result) => result,
			Err(err) => return Err((device, err)),
		};

		let time: Box<dyn GpuFuture> = match device.before_frame.take() {
			Some(mut time) => {
				time.cleanup_finished();
				Box::new(time.join(image_acquire_time))
			},
			None => Box::new(vulkano::sync::now(device.logical_device()).join(image_acquire_time)),
		};

		let commands = AutoCommandBufferBuilder::primary_one_time_submit(device.logical_device(), device.graphics_queue.family()).unwrap();

		let frame = Frame {
			device,
			swapchain: used_swapchain,
			dynamic_state: swapchain.dynamic_state.clone(),
			time,
			commands,
			swapchain_index,
		};
		Ok(frame)
	}

	/// Begin a PresentPass (the results will be visible on the screen).
	/// 
	/// # Panic.
	/// 
	/// - Panics if fails to begin the [renderpass](https://vulkan.lunarg.com/doc/view/1.0.37.0/linux/vkspec.chunked/ch07.html) command.
	pub fn begin_pass<'a, P : ?Sized, F>(
		mut self,
		pass: &'a GraphicalPass<P>,
		framebuffer: F,
		clear_values: Vec<vulkano::format::ClearValue>)
	-> PassInFrame<'a, P>
	where
		F : FramebufferAbstract + Send + Sync + Clone + 'static,
	{
		// TODO: build framebuffer automatically, using GraphicalRenderPassDescriptor information

		self.commands = self.commands.begin_render_pass(framebuffer, false, clear_values).unwrap();

		PassInFrame {
			frame: self,
			pass: pass,
		}
	}

	/// Finish drawing the frame and flush the commands to the GPU.
	/// 
	/// Releases the Device to allow starting a new frame, allocate new resources and anything else a [Device] is able to do.
	/// 
	/// # Panic.
	/// 
	/// - Panics if fails to build (finalize) the command buffer.
	#[inline]
	pub fn finish(self) -> Result<Device, (Device, FrameFinishError)> {
		let commands = self.commands.build().unwrap();
		let after_execute = match self.time.then_execute(self.device.graphics_queue.clone(), commands) {
			Ok(future) => future,
			Err(err) => return Err((self.device, FrameFinishError::Commands(err))),
		};

		let after_flush = after_execute.then_swapchain_present(self.device.graphics_queue.clone(), self.swapchain, self.swapchain_index)
			.then_signal_fence_and_flush();
		
		let after_frame = match after_flush {
			Ok(future) => future,
			Err(err) => return Err((self.device, FrameFinishError::Flush(err))),
		};
		let device = Device { before_frame: Some(Box::new(after_frame)), .. self.device };
		Ok(device)
	}
}


impl<'a, P> PassInFrame<'a, P>
where
	P : GraphicsPipelineAbstract + Send + Sync + ?Sized + 'static,
{
	// TODO: non-polymorphic vertex_buffer drawing

	/// Draw some data using a pass.
	/// 
	/// The result depends highly on the [GraphicalPass](traits.GraphicalPass.html) that was used to create the [PassInFrame].
	/// Push-constants should correspond to the ones in the shader used for creating the [GraphicalPass](traits.GraphicalPass.html).
	/// 
	/// # Panic.
	/// 
	/// - Panics if fails to write draw commands to the command buffer.
	#[inline]
	pub fn draw<VB, DSC, PC>(
		mut self,
		vertex_buffer: VB,
		descriptor_sets: DSC,
		push_constants: PC
	) -> Self
	where
		P : VertexSource<VB>,
		DSC : DescriptorSetsCollection,
	{
		self.frame.commands = self.frame.commands.draw(self.pass.pipeline.clone(), &self.frame.dynamic_state, vertex_buffer, descriptor_sets, push_constants).unwrap();
		self
	}

	/// Draw some indexed vertex data using a pass.
	/// 
	/// The result depends highly on the [GraphicalPass](traits.GraphicalPass.html) that was used to create the [PassInFrame].
	/// Push-constants should correspond to the ones in the shader used for creating the [GraphicalPass](traits.GraphicalPass.html).
	/// 
	/// # Panic.
	/// 
	/// - Panics if fails to write draw commands to the command buffer.
	#[inline]
	pub fn draw_indexed<VB, IB, DSC, PC, I>(
		mut self,
		vertex_buffer: VB,
		index_buffer: IB,
		descriptor_sets: DSC,
		push_constants: PC
	) -> Self
	where
		P : VertexSource<VB>,
		DSC : DescriptorSetsCollection,
		IB : BufferAccess + TypedBufferAccess<Content = [I]> + Send + Sync + 'static,
		I : Index + 'static,
	{
		self.frame.commands = self.frame.commands.draw_indexed(self.pass.pipeline.clone(), &self.frame.dynamic_state, vertex_buffer, index_buffer, descriptor_sets, push_constants).unwrap();
		self
	}

	/// Finish using a GraphicalPass.
	/// 
	/// Releases the consumed [Frame] to begin the next pass or finish the frame.
	/// 
	/// # Panic.
	/// 
	/// - Panics if fails to write end [renderpass](https://vulkan.lunarg.com/doc/view/1.0.37.0/linux/vkspec.chunked/ch07.html) command to the command buffer.
	#[inline]
	pub fn finish_pass(self) -> Frame {
		let commands = self.frame.commands.end_render_pass().unwrap();
		Frame { commands, .. self.frame }
	}
}
