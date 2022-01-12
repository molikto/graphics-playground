#![cfg_attr(all(target_arch = "spirv", not(test)), no_std)]

use common_no_std::{*, svt::*};

pub const BLOCK_DIM: usvt = 2;
pub const LEVEL_COUNT: usize = 10;
// pub const BLOCK_DIM: usvt = 4;
// pub const LEVEL_COUNT: usize = 5;

#[cfg(target_arch = "spirv")]
pub type MySvt<'a>  = Svt<&'a [usvt], BLOCK_DIM, LEVEL_COUNT>;

#[cfg(not(target_arch = "spirv"))]
pub type MySvtMut  = SvtMut<BLOCK_DIM, LEVEL_COUNT>;

pub const BLOCK_RENDER_SIZE: f32 = 1.0;//;0.25;

pub fn to_simulation_coor(v: Vec3) -> Vec3 {
    return v;
    //Vec3::new(v[0] / BLOCK_RENDER_SIZE, v[2] / BLOCK_RENDER_SIZE, v[1] / BLOCK_RENDER_SIZE)
}

pub fn from_simulation_coor(v: Vec3) -> Vec3 {
    return v;
    //Vec3::new(v[0] * BLOCK_RENDER_SIZE, v[2] * BLOCK_RENDER_SIZE, v[1] * BLOCK_RENDER_SIZE)
}
