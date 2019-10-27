# Developer Diary

## 06-10-2019

Greg Created the project.

Decided to name it **gaclen**, short for **Ga**me **Cl**ient **En**gine, as it is planned to be used exclusively by the clients (and tools) but *not* servers.

## 12-10-2019

Greg worked more on the project.

- Created graphics module with context and device submodules.
- Created initializeable graphics::context::Context object, an instance of Vulkan-driver context.
- Created initializeable graphics::device::Device object, the core of using hardware accelerated rendering and computation.

## 19-10-2019

Greg made rendered triangle.

- Created graphics::pipeline::Pass struct.
- Created the functionality to record command buffers.
- Created the functionality to submit command buffers and present.

## 28-10-2019

Greg published [gaclen](https://crates.io/crates/gaclen).

- Reworked rendering loop to feel nicer.
- Learned how to publish a crate.
- Learned about rustdoc.
- Documented a bunch of code.

## 29-10-2019

Greg imroved documentation, learning a bunch of lessons.

- Improved documentation.
- Created a [lessons](lessons.md) file that lists learned lessons and ideas from this project.
