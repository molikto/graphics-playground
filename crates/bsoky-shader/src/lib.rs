#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;
use spirv_std::num_traits::Float;
use common_no_std::{math::*, shader::base_uniform::*, svt::*};
use bsoky_no_std::*;

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
    let mut frag_world_position= vec3(0.0, 0.0, 0.0);
    if face == 0 { // max z side
        frag_world_position = vec3(uv.x * map_size.x, uv.y * map_size.y, map_size.z);
    } else if face == 1 { // min z side
        frag_world_position = vec3((1.0 - uv.x) * map_size.x, (1.0 - uv.y) * map_size.y, 0.0);
    } else if face == 2 { // max x side
        frag_world_position = vec3(map_size.x, uv.x * map_size.y, uv.y * map_size.z);
    } else if face == 3 { // min x side
        frag_world_position = vec3(0.0, (1.0 - uv.x) * map_size.y, (1.0 - uv.y) * map_size.z);
    } else if face == 4 { // max y side
        frag_world_position = vec3(uv.x * map_size.x, map_size.y, uv.y * map_size.z);
    } else if face == 5 { // min y side
        frag_world_position = vec3((1.0 - uv.x) * map_size.x, 0.0, (1.0 - uv.y) * map_size.z);
    }
    return frag_world_position;
}

// A simple vert/frag shader to copy an image to the swapchain.
#[spirv(fragment)]
pub fn fragment(
    #[spirv(frag_coord)]
    in_frag_coord: Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &ViewUniform,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] svt: &[usvt],
    #[spirv(flat)] face: u32,
    uv: Vec2,
    output: &mut Vec4,
) {
    let svt = MySvt { mem: svt };
    let total_dim = MySvt::total_dim() as f32;
    let map_size = Vec3::splat(total_dim);
    let frag_world_position = frag_world_position_from_face(from_simulation_coor(map_size), face, uv);

    //render the distance for debug purposes
    // let distance = (frag_world_position - view.world_position).length();
    // *output = Vec3::splat(distance / (total_dim as f32)).extend(1.0);
    // return;

    let seed = in_frag_coord.xy();// + Vec2::splat(constants.rng_seed_offset);
    let mut rng = SRng { seed };

    let mut ray = Ray3 {
        pos: to_simulation_coor(view.world_position),
        dir: to_simulation_coor(frag_world_position - view.world_position)
    };
    ray.dir = ray.dir.normalize_or_zero();
    if ray.dir == Vec3::ZERO {
        ray.dir = vec3(1.0, 0.0, 0.0);
    }
    let env_color = ray::shade_ray(&mut rng, svt, ray);

    // let color = clamp(env_color, Vec4(0.0), Vec4(1.0));
    // let color = color_over(env_color, Vec4(0.5, 0.5, 0.5, 1.0));
    *output = env_color.extend(1.0);
    // no need for gamma correction
    //return Vec4(pow(env_color, Vec3(1.0/2.2)), 1.0);
}