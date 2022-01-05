use super::ray::*;
use super::vec::*;

#[cfg(not(target_arch = "spirv"))]
use bevy_crevice::std140::AsStd140;

#[derive(Copy, Clone)]
pub struct Aabb2 {
    pub min: Vec2,
    pub max: Vec2,
}

impl Aabb2 {
    pub fn empty() -> Aabb2 {
        let min = f32::INFINITY;
        let min = Vec2::new(min, min);
        let max = f32::NEG_INFINITY;
        let max = Vec2::new(max, max);
        Aabb2 { min: min, max: max }
    }

    pub fn union(a: &Aabb2, b: &Aabb2) -> Aabb2 {
        Aabb2 {
            min: a.min.min(b.min),
            max: a.max.max(b.max),
        }
    }

    // pub fn inside(self: &Aabb2, p: Vec2) -> bool {
    //     p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    // }

    // TODO these inside function should account for edges
    pub fn inside(self, p: Vec2) -> bool {
        let s = self.min.step(p) - self.max.step(p);
        return s.x * s.y == 1.0;
    }

    pub fn extend(self: &Self, axis: usize, min: f32, max: f32) -> Aabb3 {
        Aabb3 {
            min: self.min.extend_axis(axis, min),
            max: self.max.extend_axis( axis, max),
        }
    }
}
impl core::ops::Add<Vec3> for Aabb3 {
    type Output = Aabb3;

    fn add(self, v: Vec3) -> Aabb3 {
        Aabb3 {
            min: self.min + v,
            max: self.max + v,
        }
    }
}

#[cfg_attr(not(target_arch = "spirv"), derive(AsStd140))]
#[derive(Copy, Clone, Default)]
pub struct Aabb3 {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb3 {
    pub fn new(min: Vec3, max: Vec3) -> Aabb3 {
        Aabb3 { min, max }
    }
    pub fn empty() -> Aabb3 {
        let min = f32::INFINITY;
        let min = Vec3::new(min, min, min);
        let max = f32::NEG_INFINITY;
        let max = Vec3::new(max, max, max);
        Aabb3 { min: min, max: max }
    }

    pub fn omit(self: &Self, side: usize) -> Aabb2 {
        Aabb2 {
            min: self.min.omit_axis(side),
            max: self.max.omit_axis(side),
        }
    }

    pub fn inside(self: &Self, p: Vec3) -> bool {
        let s = self.min.step(p) - self.max.step(p);
        return s.x * s.y * s.z == 1.0;
    }

    pub fn union(a: &Aabb3, b: &Aabb3) -> Aabb3 {
        Aabb3 {
            min: a.min.min(b.min),
            max: a.max.max(b.max),
        }
    }

    pub fn hit_fast(self: &Aabb3, ray: &Ray3, t_min: f32, t_max: f32) -> bool {
        for a in 0..3 {
            let inv = 1.0 / ray.dir.get(a);
            let mut t0 = (self.min.get(a) - ray.pos.get(a)) * inv;
            let mut t1 = (self.max.get(a) - ray.pos.get(a)) * inv;
            if inv < 0.0 {
                let t = t0;
                t0 = t1;
                t1 = t;
            }
            let t_min = if t0 > t_min { t0 } else { t_min };
            let t_max = if t0 < t_max { t1 } else { t_max };
            if t_max <= t_min {
                return false;
            }
        }
        return true;
    }

    pub fn hit(self: Aabb3, ray: &Ray3, t_min: f32, t_max: f32) -> HitRecord3 {
        let mut candidate = HitRecord3::default();
        for a in 0..3 {
            let side = self.omit(a);
            let inv = 1.0 / ray.dir.get(a);
            let mut t0 = (self.min.get(a) - ray.pos.get(a)) * inv;
            let mut t1 = (self.max.get(a) - ray.pos.get(a)) * inv;
            let mut axis = -Vec3::new_axis(a);
            if inv < 0.0 {
                let temp = t0;
                t0 = t1;
                t1 = temp;
                axis = -axis;
            }
            let mut t_max1 = t_max;
            if candidate.is_hit {
                t_max1 = candidate.t;
            }
            if t_min < t0 && t_max1 > t0 {
                let point = ray.at(t0);
                if side.inside(point.omit_axis(a)) {
                    candidate.is_hit = true;
                    candidate.from_inside = true;
                    candidate.nor = axis;
                    candidate.t = t0;
                }
            } else if t_min < t1 && t_max1 > t1 {
                let point = ray.at(t1);
                if side.inside(point.omit_axis(a)) {
                    candidate.is_hit = true;
                    candidate.from_inside = false;
                    candidate.nor = axis;
                    candidate.t = t1;
                }
            } else if t_max <= t0 || t_min >= t1 {
                return candidate;
            }
        }
        return candidate;
    }
}
