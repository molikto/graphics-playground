#![cfg_attr(all(target_arch = "spirv", not(test)), no_std, feature(asm))]
#![feature(asm_experimental_arch)]

pub mod lang;
#[cfg(not(target_arch = "spirv"))]
mod lang_std;
pub use lang::*;
pub mod math;
pub use math::*;
pub mod graphics;
pub use graphics::*;
pub mod shader;


// legacy code

// #[cfg(not(target_arch = "spirv"))]
// pub mod geometry;