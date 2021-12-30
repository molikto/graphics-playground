use crate::math::*;

use num_traits::Float;

pub trait SrgbColorSpace {
    fn linear_to_nonlinear_srgb(self) -> Self;
    fn nonlinear_to_linear_srgb(self) -> Self;
}

// source: https://entropymine.com/imageworsener/srgbformula/
impl SrgbColorSpace for f32 {
    fn linear_to_nonlinear_srgb(self) -> f32 {
        if self <= 0.0 {
            return self;
        }

        if self <= 0.0031308 {
            self * 12.92 // linear falloff in dark values
        } else {
            (1.055 * self.powf(1.0 / 2.4)) - 0.055 // gamma curve in other area
        }
    }

    fn nonlinear_to_linear_srgb(self) -> f32 {
        if self <= 0.0 {
            return self;
        }
        if self <= 0.04045 {
            self / 12.92 // linear falloff in dark values
        } else {
            ((self + 0.055) / 1.055).powf(2.4) // gamma curve in other area
        }
    }
}

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Clone, Copy)]
pub struct RgbaLinear(pub Vec4);

impl RgbaLinear {
    #[inline]
    pub fn lerp(a: RgbaLinear, b: RgbaLinear, t: f32) -> RgbaLinear {
        RgbaLinear(Vec4::lerp(a.0, b.0, t))
    }

    pub fn with_a(self: Self, a: f32) -> RgbaLinear {
        RgbaLinear(Vec4::new(self.0.x, self.0.y, self.0.z, a))
    }
    #[inline]
    pub fn to_rgba(self: Self) -> Rgba {
        Rgba(vec4(
            (self.0.x).linear_to_nonlinear_srgb(),
            (self.0.y).linear_to_nonlinear_srgb(),
            (self.0.z).linear_to_nonlinear_srgb(),
            self.0.w,
        ))
    }
}

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Clone, Copy)]
pub struct RgbaLinearPremultiplied(pub Vec4);

impl RgbaLinearPremultiplied {
    #[inline]
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> RgbaLinearPremultiplied {
        Self(Vec4::new(r, g, b, a))
    }
    #[inline]
    pub fn to_rgba(self: Self) -> Rgba {
        Rgba(vec4(
            (self.0.x / self.0.w).linear_to_nonlinear_srgb(),
            (self.0.y / self.0.w).linear_to_nonlinear_srgb(),
            (self.0.z / self.0.w).linear_to_nonlinear_srgb(),
            self.0.w,
        ))
    }
}

impl core::ops::AddAssign<RgbaLinearPremultiplied> for RgbaLinearPremultiplied {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl core::ops::MulAssign<f32> for RgbaLinearPremultiplied {
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
    }
}

impl core::ops::DivAssign<f32> for RgbaLinearPremultiplied {
    fn div_assign(&mut self, rhs: f32) {
        self.0 /= rhs;
    }
}

impl core::ops::Mul<f32> for RgbaLinearPremultiplied {
    type Output = RgbaLinearPremultiplied;
    fn mul(self, rhs: f32) -> Self::Output {
        let mut clone = self;
        clone *= rhs;
        clone
    }
}
impl core::ops::Div<f32> for RgbaLinearPremultiplied {
    type Output = RgbaLinearPremultiplied;
    fn div(self, rhs: f32) -> Self::Output {
        let mut clone = self;
        clone /= rhs;
        clone
    }
}

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Clone, Copy)]
pub struct Rgba(pub Vec4);

impl Rgba {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Rgba {
        Self(Vec4::new(r, g, b, a))
    }
    #[inline]
    pub fn to_rgba_linear(self: Self) -> RgbaLinear {
        RgbaLinear(vec4(
            self.0.x.nonlinear_to_linear_srgb(),
            self.0.y.nonlinear_to_linear_srgb(),
            self.0.z.nonlinear_to_linear_srgb(),
            self.0.w,
        ))
    }

    pub fn with_a(self: Self, a: f32) -> Rgba {
        Rgba(Vec4::new(self.0.x, self.0.y, self.0.z, a))
    }

    pub fn a(self) -> f32 {
        self.0.w
    }
}
