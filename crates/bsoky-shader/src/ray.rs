use bsoky_no_std::*;
use common_no_std::{material::*, svo::{Svo, BlockRayIntersectionInfo}, *};

// from Ray Tracing in One Weekend
pub fn skybox0(ray: &Ray3) -> Vec3 {
    let unit = ray.dir.normalize();
    let t = 0.5 * (unit.y + 1.0) as f32;
    (1.0 - t) * Vec3::new(1.0, 1.0, 1.0) + t * Vec3::new(0.5, 0.7, 1.0)
}

const MAX_RAY_DEPTH: u32 = 10;
pub fn shade_ray(rng: &mut SRng, svo: Svo<BLOCK_DIM, LEVEL_COUNT>, mut current_ray: Ray3) -> Vec3 {
    let material = Lambertian {
        albedo: RgbLinear(vec3(0.4, 0.1, 0.4)),
    };
    let mut accumulate_attenuation = Vec3::ONE;
    for _ in 0..MAX_RAY_DEPTH {
        let mut final_in_info: BlockRayIntersectionInfo = BlockRayIntersectionInfo {
            mask: Vec3::ZERO,
            t: 0.00001
        };
        let error_code = svo.traverse_ray(100, current_ray, |in_info, _, info| {
            if info.data == 1 {
                final_in_info = in_info;
                return true;
            } else {
                return false;
            }
        });
        if error_code < 0 {
            return MySvo::debug_error_code_colors(error_code);
        } else {
            // debug how much voxel get traveled
            // let count = error_code as f32;
            // return Vec3::splat((count) / 100.0);
            // debug levels
            // return Vec3::splat((count) / 1000.0);

            // simple debug light levels
            // let light_level = vec3(0.6, 0.75, 1.0);
            // return vec3( 0.8,0.7, 0.5) * (light_level.dot(hit_mask.abs()));

            // no hit, use sky box color
            if final_in_info.mask == Vec3::ZERO {
                return accumulate_attenuation * skybox0(&current_ray);
            } else {
                let interaction = material.scatter(
                    rng,
                    current_ray,
                    HitRecord3 {
                        is_hit: true,
                        t: final_in_info.t,
                        from_inside: false,
                        nor: -final_in_info.mask,
                    },
                );
                match interaction {
                    MaterialInteraction { attenuation, ray } => {
                        accumulate_attenuation = accumulate_attenuation * attenuation.0;
                        current_ray = ray;
                        current_ray.advance(0.01);
                    }
                }
            }
        }
    }
    return Vec3::ZERO;
}
