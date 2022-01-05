use bsoky_no_std::*;
use common_no_std::{
    material::*,
    svo::Svo,
    *,
};

const MAX_RAY_DEPTH: u32 = 10;
pub fn shade_ray(rng: &mut SRng, svo: Svo<BLOCK_DIM, LEVEL_COUNT>, mut current_ray: Ray3) -> Vec3 {
    let material = Lambertian {
        albedo: RgbLinear(vec3(0.4, 0.1, 0.4)),
    };
    let mut accumulate_attenuation = Vec3::ONE;
    for _ in 0..MAX_RAY_DEPTH {
        let mut hit_mask: Vec3 = Vec3::ZERO;
        let mut t_final: f32 = 0.000001;
        let error_code = svo.traverse_ray(100, current_ray, |t_hit, info, mask| {
            // debug levels
            // count += info.size();
            // if count > 1000 {
            //     return true;
            // }
            if info.data == 1 {
                hit_mask = mask;
                t_final = t_hit;
                return true;
            } else {
                return false;
            }
        });
        if error_code < 0 {
            if error_code == -1 {
                return vec3(1.0, 0.0, 0.0);
            } else if error_code == -2 {
                return vec3(0.0, 1.0, 0.0);
            } else if error_code == -3 {
                return vec3(0.0, 0.0, 1.0);
            } else if error_code == -4 {
                return vec3(1.0, 0.0, 0.0);
            } else if error_code == -5 {
                return vec3(1.0, 0.0, 1.0);
            } else if error_code == -6 {
                return vec3(0.0, 1.0, 1.0);
            } else {
                panic!();
            }
        } else {
            // no hit, use sky box color
            if hit_mask == Vec3::ZERO {
                return accumulate_attenuation;
            } else {
                let interaction = material.scatter(
                    rng,
                    current_ray,
                    HitRecord3 {
                        is_hit: true,
                        t: t_final,
                        from_inside: false,
                        nor: hit_mask,
                    },
                );
                match interaction {
                    MaterialInteraction { attenuation, ray } => {
                        accumulate_attenuation = accumulate_attenuation * attenuation.0;
                        current_ray = ray;
                    },
                }
            }
            // debug how much voxel get traveled
            // let count = error_code as f32;
            // return Vec3::splat((count) / 100.0);
            // debug levels
            // return Vec3::splat((count) / 1000.0);
            // let light_level = vec3(0.6, 0.75, 1.0);
            // return vec3( 0.8,0.7, 0.5) * (light_level.dot(hit_mask.abs()));
        }
    }
    return Vec3::ZERO;
}
