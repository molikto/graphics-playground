use core::ops::{Add, Div, Mul, Rem};

pub use glam::*;
use num_traits::Zero;


pub trait VecExt {
    fn step(self, other: Self) -> Self;
    fn step_f(self, other: f32) -> Self;
    fn sum(self) -> f32;
    fn new_axis(u: usize) -> Self;
}

impl VecExt for Vec3 {

    fn step_f(self, other: f32) -> Self {
        // TODO intrinsics
        vec3(
            if self.x > other { 0.0 } else { 1.0 },
            if self.y > other { 0.0 } else { 1.0 },
            if self.z > other { 0.0 } else { 1.0 },
        )
    }

    fn step(self, other: Self) -> Self {
        // TODO intrinsics
        vec3(
            if self.x > other.x { 0.0 } else { 1.0 },
            if self.y > other.y { 0.0 } else { 1.0 },
            if self.z > other.z { 0.0 } else { 1.0 },
        )
    }

    fn sum(self) -> f32 {
        self.dot(Self::splat(1.0))
    }

    fn new_axis(a: usize) -> Vec3 {
        if a == 0 {
            Vec3::new(1.0, 0.0, 0.0)
        } else if a == 1 {
            Vec3::new(0.0, 1.0, 0.0)
        } else if a == 2 {
            Vec3::new(0.0, 0.0, 1.0)
        } else {
            panic!()
        }
    }
}

impl VecExt for Vec2 {
    fn step(self, other: Self) -> Self {
        vec2(
            if self.x > other.x { 0.0 } else { 1.0 },
            if self.y > other.y { 0.0 } else { 1.0 },
        )
    }

    fn step_f(self, other: f32) -> Self {
        // TODO intrinsics
        vec2(
            if self.x > other { 0.0 } else { 1.0 },
            if self.y > other { 0.0 } else { 1.0 },
        )
    }

    fn sum(self) -> f32 {
        self.dot(Self::splat(1.0))
    }

    fn new_axis(a: usize) -> Vec2 {
        if a == 0 {
            Vec2::new(1.0, 0.0)
        } else if a == 1 {
            Vec2::new(0.0, 1.0)
        } else {
            panic!()
        }
    }
}

pub trait Vec2Ext {
    fn extend_axis(self, a: usize, s: f32) -> Vec3;
}

impl Vec2Ext for Vec2 {
    #[inline]
    fn extend_axis(self, a: usize, s: f32) -> Vec3 {
        if a == 0 {
            Vec3::new(s, self[0], self[1])
        } else if a == 1 {
            Vec3::new(self[0], s, self[1])
        } else if a == 2 {
            Vec3::new(self[0], self[1], s)
        } else {
            panic!()
        }
    }
}

pub trait Vec3Ext {
    fn omit_axis(self, a: usize) -> Vec2;
}

impl Vec3Ext for Vec3 {
    #[inline]
    fn omit_axis(self, a: usize) -> Vec2 {
        if a == 0 {
            self.yz()
        } else if a == 1 {
            self.xz()
        } else if a == 2 {
            self.xy()
        } else {
            panic!()
        }
    }
}



// suvec3


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SUVec3 {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}


pub fn suvec3(x: u16, y: u16, z: u16) -> SUVec3 {
    SUVec3 { x, y, z }
}

impl Add<SUVec3> for SUVec3 {
    type Output = SUVec3;

    fn add(self, rhs: SUVec3) -> Self::Output {
        SUVec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }

}

impl Zero for SUVec3 {
    fn zero() -> Self {
        SUVec3 { x: 0, y: 0, z: 0 }
    }

    fn is_zero(&self) -> bool {
        self.x == 0 && self.y == 0 && self.z == 0
    }

    fn set_zero(&mut self) {
        *self = Zero::zero();
    }
}

pub trait AsSUVec3 {
    fn as_suvec3(self) -> SUVec3;
}

impl AsSUVec3 for Vec3 {
    fn as_suvec3(self) -> SUVec3 {
        SUVec3 {
            x: self.x as u16,
            y: self.y as u16,
            z: self.z as u16,
        }
    }
}

impl SUVec3 {
    pub const ZERO: SUVec3 = SUVec3 { x: 0, y: 0, z: 0 };
    pub fn new(x: u16, y: u16, z: u16) -> SUVec3 {
        SUVec3 { x, y, z }
    }
    pub fn as_vec3(self) -> Vec3 {
        vec3(self.x as f32, self.y as f32, self.z as f32)
    }
}

impl Div<u16> for SUVec3 {
    type Output = SUVec3;

    fn div(self, rhs: u16) -> Self::Output {
        SUVec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Mul<u16> for SUVec3 {
    type Output = SUVec3;

    fn mul(self, rhs: u16) -> Self::Output {
        SUVec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Rem<u16> for SUVec3 {
    type Output = SUVec3;

    fn rem(self, rhs: u16) -> Self::Output {
        SUVec3 {
            x: self.x % rhs,
            y: self.y % rhs,
            z: self.z % rhs,
        }
    }
}