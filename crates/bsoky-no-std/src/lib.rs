#![cfg_attr(not(test), no_std)]

use common_no_std::{*, svo::*};

pub const BLOCK_DIM: usvo = 2;
pub const LEVEL_COUNT: usize = 8;

pub type MySvo<'a>  = Svo<&'a [usvo], BLOCK_DIM, LEVEL_COUNT>;
pub type MySvoMut<'a>  = Svo<&'a mut [usvo], BLOCK_DIM, LEVEL_COUNT>;

pub const BLOCK_RENDER_SIZE: f32 = 1.0;//;0.25;

pub fn to_simulation_coor(v: Vec3) -> Vec3 {
    return v;
    //Vec3::new(v[0] / BLOCK_RENDER_SIZE, v[2] / BLOCK_RENDER_SIZE, v[1] / BLOCK_RENDER_SIZE)
}

pub fn from_simulation_coor(v: Vec3) -> Vec3 {
    return v;
    //Vec3::new(v[0] * BLOCK_RENDER_SIZE, v[2] * BLOCK_RENDER_SIZE, v[1] * BLOCK_RENDER_SIZE)
}
