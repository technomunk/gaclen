# Developer Diary

## 06-10-2019

Greg Created the project.

Decided to name it **gaclen**, short for **Ga**me **Cl**ient **En**gine, as it is planned to be used exclusively by the clients (and tools) but *not* servers.

## 12-10-2019

Greg worked more on the project.

- Created graphics module with context and device submodules.
- Created initializable graphics::context::Context object, an instance of Vulkan-driver context.
- Created initializable graphics::device::Device object, the core of using hardware accelerated rendering and computation.

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

Greg improved documentation, learning a bunch of lessons.

- Improved documentation.
- Created a [lessons](lessons.md) file that lists learned lessons and ideas from this project.

## 02-11-2019

Greg introduced changelog and 'expose-underlying-vulkano' feature.
Since the changelog this diary is reserved for personal entries in free form.

## 16-02-2020

Greg split the `Device` into smaller modules related to the relevant functionality, such as `swapchain` or `buffer`.
