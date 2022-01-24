use core::ops::{Add, Div, Mul, Rem, Neg};

pub use glam::*;

#[cfg(target_arch = "spirv")]
pub use num_traits::*;

use num_traits::Zero;

pub trait MyVecExt {
    // TODO CPU version with indexing
    fn get(self, a: usize) -> f32;
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
    fn omit_by_nonzero(self, mask: Sign3) -> Vec2;
    fn reflect(self, normal: Self) -> Self;
    fn sin(self) -> Vec3;
    fn sign(self) -> Sign3;
    fn leq(self, other: f32) -> Sign3;
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

    fn sign(self) -> Sign3 {
        let signum = self.signum();
        Sign3::new(signum.x, signum.y, signum.z)
    }

    fn leq(self, other: f32) -> Sign3 {
        Sign3::new_nonneg(self.x <= other, self.y <= other, self.z <= other)
    }

    fn omit_by_nonzero(self, mask: Sign3) -> Vec2 {
        if mask.x() {
            self.yz()
        } else if mask.y() {
            self.xz()
        } else {
            self.xy()
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Bool3(u32);

impl Bool3 {}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Sign3(u32);

impl Sign3 {
    pub const ZERO: Sign3 = Sign3(0);

    pub fn to_face3(self) -> Face3 {
        Face3(self.0)
    }


    pub fn new_nonneg(x: bool, y: bool, z: bool) -> Sign3 {
        Self((x as u32) << 1 | (y as u32) << 3 | (z as u32) << 5)
    }
    pub fn new(x: f32, y: f32, z: f32) -> Sign3 {
        Self(
            if x < 0.0 {
                1
            } else if x > 0.0 {
                2
            } else {
                0
            } | if y < 0.0 {
                4
            } else if y > 0.0 {
                8
            } else {
                0
            } | if z < 0.0 {
                16
            } else if z > 0.0 {
                32
            } else {
                0
            },
        )
    }
    pub fn non_neg(self) -> Self {
        Self(self.0 & 0b101010)
    }

    pub fn non_neg_mul(self, other: Vec3) -> Vec3 {
        // self.non_neg().as_vec3() + other
        let mut out = other;
        if self.0 & 2 == 0 {
            out.x = 0.0;
        };
        if self.0 & 8 == 0 {
            out.y = 0.0;
        };
        if self.0 & 32 == 0 {
            out.z = 0.0;
        };
        out
    }

    pub fn non_neg_add(self, other: Vec3) -> Vec3 {
        // self.non_neg().as_vec3() + other
        let mut out = other;
        if self.0 & 2 != 0 {
            out.x +=1.0;
        };
        if self.0 & 8 != 0 {
            out.y += 1.0;
        };
        if self.0 & 32 != 0 {
            out.z += 1.0;
        };
        out
    }

    pub fn as_vec3(self) -> Vec3 {
        let x = if self.0 & 1 != 0 {
            -1.0
        } else if self.0 & 2 != 0 {
            1.0
        } else {
            0.0
        };
        let y = if self.0 & 4 != 0 {
            -1.0
        } else if self.0 & 8 != 0 {
            1.0
        } else {
            0.0
        };
        let z = if self.0 & 16 != 0 {
            -1.0
        } else if self.0 & 32 != 0 {
            1.0
        } else {
            0.0
        };
        vec3(x, y, z)
    }
    pub fn xn(self) -> bool {
        self.0 & 1 != 0
    }
    pub fn xp(self) -> bool {
        self.0 & 2 != 0
    }
    pub fn x(self) -> bool {
        self.0 & (1 | 2) != 0
    }
    pub fn yn(self) -> bool {
        self.0 & 4 != 0
    }
    pub fn yp(self) -> bool {
        self.0 & 8 != 0
    }
    pub fn y(self) -> bool {
        self.0 & (4 | 8) != 0
    }
    pub fn zn(self) -> bool {
        self.0 & 16 != 0
    }
    pub fn zp(self) -> bool {
        self.0 & 32 != 0
    }
    pub fn z(self) -> bool {
        self.0 & (32 | 16) != 0
    }
    pub fn set_xn(&mut self, xn: bool) {
        self.0 = (self.0 & !1) | (if xn { 1 } else { 0 });
    }

    pub fn set_xp(&mut self, xp: bool) {
        self.0 = (self.0 & !2) | (if xp { 2 } else { 0 });
    }

    pub fn set_yn(&mut self, yn: bool) {
        self.0 = (self.0 & !4) | (if yn { 4 } else { 0 });
    }

    pub fn set_yp(&mut self, yp: bool) {
        self.0 = (self.0 & !8) | (if yp { 8 } else { 0 });
    }

    pub fn set_zn(&mut self, zn: bool) {
        self.0 = (self.0 & !16) | (if zn { 16 } else { 0 });
    }
    pub fn set_zp(&mut self, zp: bool) {
        self.0 = (self.0 & !32) | (if zp { 32 } else { 0 });
    }
}

impl Neg for Sign3 {
    type Output = Sign3;

    // TODO this is bad
    fn neg(self) -> Sign3 {
        let v= self.as_vec3();
        Self::new(-v.x, -v.y, -v.z)
    }
}

impl Mul<Sign3> for Sign3 {
    type Output = Sign3;

    // TODO this is bad
    fn mul(self, rhs: Sign3) -> Self::Output {
        let v = self.as_vec3() * rhs.as_vec3();
        Self::new(v.x, v.y, v.z)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Face3(u32);

impl Face3 {
    pub const NONE: Face3 = Face3(0);
    pub const FULL: Face3 = Face3::new(true, true, true, true, true, true);
    pub const fn from_raw(u: u32) -> Self {
        Face3(u)
    }
    pub const fn to_raw(self) -> u32 {
        self.0
    }
    pub const fn new(xn: bool, xp: bool, yn: bool, yp: bool, zn: bool, zp: bool) -> Self {
        Face3(
            (if xn { 1 } else { 0 })
                | (if xp { 2 } else { 0 })
                | (if yn { 4 } else { 0 })
                | (if yp { 8 } else { 0 })
                | (if zn { 16 } else { 0 })
                | (if zp { 32 } else { 0 }),
        )
    }

    pub fn xn(self) -> bool {
        self.0 & 1 != 0
    }
    pub fn xp(self) -> bool {
        self.0 & 2 != 0
    }
    pub fn x(self) -> bool {
        self.0 & (1 | 2) != 0
    }
    pub fn yn(self) -> bool {
        self.0 & 4 != 0
    }
    pub fn yp(self) -> bool {
        self.0 & 8 != 0
    }
    pub fn y(self) -> bool {
        self.0 & (4 | 8) != 0
    }
    pub fn zn(self) -> bool {
        self.0 & 16 != 0
    }
    pub fn zp(self) -> bool {
        self.0 & 32 != 0
    }
    pub fn z(self) -> bool {
        self.0 & (32 | 16) != 0
    }
    pub fn set_xn(&mut self, xn: bool) {
        self.0 = (self.0 & !1) | (if xn { 1 } else { 0 });
    }

    pub fn set_xp(&mut self, xp: bool) {
        self.0 = (self.0 & !2) | (if xp { 2 } else { 0 });
    }

    pub fn set_yn(&mut self, yn: bool) {
        self.0 = (self.0 & !4) | (if yn { 4 } else { 0 });
    }

    pub fn set_yp(&mut self, yp: bool) {
        self.0 = (self.0 & !8) | (if yp { 8 } else { 0 });
    }

    pub fn set_zn(&mut self, zn: bool) {
        self.0 = (self.0 & !16) | (if zn { 16 } else { 0 });
    }
    pub fn set_zp(&mut self, zp: bool) {
        self.0 = (self.0 & !32) | (if zp { 32 } else { 0 });
    }

    pub fn and(self, o: Face3) -> Face3 {
        Face3::from_raw(self.0 & o.0)
    }

    pub fn count(self) -> u32 {
        let mut c = 0;
        let mut n = self.0;
        while n != 0 {
            if n & 1 != 0 {
                c += 1;
            }
            n = n >> 1;
        }
        c
    }
}
