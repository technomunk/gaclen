//! Infrastructure containing device configuration used for computations.

mod graphical_pass;
mod builder;

pub use graphical_pass::*;
pub use builder::{GraphicalPassBuilder, PrimitiveTopology, StoreOp, LoadOp};
pub use vulkano::descriptor::descriptor_set::{FixedSizeDescriptorSet, FixedSizeDescriptorSetsPool};
