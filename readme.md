# **Ga**me **Cl**ient **En**gine

A full-stack 3D game client engine in [Rust](https://www.rust-lang.org).

## Setup

The gaclen_shader project relies on [shaderc-rs](https://github.com/google/shaderc-rs) which requires [additional setup steps](https://github.com/google/shaderc-rs#setup). Please follow them.

## What it does

Gaclen accelerates game client creation through:

- Providing a thin comfort layer for structuring one's rendering logic on top of [Vulkano](https://vulkano.rs/).

It also enforces minimal predetermined ideas, like scene organization, leaving it to higher level logic, allowing it to be more specialized for a specific game's need.

## What it is planned to do

In the (hopefully) near future Gaclen will:

- Provide an intermediate input layer, that organized different possible input in a portable and robust way.
- Provide an intermediate audio layer, allowing playing and processing audio in a portable way.
- Provide text utilities, including font loading, glyph generation, layout and rendering.

## What it might do

These features are not currently planned, but might be implemented in the future:

- Skeletal animation.
- Inverse kinematics.
- Vulkan-specific linear algebra.

## What id does NOT do

These features will not be supported, since they make up a game, or should be common between client and server.

- Networking.
- Game logic.
