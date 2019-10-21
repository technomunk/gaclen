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