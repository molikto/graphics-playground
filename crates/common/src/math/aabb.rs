use super::face_mask::*;

use super::axis::*;
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
    pub fn contains(self, p: Vec2) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }

    pub fn extend(self: &Self, axis: Axis3, min: f32, max: f32) -> Aabb3 {
        Aabb3 {
            min: self.min.extend_axis(axis, min),
            max: self.max.extend_axis(axis, max),
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

#[cfg_attr(not(target_arch = "spirv"), derive(AsStd140, Debug))]
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

    pub fn omit(&self, side: Axis3) -> Aabb2 {
        Aabb2 {
            min: self.min.omit_axis(side),
            max: self.max.omit_axis(side),
        }
    }

    pub fn contains(&self, p: Vec3) -> bool {
        self.min.x <= p.x
            && p.x <= self.max.x
            && self.min.y <= p.y
            && p.y <= self.max.y
            && self.min.z <= p.z
            && p.z <= self.max.z
    }

    pub fn union(&self, b: &Aabb3) -> Aabb3 {
        Aabb3 {
            min: self.min.min(b.min),
            max: self.max.max(b.max),
        }
    }

    pub fn hit_fast1(&self, ray: &Ray3) -> f32 {
        let rdinv = 1.0 / ray.dir;
        let t1 = (self.min - ray.pos) * rdinv;
        let t2 = (self.max - ray.pos) * rdinv;
        let t_min = t1.min(t2);
        let t_max = t1.max(t2);
        let tmin = t_min.x.max(0.0).max(t_min.y.max(t_min.z));
        let tmax = t_max.x.min(t_max.y.min(t_max.z));
        if tmax > tmin {
            if tmin < 0.0 {
                //inside
                0.0
            } else {
                tmin
            }
        } else {
            -1.0
        }
    }

    pub fn hit_fast(&self, ray: &Ray3, t_min: f32, t_max: f32) -> bool {
        for a in 0..3 {
            let a = Axis3::from_raw(a);
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
            let a = Axis3::from_raw(a);
            let side = self.omit(a);
            let inv = 1.0 / ray.dir.get(a);
            let mut t0 = (self.min.get(a) - ray.pos.get(a)) * inv;
            let mut t1 = (self.max.get(a) - ray.pos.get(a)) * inv;
            let mut axis = -a.as_vec3();
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
                if side.contains(point.omit_axis(a)) {
                    candidate.is_hit = true;
                    candidate.from_outside = false;
                    candidate.nor = axis;
                    candidate.t = t0;
                }
            } else if t_min < t1 && t_max1 > t1 {
                let point = ray.at(t1);
                if side.contains(point.omit_axis(a)) {
                    candidate.is_hit = true;
                    candidate.from_outside = true;
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

pub struct Aap3 {
    pub axis: Axis3,
    pub pos: f32,
    pub extent: Aabb2,
}

impl Aap3 {
    pub fn from_unit_pos_face(pos: Vec3, mask: Face3) -> Aap3 {
        let pos1 = (pos + mask.as_vec3()).get(mask.axis);
        let pos = pos.omit_axis(mask.axis);
        Aap3 {
            axis: mask.axis,
            pos: pos1,
            extent: Aabb2 {
                min: pos,
                max: pos + Vec2::ONE
            },
        }
    }

    pub fn points(self) -> [Vec3; 4] {
        let mut points = [Vec3::ZERO; 4];
        points[0] = self.extent.min.extend_axis(self.axis, self.pos);
        points[1] = vec2(self.extent.min.x, self.extent.max.y).extend_axis(self.axis, self.pos);
        points[2] = self.extent.max.extend_axis(self.axis, self.pos);
        points[3] = vec2(self.extent.max.x, self.extent.min.y).extend_axis(self.axis, self.pos);
        points
    }
}
