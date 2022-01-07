use super::color::*;
use crate::math::*;

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

pub trait AbstractMaterial {
    fn scatter(self, rng: &mut SRng, ray: Ray3, hit: HitRecord3) -> MaterialInteraction;
}

#[derive(Copy, Clone)]
pub struct Lambertian {
    pub albedo: RgbLinear,
}

const LAMBERTIAN_SCATTER: u8 = 1;

impl AbstractMaterial for Lambertian {
    fn scatter(self, rng: &mut SRng, ray: Ray3, hit: HitRecord3) -> MaterialInteraction {
        let mut ray_dir = if LAMBERTIAN_SCATTER == 0 {
            hit.nor + rng.gen_in_unit_sphere().normalize()
        } else if LAMBERTIAN_SCATTER == 1 {
            rng.gen_in_hemisphere(hit.nor)
        } else {
            panic!();
        };
        // avoid the case normal and ray_dir cancel out
        ray_dir = ray_dir.normalize_or_zero();
        if ray_dir == Vec3::ZERO {
            ray_dir = hit.nor;
        }
        MaterialInteraction {
            attenuation: self.albedo,
            ray: Ray3 {
                pos: ray.at(hit.t),
                dir: ray_dir,
            },
        }
    }
}

#[derive(Copy, Clone)]
pub struct Metal {
    pub albedo: RgbLinear,
    pub fuzz: f32,
}

impl AbstractMaterial for Metal {
    fn scatter(self, rng: &mut SRng, ray: Ray3, hit: HitRecord3) -> MaterialInteraction {
        let n = hit.nor;
        let v: Vec3 = ray.dir.normalize();
        let reflected = v.reflect(n);
        MaterialInteraction {
            attenuation: self.albedo,
            ray: Ray3 {
                pos: ray.at(hit.t),
                dir: reflected + rng.gen_in_unit_sphere() * self.fuzz,
            },
        }
    }
}

#[derive(Copy, Clone)]
pub struct Dielectric {
    pub ref_idx: f32,
}

fn refract(uv: Vec3, n: Vec3, eta_over_etat: f32) -> Vec3 {
    // not understanding, just copying
    let cos_theta = (-uv).dot(n);
    let r_out_parallel: Vec3 = eta_over_etat * (uv + cos_theta * n);
    let r_out_perp: Vec3 = -(1.0 - r_out_parallel.length_squared()).sqrt() * n;
    r_out_parallel + r_out_perp
}

fn schlick(cosine: f32, ref_idx: f32) -> f32 {
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

impl AbstractMaterial for Dielectric {
    fn scatter(self, rng: &mut SRng, ray: Ray3, hit: HitRecord3) -> MaterialInteraction {
        let refraction_index = self.ref_idx;
        let refrection_ratio = if hit.from_outside {
            1.0 / refraction_index
        } else {
            refraction_index
        };
        let v: Vec3 = ray.dir.normalize();
        // something I don't bother to understand
        let cos_theta = (-v).dot(hit.nor).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract = refrection_ratio * sin_theta > 1.0;
        let dir = if cannot_refract || rng.gen() < schlick(cos_theta, refrection_ratio) {
            v.reflect(hit.nor)
        } else {
            refract(v, hit.nor, refrection_ratio)
        };
        MaterialInteraction {
            attenuation: RgbLinear(Vec3::new(1.0, 1.0, 1.0)),
            ray: Ray3 {
                pos: ray.at(hit.t),
                dir,
            },
        }
    }
}

pub enum Material {
    Lambertian(Lambertian),
    Metal(Metal),
    Dielectric(Dielectric),
}

// impl AbstractMaterial for Material {
//     fn scatter(self, rng: &mut SRng, ray: Ray3, hit: HitRecord3) -> MaterialInteraction {
//         match self {
//             Material::Lambertian(i) => i.scatter(rng, ray, hit),
//             Material::Metal(i) => i.scatter(rng, ray, hit),
//             Material::Dielectric(i) => i.scatter(rng, ray, hit),
//         }
//     }
// }
