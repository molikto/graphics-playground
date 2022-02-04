use core::ops::{Neg, Mul};

use glam::*;

use super::axis::*;

use super::face_mask::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Sign3(u32);

impl Sign3 {
    pub const ZERO: Sign3 = Sign3(0);

    pub fn to_face_mask(self) -> FaceMask3 {
        FaceMask3::from_raw(self.0)
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

    pub fn first_face(&self) -> Face3 {
        if self.xp() {
            Face3 {
                axis: Axis3::X,
                p: true
            }
        } else if self.xn() {
            Face3 {
                axis: Axis3::X,
                p: false
            }
        } else if self.yp() {
            Face3 {
                axis: Axis3::Y,
                p: true
            }
        } else if self.yn() {
            Face3 {
                axis: Axis3::Y,
                p: false
            }
        } else if self.zp() {
            Face3 {
                axis: Axis3::Z,
                p: true
            }
        } else {
            Face3 {
                axis: Axis3::Z,
                p: false
            }
        }
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
