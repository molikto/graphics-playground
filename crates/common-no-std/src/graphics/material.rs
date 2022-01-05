use super::color::*;
use crate::math::*;

fn random_in_unit_sphere(rng: &mut SRng) -> Vec3 {
    let mut p;
    loop {
        let rv = vec3(rng.gen(), rng.gen(), rng.gen());
        p = 2.0 * rv - Vec3::ONE;
        let p_len2 = p.length_squared();
        if p_len2 < 1.0 {
            break p;
        }
    }
}

// #[derive(Copy, Clone)]
// pub enum MaterialInteraction {
//     Scatter {
//         pub attenuation: RgbLinear,
//         pub ray: Ray3
//     },
//     Emit {
//         pub color: RgbLinear
//     }
// }

#[derive(Copy, Clone)]
pub struct MaterialInteraction {
    pub attenuation: RgbLinear,
    pub ray: Ray3,
}

pub trait Material {
    fn scatter(self, rng: &mut SRng, ray: Ray3, hit: HitRecord3) -> MaterialInteraction;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Lambertian {
    pub albedo: RgbLinear,
}

impl Material for Lambertian {
    fn scatter(self, rng: &mut SRng, ray: Ray3, hit: HitRecord3) -> MaterialInteraction {
        MaterialInteraction {
            attenuation: self.albedo,
            ray: Ray3 {
                pos: ray.at(hit.t),
                dir: hit.nor + random_in_unit_sphere(rng),
            },
        }
    }
}
