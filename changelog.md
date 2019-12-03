# Changelog

## 0.0.7 Textured Cube

- **!IMPORTANT!** removes implicit viewport transformation, this results in:
  - flipped y-screenspace direction
  - default depth now works as expected
- **!BREAKING CHANGE!** device.draw() now requires a descriptor_set
- adds basic_ and inverse_ depth tests to GraphicalPassBuilder
- introduces descriptor_sets
- adds phong_cube example

## 0.0.6 Building Passes in Frames

- **!BREAKING CHANGE!** refactors Drawing device into 2 sub-states:
  - *Frame* - active frame
  - *PassInFrame* - an active graphical pass within a frame
- **!BREAKING CHANGE!** refactors GraphicalPass to be struct and not a trait
- changes the example to handle failing resizing device by skipping a frame
- changes the example to work with breaking changes
- updates *'vulkano'* dependency to 0.16.0
- moves re-exports to sub-projects
  - vulkano is now exported by gaclen::graphics
  - winit is now exported by gaclen::window
  - gaclen::window also directly exports winit items
- creates a split gaclen_shader project that re-exports a tweaked version of vulkano_shader! macro
  - this drops the necessity of depending on vulkano
  - vulkano can be used from gaclen::graphics::vulkano
- creates GraphicalPassBuilder for initializing a GraphicalPass

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
