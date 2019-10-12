# Developer Diary

## 06-10-2019

Greg Created the project.

Decided to name it **gaclen**, short for "Game Client Engine", as it is planned to be used exclusively by the clients (and tools) and not servers.

## 12-10-2019

Greg worked more on the project.

- Created graphics module with context and device submodules.
- Created initializeable graphics::context::Context object, an instance of Vulkan-driver context.
- Created initializeable graphics::device::Device object, the core of using hardware accelerated rendering and computation.
