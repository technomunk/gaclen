# Changelog

## 0.0.10

- **!IMPORTANT!** adds initial support for generic attachments
- **!BREAKING CHANGE!** refactors attachments in `GraphicalPass` creation:
  - `add_attachment_*()` are now broken into separate `add_image_attachment()` and `add_depth_attachment()`
  - swapchain is now just a special case of the above method call

## 0.0.9 Unclear Passes

- **!IMPORTANT!** adds `GraphicalPassBuilder::add_attachment*()` functionality
  - allows to set user-defined load and store operators
  - unify workflow for using swapchain and user-declared image resources
- **!BREAKING CHANGE!** refactors `GraphicalPass`:
  - gets rid of 2 generic parameters (descriptor and present-pass markers)
  - forces the user to create framebuffers
- **!BREAKING CHANGE!** `GraphicalPassBuilder::build_present_pass()` is now just `GraphicalPassBuilder::build()`
- **!BREAKING CHANGE!** `Frame::begin_pass()` now takes a framebuffer argument
- adds `GraphicalPass::begin_framebuffer()` method to start building a framebuffer *(see examples for help)*
- adds `Frame::get_swapchain_image()` and `Frame::get_swapchain_depth()` accessors to current images for building framebuffers
- adds `Device::with_depth_format()` constructor for specifying custom swapchain depth format
- minorly improves documentation
- updates project readme

## 0.0.8 Mutated Immutable Docs

- fixes and improves documentation for `Device`:
  - fixes broken links
  - extends some method documentation
  - lists panic scenarios for some methods
- adds device-local `ImmutableBuffer` functionality
- adds `PassInFrame::draw_indexed()` functionality
- `PassInFrame::draw()` methods should now have static-dispatch

## 0.0.7 Textured Cube

- **!IMPORTANT!** removes implicit viewport transformation, this results in:
  - flipped y-screenspace direction
  - default depth now works as expected
- **!BREAKING CHANGE!** `Device::draw()` now requires a descriptor_set
- **!BREAKING CHANGE!** renames `Device::create_buffer()` to `Device::create_cpu_accessible_buffer()`
- adds basic_ and inverse_ depth tests to `GraphicalPassBuilder`
- adds descriptor sets
  - adds `GraphicalPass::start_persistent_descriptor_set()`
- adds image support
  - adds `Device::create_immutable_image_from_iter()`
  - adds `Device::create_sampler()` and `Device::create_simple_linear_repeat_sampler()`
- adds phong_cube example

## 0.0.6 Building Passes in Frames

- **!BREAKING CHANGE!** refactors Drawing device into 2 sub-states:
  - *Frame* - active frame
  - *PassInFrame* - an active graphical pass within a frame
- **!BREAKING CHANGE!** refactors `GraphicalPass` to be struct and not a trait
- changes the example to handle failing resizing device by skipping a frame
- changes the example to work with breaking changes
- updates *'vulkano'* dependency to 0.16.0
- moves re-exports to sub-projects
  - vulkano is now exported by `gaclen::graphics`
  - winit is now exported by `gaclen::window`
  - `gaclen::window` also directly exports winit items
- creates a split gaclen_shader project that re-exports a tweaked version of vulkano_shader! macro
  - this drops the necessity of depending on vulkano
  - vulkano can be used from `gaclen::graphics::vulkano`
- creates `GraphicalPassBuilder` for initializing a `GraphicalPass`

## 0.0.5 PresentMode

- **!BREAKING CHANGE!** allows different present modes for device swapchain
- re-exports vulkano-shader of compatible version with used vulkano

## 0.0.4 First 'Feature'

- introduces 'expose-underlying-vulkano' feature that exposes vulkano members of gaclen structs to allow for more flexibility
- re-exports vulkano and winit
- minorly improves documentation
- fixes spelling in the source code and documentation

## 0.0.3 Improved Documentation

- adds 'lessons.md' file with learned lessons
- improves documentation, making it a lot more appealing

## 0.0.2 Initial Documentation

- adds licenses for the project
- introduces documentation

## 0.0.1 First Publication

- first release
