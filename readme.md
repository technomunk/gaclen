# **Ga**me **Cl**ient **En**gine

A full-stack 3D game client engine in [Rust](https://www.rust-lang.org).

## Setup

The gaclen_shader project relies on [shaderc-rs](https://github.com/google/shaderc-rs) which requires [additional setup steps](https://github.com/google/shaderc-rs#setup). Please follow them.

## Supported features

- [x] Unlit 3D object rendering
- [ ] Lit 3D object rendering
- [ ] Semi-transparent object rendering
- [ ] Text rendering
- [ ] Extended debug information and pipeline
- [ ] Forward kinematics
- [ ] Shadows
- [ ] Water effects
- [ ] Post-effects
- [ ] Audio
- [ ] Particle effects
- [ ] Input processing

## Unsupported features

These features will not be supported, since they make up a game, or should be common between client and server.

- Networking
- Game logic

## Potential future features

- [ ] Limited physics simulation
- [ ] Inverse kinematics
- [ ] Forward rendering
