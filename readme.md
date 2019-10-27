# **Ga**me **Cl**ient **En**gine

A full-stack 3D game client engine in [Rust](https://www.rust-lang.org).

## Setup

1. Add "gaclen" of appropriate version to dependencies of your project (in Cargo.toml file).
2. Additionally follow [vulkano-shaders](https://github.com/vulkano-rs/vulkano#setup) setup steps, as the project depends on that package.

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
