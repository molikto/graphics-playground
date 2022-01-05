use crate::math::*;
use super::color::*;


#[derive(Copy, Clone)]
pub enum MaterialInteraction {
    Scatter {
        attenuation: RgbLinear,
        ray: Ray3
    },
    Emit {
        color: RgbLinear
    }
}


pub trait Material {
    fn scatter(
        self,
        ray: Ray3,
        hit: HitRecord3
    ) -> bool;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Lambertian {
    pub albedo: RgbLinear,
}


impl Material for Lambertian {
    fn scatter(
        self,
        ray: Ray3,
        hit: HitRecord3
    ) -> bool {
        todo!()
    }
}