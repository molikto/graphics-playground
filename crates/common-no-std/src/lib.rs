#![cfg_attr(not(test), no_std)]

pub mod lang;
pub use lang::*;
pub mod math;
pub use math::*;
pub mod graphics;
pub use graphics::*;
pub mod shader;