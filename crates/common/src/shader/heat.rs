
use crate::math::vec::*;

pub fn heat(x: f32) -> Vec3 {
    (x.min(1.0).max(0.0) * 3.0 - vec3(1.0, 2.0, 3.0)).sin() * 0.5 + 0.5
}
