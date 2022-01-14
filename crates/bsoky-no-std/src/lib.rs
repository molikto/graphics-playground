#![cfg_attr(all(target_arch = "spirv", not(test)), no_std)]

use common_no_std::{*, svt::*};

// pub const BLOCK_DIM: usvt = 2;
// pub const LEVEL_COUNT: usize = 10;
pub const BLOCK_DIM: usvt = 4;
pub const LEVEL_COUNT: usize = 3;

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

// TODO change order of things in this will make it dont work???!!!
#[cfg_attr(not(target_arch = "spirv"), derive(bevy_crevice::std430::AsStd430))]
#[cfg_attr(not(target_arch = "spirv"), derive(bevy_crevice::std140::AsStd140))]
pub struct EnvShaderUniform {
    pub camera_transform: Mat4,
    pub inverse_view: Mat4,
    pub size: UVec2,
    pub time: f32,
    pub frame_index: u32,
}