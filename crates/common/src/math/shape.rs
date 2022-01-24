use super::vec::*;
use crate::aabb::*;
use crate::ray::*;

pub trait ShapeIntersect {
    type Ohter;
    fn intersect(&self, other: &Self::Ohter) -> bool;
}

#[derive(Copy, Clone)]
pub struct Sphere {
    pub pos: Vec3,
    pub r: f32,
}

impl Sphere {
}

impl ShapeIntersect for Sphere {
    type Ohter = Aabb3;

    fn intersect(&self, other: &Self::Ohter) -> bool {
        let ray = Ray3 { pos: self.pos, dir: ((other.min + other.max) / 2.0 - self.pos).normalize() };
        let hit = other.hit_fast(&ray, 0.0, self.r);
        hit
    }
}
