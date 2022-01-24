#![cfg_attr(all(target_arch = "spirv", not(test)), no_std)]

use common::{*, svt::*};

pub mod sim;

// pub const BLOCK_DIM: usvt = 2;
// pub const LEVEL_COUNT: usize = 10;
pub const BLOCK_DIM: usvt = 2;
pub const LEVEL_COUNT: usize = 6;
// pub const BLOCK_DIM: usvt = 4;
// pub const LEVEL_COUNT: usize = 5;

pub type MySvt<'a>  = Svt<sim::VoxelData, &'a [usvt], BLOCK_DIM, LEVEL_COUNT>;

#[cfg(not(target_arch = "spirv"))]
pub type MySvtMut  = SvtMut<sim::VoxelData, BLOCK_DIM, LEVEL_COUNT>;