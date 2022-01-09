use bsoky_no_std::*;
use common_no_std::{
    material::*,
    svo::{usvo, BlockRayIntersectionInfo, Svo},
    *,
};

// from Ray Tracing in One Weekend
pub fn skybox0(ray: &Ray3) -> Vec3 {
    let unit = ray.dir.normalize();
    let t = 0.5 * (unit.y + 1.0) as f32;
    (1.0 - t) * Vec3::new(1.0, 1.0, 1.0) + t * Vec3::new(0.5, 0.7, 1.0)
}

const MAX_RAY_DEPTH: u32 = 4;
pub fn shade_ray(rng: &mut SRng, svo: MySvo, mut current_ray: Ray3) -> Vec3 {
    let material1 = Lambertian {
        albedo: RgbLinear(vec3(0.4, 0.1, 0.4)),
    };
    let material2 = Metal {
        albedo: RgbLinear(vec3(0.9, 0.3, 0.4)),
        fuzz: 0.1,
    };
    let material3 = Dielectric {
        ref_idx: 1.5
    };
    let mut accumulate_attenuation = Vec3::ONE;
    for _ in 0..MAX_RAY_DEPTH {
        let mut final_in_info: BlockRayIntersectionInfo = BlockRayIntersectionInfo {
            mask: Vec3::ZERO,
            t: -1.0,
        };
        let mut material_index: usvo = 0;
        let error_code = svo.traverse_ray(400, current_ray, |in_info, _, info| {
            if info.data != 0 {
                final_in_info = in_info;
                material_index = info.data;
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
            // return Vec3::splat((count) / 400.0);

            // debug levels
            // return Vec3::splat((count) / 1000.0);

            // simple debug light levels
            // let light_level = vec3(0.6, 0.75, 1.0);
            // return vec3( 0.8,0.7, 0.5) * (light_level.dot(hit_mask.abs()));

            // no hit, use sky box color
            if final_in_info.t == -1.0 {
                return accumulate_attenuation * skybox0(&current_ray);
            } else {
                let hit_record = HitRecord3 {
                    is_hit: true,
                    t: final_in_info.t,
                    from_outside: true,
                    nor: -final_in_info.mask,
                };
                let mut interaction: MaterialInteraction;
                if material_index == 1 {
                    interaction = material1.scatter(rng, current_ray, hit_record);
                } else if material_index == 2 {
                    interaction = material2.scatter(rng, current_ray, hit_record);
                } else {
                    interaction = material3.scatter(rng, current_ray, hit_record);
                };
                match interaction {
                    MaterialInteraction { attenuation, ray } => {
                        accumulate_attenuation = accumulate_attenuation * attenuation.0;
                        current_ray = ray;
                        current_ray.advance(0.001); // TOOD is this correct? "Fixing Shadow Acne" in "Ray Tracing in One Weekend"?
                    }
                };
            }
        }
    }
    return Vec3::ZERO;
}
