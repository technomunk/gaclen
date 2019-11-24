//! ***Ga**me* ***Cl**ient* ***En**gine* provides functionality specific to game clients such as:
//! - Rendering using hardware acceleration (uses [vulkano](https://vulkano.rs/))
//! - Handling OS integration, populating window context and processing input (uses [winit](https://docs.rs/winit/))
//! - Processing and playing audio (planned)
//! 
//! The library intentionally does **not** support the following:
//! - **Networking**, as it should be shared between game clients and servers.
//! - **Game logic**, the details of game logic implementation are left up to the using code. [Gaclen](index.html) provides the bases for creating the client for the game.
//! 
//! Notes:
//! - The library is in active development and has limited functionality at the moment.
//! - Members exposes with 'expose-underlying-vulkano' feature use [nightly documentation](https://github.com/rust-lang/rust/issues/43466). The links will be broken.
//! - The examples use sister-project: [gaclen_shader](https://crates.io/crates/gaclen_shader).

pub mod window;
pub mod graphics;