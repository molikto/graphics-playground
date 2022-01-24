use bsoky_common_cgpu::{sim::VoxelData, *};
use common::{
    graphics::{material::*, *},
    math::*,
    svt::*,
};
use spirv_std::image;

#[derive(PartialEq, Eq)]
#[repr(C)]
pub enum RenderMode {
    RayTracing,
    DotNShading,
    Iteration,
}

const MAX_RAY_DEPTH: u32 = 2;

pub const MAX_ITERATION: u32 = 256;
pub const MAX_HEAT_ITERATION: u32 = 128;

// from https://raytracing.github.io/
pub fn skybox0(ray: Ray3) -> Vec3 {
    let unit = ray.dir;
    let t = 0.5 * (unit.y + 1.0) as f32;
    (1.0 - t) * Vec3::new(1.0, 1.0, 1.0) + t * Vec3::new(0.5, 0.7, 1.0)
}

pub fn face_color(
    current_ray: Ray3,
    in_info: BlockRayIntersectionInfo,
    voxel_data: VoxelData,
    textures: &image::Image2d,
) -> Vec4 {
    // TODO bad code? for the 15.xxx?
    let pos = current_ray.at(in_info.t);
    let pos_fract = pos.fract();
    let uv = pos_fract.omit_by_nonzero(in_info.mask);
    let mut color: Vec4 = textures.fetch((uv * 15.99999).as_ivec2());
    color 
}

pub fn shade_ray(
    render_mode: RenderMode,
    rng: &mut SRng,
    svt: &MySvt,
    textures: &image::Image2d,
    mut current_ray: Ray3,
) -> Vec3 {
    let mut accumulate_attenuation = Vec3::ONE;
    for _ in 0..MAX_RAY_DEPTH {
        let mut final_in_info: BlockRayIntersectionInfo = BlockRayIntersectionInfo {
            mask: Sign3::ZERO,
            t: -1.0,
        };
        let mut result_voxel_data: VoxelData = VoxelData::EMPTY;
        let error_code = svt.traverse_ray(MAX_ITERATION, current_ray, |in_info, _, info| {
            if info.data != VoxelData::EMPTY {
                final_in_info = in_info;
                result_voxel_data = info.data;
                return true;
            } else {
                return false;
            }
        });
        if error_code < 0 {
            return MySvt::debug_error_code_colors(error_code);
        } else {
            if render_mode == RenderMode::Iteration {
                let count = error_code as f32;
                return Vec3::splat((count) / (MAX_HEAT_ITERATION as f32));
                //return heat::heat((count) / (MAX_ITERATION as f32));
            } else {
                let color = face_color(current_ray, final_in_info, result_voxel_data, textures);
                let mask = final_in_info.mask.as_vec3();
                if render_mode == RenderMode::DotNShading {
                    let light_level = vec3(0.4, 0.75, 1.0);
                    return color.xyz() * (light_level.dot(mask.abs()));
                } else {
                    // no hit, use sky box color
                    if final_in_info.t == -1.0 {
                        return accumulate_attenuation * skybox0(current_ray);
                    } else {
                        let hit_record = HitRecord3 {
                            is_hit: true,
                            t: final_in_info.t,
                            from_outside: true,
                            nor: mask,
                        };
                        let interaction: MaterialInteraction;
                        let material = Lambertian {
                            albedo: RgbLinear(color.xyz()),
                        };
                        interaction = material.scatter(rng, current_ray, hit_record);
                        match interaction {
                            MaterialInteraction { attenuation, ray } => {
                                accumulate_attenuation = accumulate_attenuation * attenuation.0;
                                current_ray = ray;
                                current_ray.advance(0.0001); // TOOD is this correct? "Fixing Shadow Acne" in "Ray Tracing in One Weekend"?
                            }
                        };
                    }
                }
            }
        }
    }
    return Vec3::ZERO;
}
