use crate::math::*;


#[allow(non_snake_case)]

pub fn Beer_Lambert(absorption_coefficient: f32, distance: f32) -> f32 {
  return f32::exp(-absorption_coefficient * distance);
}

#[allow(non_snake_case)]
pub fn Beer_Lambert3(absorption_coefficient: Vec3, distance: f32) -> Vec3 {
  return Vec3::exp(-absorption_coefficient * distance);
}