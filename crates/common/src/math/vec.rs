use core::ops::{Add, Div, Mul, Rem};
use super::axis::*;

use super::sign::*;

pub use glam::*;

#[cfg(target_arch = "spirv")]
pub use num_traits::*;

use num_traits::Zero;

pub trait MyVecExt {
    // TODO CPU version with indexing
    fn try_normalize_or(self, other: Self) -> Self;
    fn sum(self) -> f32;
}

impl MyVecExt for Vec3 {
    fn sum(self) -> f32 {
        self.dot(Self::splat(1.0))
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

    fn try_normalize_or(self, other: Self) -> Self {
        let rcp = self.length_recip();
        if rcp.is_finite() && rcp > 0.0 {
            self * rcp
        } else {
            other
        }
    }
}

pub trait MyVec2Ext {
    fn get(self, a: Axis2) -> f32;
    fn extend_axis(self, a: Axis3, s: f32) -> Vec3;
}

impl MyVec2Ext for Vec2 {
    #[inline]
    fn extend_axis(self, a: Axis3, s: f32) -> Vec3 {
        if a == Axis3::X {
            Vec3::new(s, self.x, self.y)
        } else if a == Axis3::Y {
            Vec3::new(self.x, s, self.y)
        } else if a == Axis3::Z {
            Vec3::new(self.x, self.y, s)
        } else {
            panic!()
        }
    }

    fn get(self, a: Axis2) -> f32 {
        if a == Axis2::X {
            self.x
        } else {
            self.y
        }
    }
}

pub trait MyVec3Ext {
    fn get(self, a: Axis3) -> f32;
    fn omit_axis(self, a: Axis3) -> Vec2;
    fn omit_by_nonzero(self, mask: Sign3) -> Vec2;
    fn reflect(self, normal: Self) -> Self;
    fn sin(self) -> Vec3;
    fn sign(self) -> Sign3;
    fn margin_unit(self, margin: f32) -> Sign3;
    fn leq(self, other: f32) -> Sign3;
    fn de_eps(self, eps: f32) -> Vec3;
}

impl MyVec3Ext for Vec3 {
    
    #[inline]
    fn get(self, a: Axis3) -> f32 {
        if a == Axis3::X {
            self.x
        } else if a == Axis3::Y {
            self.y
        } else {
            self.z
        }
    }

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
    fn omit_axis(self, a: Axis3) -> Vec2 {
        if a == Axis3::X {
            self.yz()
        } else if a == Axis3::Y {
            self.xz()
        } else if a == Axis3::Z {
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

    fn margin_unit(self, margin: f32) -> Sign3 {
        let mut res_faces = Sign3::ZERO;
        let pos_margin = 1.0 - margin;
        if self.x < margin {
            res_faces.set_xn(true)
        } else if self.x > pos_margin {
            res_faces.set_xp(true)
        }
        if self.y < margin {
            res_faces.set_yn(true)
        } else if self.y > pos_margin {
            res_faces.set_yp(true)
        }
        if self.z < margin {
            res_faces.set_zn(true)
        } else if self.z > pos_margin {
            res_faces.set_zp(true)
        }
        res_faces
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_margin() {
        assert_eq!(
        Vec3::new(0.95, 0.5, 0.0).margin_unit(0.1).as_vec3(),
        Vec3::new(1.0, 0.0, -1.0))
    }

}
