use std::cmp::Ordering;

use super::{Ray3, f32, aabb::Aabb3};


pub trait BaseHitResult {
  fn t(self: &Self) -> f32;
}

pub trait Hittable: Clone {
  type T: BaseHitResult;
  fn aabb(self: &Self) -> Aabb3;
  fn hit(&self, ray: &Ray3, t_min: f32, t_max: f32) -> Option<Self::T>;
}

#[derive(Debug)]
pub struct Bvh3<T : Hittable> {
  aabb: Aabb3,
  body: Bvh3Inner<T>
}

#[derive(Debug)]
pub enum Bvh3Inner<T : Hittable> {
  Empty,
  Leaf {
    item: T // TODO is it a good idea to have an indirection here? so memory is more compact?
  },
  Node {
    left: Box<Bvh3<T>>,
    right: Box<Bvh3<T>>
  }
}

impl<T : Hittable> Bvh3<T> {
  pub fn hit<'a>(self: &'a Self, ray: &Ray3, t_min: f32, t_max: f32) -> Option<(T::T, &'a T)> {
    if self.aabb.hit_fast(ray, t_min, t_max) {
      match &self.body {
        Bvh3Inner::Node { left, right } => {
          match left.hit(ray, t_min, t_max) {
            Option::Some(res) => right.hit(ray, t_min, res.0.t()).or_else(|| { Some(res) }),
            Option::None => right.hit(ray, t_min, t_max)
          }
        }
        Bvh3Inner::Leaf { item } => item.hit(ray, t_min, t_max).map(|a | { (a, item) }),
        Bvh3Inner::Empty => panic!(),
    }
    } else {
      None
    }
  }

  fn new_rec(items: &mut [T], axis: usize) -> Bvh3<T> {
    let span = items.len();
    if span == 0 {
      Bvh3 { aabb: Aabb3::empty(), body: Bvh3Inner::Empty }
    } else if span == 1 {
      let item = items[0].clone(); // the only clone we use!
      Bvh3 { aabb: item.aabb(), body: Bvh3Inner::Leaf { item: item } }
    } else {
      items.sort_by(|a, b| {
        let diff = a.aabb().min[axis] - b.aabb().min[axis];
        if diff == 0.0 {
          Ordering::Equal
        }  else if diff < 0.0 {
          Ordering::Less
        } else {
          Ordering::Greater
        }
       });
      let mid = items.len() / 2;
      let axis = (axis + 1) % 3;
      let left = Self::new_rec(&mut items[0.. mid], axis);
      let right = Self::new_rec(&mut items[mid.. span], axis);
      Bvh3 { aabb: Aabb3::union(&left.aabb, &right.aabb), body: Bvh3Inner::Node { left: Box::new(left), right: Box::new(right) } }
    }
  }
  pub fn new(mut items: Vec<T>) -> Bvh3<T> {
    Self::new_rec(&mut items, 0)
  }
}
