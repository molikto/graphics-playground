use bsoky_no_std::*;
use common_no_std::{svo::Svo, *};

pub fn shade_ray(svo: Svo<BLOCK_DIM, LEVEL_COUNT>, ray: Ray3) -> Vec3 {
    let mut count = 0;
    let mut hit_mask: Vec3 = Vec3::ZERO;
    let error_code = svo.traverse_ray(ray, |info, mask| {
        // debug count
        count += 1;
        if count > 400 {
            return true;
        }
        // debug levels
        // count += info.size();
        // if count > 1000 {
        //     return true;
        // }
        if info.data == 1 {
            hit_mask = mask;
            return true;
        } else {
            return false;
        }
    });
    if error_code != 0 {
        if error_code == 1 {
            return vec3(1.0, 0.0, 0.0);
        } else if error_code == 2 {
            return vec3(0.0, 1.0, 0.0);
        } else if error_code == 3 {
            return vec3(0.0, 0.0, 1.0);
        } else if error_code == 4 {
            return vec3(1.0, 0.0, 0.0);
        } else if error_code == 5 {
            return vec3(1.0, 0.0, 1.0);
        } else if error_code == 6 {
            return vec3(0.0, 1.0, 1.0);
        } else {
            panic!();
        }
    } else {
        // debug how much voxel get traveled
        return Vec3::splat((count as f32) / 100.0);
        // debug levels
        // return Vec3::splat((count as f32) / 1000.0);
        // let light_level = vec3(0.6, 0.75, 1.0);
        // return vec3( 0.8,0.7, 0.5) * (light_level.dot(hit_mask.abs()));
    }
}
