/// **Ga**me **Cl**ient **En**gine provides functionality specific to game clients such as:
/// - Rendering using hardware acceleration (uses [vulkano](https://vulkano.rs/))
/// - Handling OS integration, populating window context and processing input (uses [winit](https://docs.rs/winit/))
/// - Processing and playing audio (planned)
/// 
/// The library intentionally does **not** support the following:
/// - **Networking**, as it should be shared between game clients and servers.
/// - **Game logic**, the details of game logic implementation are left up to the using code. [Gaclen](index.html) provides the bases for creating the client for the game.
/// 
/// Note that the library is in active development and most of the features are not supported.

pub mod window;
pub mod graphics;

#[cfg(test)]
mod tests {
#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
