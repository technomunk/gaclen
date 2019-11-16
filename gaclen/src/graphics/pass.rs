//! Infrastructure for interpreting and computing data.
//! 
//! Example passes are:
//! - **Shadow** - drawing a scene from the point of view of a light source in order to save depth information.
//! - **Albedo** - drawing typically-represented geometry with lighting and optional shading.
//! - **Post-process** - screen-space based techniques for processing image before presenting it on the screen.

pub mod graphical_pass;
pub mod builder;

pub use graphical_pass::*;
pub use builder::GraphicalPassBuilder;