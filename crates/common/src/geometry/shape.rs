use std::mem::swap;

use super::vec::*;
use super::aabb::*;
use super::bvh::*;
use super::ray::*;

#[derive(Copy, Clone, Debug)]
pub struct HitRecord {
    pub point: Vec3,
    pub out: bool,
    pub nor: Vec3,
    pub t: f32,
}

impl BaseHitResult for HitRecord {
    fn t(self: &Self) -> f32 {
        return self.t;
    }
}

impl Hittable for Aabb3 {
    type T = HitRecord;

    fn aabb(self: &Self) -> Aabb3 {
        *self
    }

    fn hit(&self, ray: &Ray3, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut candidate: Option<HitRecord> = None;
        for a in 0..3 {
            let side = self.omit(a);
            let inv = 1.0 / ray.dir[a];
            let mut t0 = (self.min[a] - ray.pos[a]) * inv;
            let mut t1 = (self.max[a] - ray.pos[a]) * inv;
            let mut axis = -Vec3::new_axis(a);
            if inv < 0.0 {
                swap(&mut t0, &mut t1);
                axis = -axis;
            }
            let t_max_original = t_max;
            let t_max = candidate.map_or(t_max, |c| {c.t});
            if t_min < t0 && t_max > t0 {
                // go inside from min side
                let point = ray.at(t0);
                if side.inside(point.omit_axis(a)) {
                    candidate = Some(HitRecord {
                        point: point,
                        out: true,
                        nor: axis,
                        t: t0,
                    });
                }
            } else if t_min < t1 && t_max > t1 {
                // to outside from max side
                let point = ray.at(t1);
                if side.inside(point.omit_axis(a)) {
                    candidate = Some(HitRecord {
                        point: point,
                        out: false,
                        nor: axis,
                        t: t1,
                    });
                }
            } else if t_max_original <= t0 || t_min >= t1 {
                return None;
            }
        }
        return candidate;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Hittable for Sphere {
    type T = HitRecord;
    fn aabb(self: &Self) -> Aabb3 {
        let r = self.radius;
        let r = Vec3::new(r, r, r);
        return Aabb3 {
            min: self.center - r,
            max: self.center + r,
        };
    }

    fn hit(&self, ray: &Ray3, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let amc: Vec3 = ray.pos - self.center;
        let a = ray.dir.length_squared();
        let b = ray.dir.dot(amc);
        let c = amc.length_squared() - self.radius * self.radius;
        let discriminant = b * b - a * c;
        if discriminant > 0.0 {
            let d_sqrt = discriminant.sqrt();
            let t = (-b - d_sqrt) / a;
            let t = if t > t_min && t < t_max {
                Some(t)
            } else {
                let t = (-b + d_sqrt) / a;
                if t > t_min && t < t_max {
                    Some(t)
                } else {
                    Option::None
                }
            };
            t.map(|t| {
                let point = ray.at(t);
                let mut normal: Vec3 = (point - self.center) / self.radius;
                let out = normal.dot(ray.dir) <= 0.0;
                if !out {
                    normal = -normal;
                }
                HitRecord {
                    point,
                    nor: normal,
                    out,
                    t,
                }
            })
        } else {
            Option::None
        }
    }
}

#[derive(Clone, Debug)]
pub struct RotateY {
    pub shape: Box<Shape>,
    pub theta: f32
}

impl Hittable for RotateY {
    type T = HitRecord;

    fn aabb(self: &Self) -> Aabb3 {
        todo!()
    }

    fn hit(&self, ray: &Ray3, t_min: f32, t_max: f32) -> Option<HitRecord> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub enum Shape {
    Sphere(Sphere),
    Aabb3(Aabb3),
    Translate { 
        shape: Box<Shape>,
        v: Vec3
    },
    RotateY(RotateY)
}
impl Hittable for Shape {
    type T = HitRecord;
    fn aabb(&self) -> Aabb3 {
        match self {
            Shape::Sphere(s) => s.aabb(),
            Shape::Aabb3(s) => s.aabb(),
            Shape::RotateY(s) => s.aabb(),
            Shape::Translate { shape, v } => shape.aabb() + *v
        }
    }

    fn hit(&self, ray: &Ray3, t_min: f32, t_max: f32) -> Option<HitRecord> {
        match self {
            Shape::Sphere(s) => s.hit(ray, t_min, t_max),
            Shape::Aabb3(s) => s.hit(ray, t_min, t_max),
            Shape::RotateY(s) => s.hit(ray, t_min, t_max),
            Shape::Translate { shape, v } => shape.hit(&(*ray + (-*v)), t_min, t_max).map(|mut r| {
                r.point = r.point + *v;
                r
            })
        }
    }
}
