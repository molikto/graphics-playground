#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

use bsoky_no_std::*;
use common_no_std::{math::*, shader::base_uniform::*, svt::*};
#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;
use spirv_std::num_traits::Float;
use spirv_std::*;

mod ray;
mod voxel;

#[spirv(vertex)]
pub fn vertex(
    #[spirv(vertex_index)] in_vertex_index: u32,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &ViewUniform,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &MeshUniform,
    position: Vec3,
    normal: Vec3,
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

pub fn frag_world_position_from_face(map_size: Vec3, face: u32, uv: Vec2) -> Vec3 {
    // TODO don't branch that much
    let mut frag_world_position = vec3(0.0, 0.0, 0.0);
    if face == 0 {
        // max z side
        frag_world_position = vec3(uv.x * map_size.x, uv.y * map_size.y, map_size.z);
    } else if face == 1 {
        // min z side
        frag_world_position = vec3((1.0 - uv.x) * map_size.x, (1.0 - uv.y) * map_size.y, 0.0);
    } else if face == 2 {
        // max x side
        frag_world_position = vec3(map_size.x, uv.x * map_size.y, uv.y * map_size.z);
    } else if face == 3 {
        // min x side
        frag_world_position = vec3(0.0, (1.0 - uv.x) * map_size.y, (1.0 - uv.y) * map_size.z);
    } else if face == 4 {
        // max y side
        frag_world_position = vec3(uv.x * map_size.x, map_size.y, uv.y * map_size.z);
    } else if face == 5 {
        // min y side
        frag_world_position = vec3((1.0 - uv.x) * map_size.x, 0.0, (1.0 - uv.y) * map_size.z);
    }
    return frag_world_position;
}

#[spirv(fragment)]
pub fn fragment(
    #[spirv(frag_coord)] in_frag_coord: Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &ViewUniform,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] svt: &[usvt],
    #[spirv(flat)] face: u32,
    uv: Vec2,
    output: &mut Vec4,
) {
    let svt = MySvt { mem: svt };
    let total_dim = MySvt::TOTAL_DIM as f32;
    let map_size = Vec3::splat(total_dim);
    let frag_world_position =
        frag_world_position_from_face(from_simulation_coor(map_size), face, uv);

    //render the distance for debug purposes
    // let distance = (frag_world_position - view.world_position).length();
    // *output = Vec3::splat(distance / (total_dim as f32)).extend(1.0);
    // return;

    let seed = in_frag_coord.xy();
    let mut rng = SRng { seed };

    let mut ray = Ray3 {
        pos: to_simulation_coor(view.world_position),
        dir: to_simulation_coor(frag_world_position - view.world_position).normalize_or_zero(),
    };
    let env_color = ray::shade_ray(&mut rng, svt, ray);

    // let color = clamp(env_color, Vec4(0.0), Vec4(1.0));
    // let color = color_over(env_color, Vec4(0.5, 0.5, 0.5, 1.0));
    *output = env_color.extend(1.0);
    // no need for gamma correction
    //return Vec4(pow(env_color, Vec3(1.0/2.2)), 1.0);
}


#[spirv(compute(threads(8, 8)))]
pub fn compute(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] uniform: &EnvShaderUniform,
    #[spirv(descriptor_set = 1, binding = 0)] src_tex: &Image!(2D, format=rgba32f, sampled=false),
    #[spirv(descriptor_set = 1, binding = 1)] desc_tex: &Image!(2D, format=rgba32f, sampled=false),
    #[spirv(storage_buffer, descriptor_set = 1, binding = 2)] svt: &[usvt],
    #[spirv(global_invocation_id)] global_ix: UVec3,
) {
    let size = uniform.size;
    if global_ix.x < size.x && global_ix.y < size.y {
        let mut frag_coord = uvec2(global_ix.x, size.y - global_ix.y).as_vec2();
        let seed = frag_coord.xy() + uniform.time;
        let mut rng = SRng { seed };
        frag_coord = frag_coord + rng.gen_vec2();
        frag_coord = frag_coord / vec2(size.x as f32, size.y as f32) * 2.0 - 1.0;

        let pos = (uniform.camera_transform * Vec3::ZERO.extend(1.0)).truncate();
        let mut dir = (uniform.inverse_view * frag_coord.extend(-1.0).extend(1.0)).truncate();
        dir = (uniform.camera_transform * dir.extend(0.0)).truncate().normalize();
        let ray = Ray3 { pos, dir };

        let svt = MySvt { mem: svt };
        let env_color = ray::shade_ray(&mut rng, svt, ray);

        // debug code
        //let frag_color = ((Vec3::splat(direction.normalize().y) + 0.4)).extend(1.0);

        // debug code
        // let frag_color = vec4(
        //     frag_coord.x + 0.5,
        //     frag_coord.y + 0.5,
        //     uniform.time.sin(),
        //     1.0,
        // );
        let tex_coor = global_ix.truncate().as_ivec2();
        let prev_color: Vec4 = src_tex.read(tex_coor);
        let frame_index = uniform.frame_index as f32;
        let frag_color = (frame_index * prev_color.truncate() + env_color) / (frame_index + 1.0);
        let frag_color = frag_color.extend(1.0);
        unsafe { src_tex.write(tex_coor, frag_color) }
    }
}

// A simple vert/frag shader to copy an image to the swapchain.

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
