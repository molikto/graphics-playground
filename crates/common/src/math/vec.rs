use core::ops::{Add, Div, Mul, Rem};

pub use glam::*;

#[cfg(target_arch = "spirv")]
pub use num_traits::*;

use num_traits::Zero;

pub trait MyVecExt {
    // TODO CPU version with indexing
    fn get(self, a: usize) -> f32;
    fn step(self, other: Self) -> Self;
    fn step_f(self, other: f32) -> Self;
    fn try_normalize_or(self, other: Self) -> Self;
    fn sum(self) -> f32;
    fn new_axis(u: usize) -> Self;
}

pub trait MyUVecExt {
    fn get(self, a: usize) -> u32;
}

impl MyUVecExt for UVec3 {
    fn get(self, a: usize) -> u32 {
        match a {
            0 => self.x,
            1 => self.y,
            _ => self.z,
        }
    }
}

impl MyVecExt for Vec3 {
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

    #[inline]
    fn get(self, a: usize) -> f32 {
        match a {
            0 => self.x,
            1 => self.y,
            _ => self.z,
        }
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

    fn try_normalize_or(self, other: Self) -> Self {
        let rcp = self.length_recip();
        if rcp.is_finite() && rcp > 0.0 {
            self * rcp
        } else {
            other
        }
    }
}

impl MyVecExt for Vec2 {
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

    fn try_normalize_or(self, other: Self) -> Self {
        let rcp = self.length_recip();
        if rcp.is_finite() && rcp > 0.0 {
            self * rcp
        } else {
            other
        }
    }

    #[inline]
    fn get(self, a: usize) -> f32 {
        return match a {
            0 => self.x,
            _ => self.y,
        };
    }
}

pub trait MyVec2Ext {
    fn extend_axis(self, a: usize, s: f32) -> Vec3;
}

impl MyVec2Ext for Vec2 {
    #[inline]
    fn extend_axis(self, a: usize, s: f32) -> Vec3 {
        if a == 0 {
            Vec3::new(s, self.x, self.y)
        } else if a == 1 {
            Vec3::new(self.x, s, self.y)
        } else if a == 2 {
            Vec3::new(self.x, self.y, s)
        } else {
            panic!()
        }
    }
}

pub trait MyVec3Ext {
    fn omit_axis(self, a: usize) -> Vec2;
    fn reflect(self, normal: Self) -> Self;
    fn sin(self) -> Vec3;
    fn sign_bit(self) -> Vec3;
    fn de_eps(self, eps: f32) -> Vec3;
}

impl MyVec3Ext for Vec3 {
    fn de_eps(self, eps: f32) -> Vec3 {
        let mut d = self;
        d.x = if d.x.abs() > eps {
            d.x
        } else if d.x >= 0.0 {
            eps
        } else {
            -eps
        };
        d.y = if d.y.abs() > eps {
            d.y
        } else if d.y >= 0.0 {
            eps
        } else {
            -eps
        };
        d.z = if d.z.abs() > eps {
            d.z
        } else if d.z >= 0.0 {
            eps
        } else {
            -eps
        };
        d
    }

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

    fn reflect(self, normal: Vec3) -> Self {
        self - 2.0 * normal * self.dot(normal)
    }

    fn sin(self) -> Vec3 {
        Self::new(self.x.sin(), self.y.sin(), self.z.sin())
    }

    fn sign_bit(self) -> Vec3 {
        vec3(
            if self.x >= 0.0 { 1.0 } else { 0.0 },
            if self.y >= 0.0 { 1.0 } else { 0.0 },
            if self.z >= 0.0 { 1.0 } else { 0.0 },
        )
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

//// legacy code
