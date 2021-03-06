
// from https://github.com/EmbarkStudios/rust-gpu
//! Ported to Rust from <https://github.com/Tw1ddle/Sky-Shader/blob/master/src/shaders/glsl/sky.fragment>

use core::f32::consts::PI;
use glam::{const_vec3, vec3, Vec3};

pub fn saturate(x: f32) -> f32 {
    x.max(0.0).min(1.0)
}

pub fn pow(v: Vec3, power: f32) -> Vec3 {
    vec3(v.x.powf(power), v.y.powf(power), v.z.powf(power))
}

pub fn exp(v: Vec3) -> Vec3 {
    vec3(v.x.exp(), v.y.exp(), v.z.exp())
}

/// Based on: <https://seblagarde.wordpress.com/2014/12/01/inverse-trigonometric-functions-gpu-optimization-for-amd-gcn-architecture/>
pub fn acos_approx(v: f32) -> f32 {
    let x = v.abs();
    let mut res = -0.155972 * x + 1.56467; // p(x)
    res *= (1.0f32 - x).sqrt();

    if v >= 0.0 {
        res
    } else {
        PI - res
    }
}

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    // Scale, bias and saturate x to 0..1 range
    let x = saturate((x - edge0) / (edge1 - edge0));
    // Evaluate polynomial
    x * x * (3.0 - 2.0 * x)
}
// Note: This cfg is incorrect on its surface, it really should be "are we compiling with std", but
// we tie #[no_std] above to the same condition, so it's fine.
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::Ray3;

const DEPOLARIZATION_FACTOR: f32 = 0.035;
const MIE_COEFFICIENT: f32 = 0.005;
const MIE_DIRECTIONAL_G: f32 = 0.8;
const MIE_K_COEFFICIENT: Vec3 = const_vec3!([0.686, 0.678, 0.666]);
const MIE_V: f32 = 4.0;
const MIE_ZENITH_LENGTH: f32 = 1.25e3;
const NUM_MOLECULES: f32 = 2.542e25f32;
const PRIMARIES: Vec3 = const_vec3!([6.8e-7f32, 5.5e-7f32, 4.5e-7f32]);
const RAYLEIGH: f32 = 1.0;
const RAYLEIGH_ZENITH_LENGTH: f32 = 8.4e3;
const REFRACTIVE_INDEX: f32 = 1.0003;
const SUN_ANGULAR_DIAMETER_DEGREES: f32 = 0.0093333;
const SUN_INTENSITY_FACTOR: f32 = 1000.0;
const SUN_INTENSITY_FALLOFF_STEEPNESS: f32 = 1.5;
const TURBIDITY: f32 = 2.0;

pub fn tonemap(col: Vec3) -> Vec3 {
    // see https://www.desmos.com/calculator/0eo9pzo1at
    const A: f32 = 2.35;
    const B: f32 = 2.8826666;
    const C: f32 = 789.7459;
    const D: f32 = 0.935;

    let z = pow(col, A);
    z / (pow(z, D) * B + Vec3::splat(C))
}

fn total_rayleigh(lambda: Vec3) -> Vec3 {
    (8.0 * PI.powf(3.0)
        * (REFRACTIVE_INDEX.powf(2.0) - 1.0).powf(2.0)
        * (6.0 + 3.0 * DEPOLARIZATION_FACTOR))
        / (3.0 * NUM_MOLECULES * pow(lambda, 4.0) * (6.0 - 7.0 * DEPOLARIZATION_FACTOR))
}

fn total_mie(lambda: Vec3, k: Vec3, t: f32) -> Vec3 {
    let c = 0.2 * t * 10e-18;
    0.434 * c * PI * pow((2.0 * PI) / lambda, MIE_V - 2.0) * k
}

fn rayleigh_phase(cos_theta: f32) -> f32 {
    (3.0 / (16.0 * PI)) * (1.0 + cos_theta.powf(2.0))
}

fn henyey_greenstein_phase(cos_theta: f32, g: f32) -> f32 {
    (1.0 / (4.0 * PI)) * ((1.0 - g.powf(2.0)) / (1.0 - 2.0 * g * cos_theta + g.powf(2.0)).powf(1.5))
}

fn sun_intensity(zenith_angle_cos: f32) -> f32 {
    let cutoff_angle = PI / 1.95; // Earth shadow hack
    SUN_INTENSITY_FACTOR
        * 0.0f32.max(
            1.0 - (-((cutoff_angle - acos_approx(zenith_angle_cos))
                / SUN_INTENSITY_FALLOFF_STEEPNESS))
                .exp(),
        )
}


pub fn sky(dir: Vec3, sun_position: Vec3) -> Vec3 {
    let up = vec3(0.0, 1.0, 0.0);
    let sunfade = 1.0 - (1.0 - saturate(sun_position.y / 450000.0).exp());
    let rayleigh_coefficient = RAYLEIGH - (1.0 * (1.0 - sunfade));
    let beta_r = total_rayleigh(PRIMARIES) * rayleigh_coefficient;

    // Mie coefficient
    let beta_m = total_mie(PRIMARIES, MIE_K_COEFFICIENT, TURBIDITY) * MIE_COEFFICIENT;

    // Optical length, cutoff angle at 90 to avoid singularity
    let zenith_angle = acos_approx(up.dot(dir).max(0.0));
    let denom = (zenith_angle).cos() + 0.15 * (93.885 - ((zenith_angle * 180.0) / PI)).powf(-1.253);

    let s_r = RAYLEIGH_ZENITH_LENGTH / denom;
    let s_m = MIE_ZENITH_LENGTH / denom;

    // Combined extinction factor
    let fex = exp(-(beta_r * s_r + beta_m * s_m));

    // In-scattering
    let sun_direction = sun_position.normalize();
    let cos_theta = dir.dot(sun_direction);
    let beta_r_theta = beta_r * rayleigh_phase(cos_theta * 0.5 + 0.5);

    let beta_m_theta = beta_m * henyey_greenstein_phase(cos_theta, MIE_DIRECTIONAL_G);
    let sun_e = sun_intensity(sun_direction.dot(up));
    let mut lin = pow(
        sun_e * ((beta_r_theta + beta_m_theta) / (beta_r + beta_m)) * (Vec3::splat(1.0) - fex),
        1.5,
    );

    lin *= Vec3::splat(1.0).lerp(
        pow(
            sun_e * ((beta_r_theta + beta_m_theta) / (beta_r + beta_m)) * fex,
            0.5,
        ),
        saturate((1.0 - up.dot(sun_direction)).powf(5.0)),
    );

    // Composition + solar disc
    let sun_angular_diameter_cos = SUN_ANGULAR_DIAMETER_DEGREES.cos();
    let sundisk = smoothstep(
        sun_angular_diameter_cos,
        sun_angular_diameter_cos + 0.00002,
        cos_theta,
    );
    let mut l0 = 0.1 * fex;
    l0 += sun_e * 19000.0 * fex * sundisk;

    lin + l0
}
