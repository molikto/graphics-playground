
use core::ops::Add;

use super::vec::*;

///
/// RAY
///
#[derive(Copy, Clone, Default)]
pub struct Ray3 {
    pub pos: Vec3,
    pub dir: Vec3
}

impl Ray3 {
    pub fn at(&self, t: f32) -> Vec3 {
        self.pos + t * self.dir
    }
}

impl Add<Vec3> for Ray3 {
  type Output = Ray3;

  fn add(self, v: Vec3) -> Ray3 {
      Ray3 { pos: self.pos + v, dir: self.dir }
  }
}

#[derive(Default)]
pub struct HitRecord3 {
    pub is_hit: bool,
    pub t: f32,
    pub pos: Vec3,
    pub out: bool,
    pub nor: Vec3,
}
