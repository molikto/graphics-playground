#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]
mod shared;
pub use shared::*;

pub mod ray;
mod util;



//
// kernels
//

use common::{math::*, shader::base_uniform::*, svt::*};
#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;
use spirv_std::*;

#[spirv(compute(threads(8, 8)))]
pub fn compute(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] uniform: &EnvShaderUniform,
    #[spirv(descriptor_set = 1, binding = 0)] src_tex: &Image!(2D, format=rgba32f, sampled=false),
    #[spirv(descriptor_set = 1, binding = 1)] _desc_tex: &Image!(2D, format=rgba32f, sampled=false),
    #[spirv(storage_buffer, descriptor_set = 1, binding = 2)] svt: &[usvt],
    #[spirv(global_invocation_id)] global_ix: UVec3,
) {
    let size = uniform.size;
    let mut frag_coord = vec2(global_ix.x as f32, global_ix.y as f32);
    frag_coord = frag_coord;

    let pos = uniform.camera_pos;
    let dir = uniform.camera_look + frag_coord.x * uniform.camera_h + frag_coord.y * uniform.camera_v;

    let svt = MySvt { mem: svt };

    let error_code = svt.traverse_ray_oct(pos, dir);
    let count = error_code as f32;
    let env_color = Vec3::splat((count) / 128.0);

    let tex_coor = global_ix.truncate().as_ivec2();
    let frag_color = env_color.extend(1.0);
    if global_ix.x < size.x && global_ix.y < size.y {
        unsafe { src_tex.write(tex_coor, frag_color) }
    }
}

// A simple vert/frag shader to copy an image to the swapchain.
// from https://github.com/googlefonts/compute-shader-101
#[spirv(vertex)]
pub fn compute_vertex(
    #[spirv(vertex_index)] in_vertex_index: u32,
    #[spirv(instance_index)] in_instance_index: u32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
    out_tex_coord: &mut Vec2,
) {
    let x = ((in_vertex_index & 1) ^ in_instance_index) as f32;
    let y = ((in_vertex_index >> 1) ^ in_instance_index) as f32;
    *out_pos = vec4(x * 2. - 1., 1. - y * 2., 0., 1.);
    *out_tex_coord = vec2(x, y);
}

#[spirv(fragment)]
pub fn compute_fragment(
    #[spirv(descriptor_set = 0, binding = 0)] image: &image::Image2d,
    #[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
    in_tex_coord: Vec2,
    output: &mut Vec4,
) {
    *output = image.sample(*sampler, in_tex_coord);
}
