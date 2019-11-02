# Changelog

## 0.0.6 Resizing Frames

- **!BREAKING CHANGE!** refactors Drawing device into 2 sub-states:
  - *Frame* - active frame
  - *PassInFrame* - an active graphical pass within a frame
- changes the example to handle failing resizing device by skipping a frame
- fixes continuous memory leak in the quad example

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
