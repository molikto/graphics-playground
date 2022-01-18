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

#[spirv(vertex)]
pub fn vertex(
    #[spirv(vertex_index)] in_vertex_index: u32,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &ViewUniform,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &MeshUniform,
    position: Vec3,
    _normal: Vec3,
    uv: Vec2,
    #[spirv(position)] out_clip_position: &mut Vec4,
    #[spirv(flat)] out_face: &mut u32,
    out_uv: &mut Vec2,
) {
    let world_position = mesh.transform * position.extend(1.0);
    *out_clip_position = view.view_proj * world_position;
    *out_face = in_vertex_index / 4;
    *out_uv = uv;
}

#[spirv(fragment)]
pub fn fragment(
    #[spirv(frag_coord)] in_frag_coord: Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &ViewUniform,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] map_data: &[usvt],
    #[spirv(flat)] face: u32,
    uv: Vec2,
    output: &mut Vec4,
) {
    let seed = in_frag_coord.xy();
    let mut rng = SRng { seed };

    let total_dim = MySvt::TOTAL_DIM as f32;
    let map_size = Vec3::splat(total_dim);
    let frag_world_position = util::frag_world_position_from_face(from_simulation_coor(map_size), face, uv);

    //render the distance for debug purposes
    // let distance = (frag_world_position - view.world_position).length();
    // *output = Vec3::splat(distance / (total_dim as f32)).extend(1.0);
    // return;

    let ray = Ray3 {
        pos: to_simulation_coor(view.world_position),
        dir: to_simulation_coor(frag_world_position - view.world_position).normalize_or_zero(),
    };

    let map_data = MySvt { mem: map_data };
    let voxel_color = ray::shade_ray(&mut rng, map_data, ray);

    *output = voxel_color.extend(1.0);
}

#[spirv(compute(threads(8, 8)))]
pub fn compute(
    #[spirv(descriptor_set = 0, binding = 0)] view_target: &Image!(2D, format=rgba32f, sampled=false),
    #[spirv(uniform, descriptor_set = 1, binding = 0)] uniform: &VoxelMapRenderUniform,
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] map_data: &[usvt],
    #[spirv(global_invocation_id)] global_ix: UVec3,
) {
    let size: UVec2 = uniform.size;
    let mut frag_coord = vec2(global_ix.x as f32, global_ix.y as f32);
    let seed = frag_coord.xy() + uniform.time;
    let mut rng = SRng { seed };
    frag_coord = frag_coord + rng.gen_vec2();

    let pos = uniform.camera_pos;
    let dir = uniform.camera_look + frag_coord.x * uniform.camera_h + frag_coord.y * uniform.camera_v;
    let ray = Ray3 { pos, dir };

    let map_data = MySvt { mem: map_data };
    let voxel_color = ray::shade_ray(&mut rng, map_data, ray);

    let tex_coor = global_ix.truncate().as_ivec2();
    let prev_color: Vec4 = view_target.read(tex_coor);
    let frame_index = uniform.frame_index as f32;
    let frag_color = (frame_index * prev_color.truncate() + voxel_color) / (frame_index + 1.0);
    let frag_color = frag_color.extend(1.0);
    if global_ix.x < size.x && global_ix.y < size.y {
        unsafe { view_target.write(tex_coor, frag_color) }
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
