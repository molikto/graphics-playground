use super::shared::*;
use common::{
    material::*,
    svt::{usvt, BlockRayIntersectionInfo},
    *, shader::heat,
};

#[derive(PartialEq, Eq)]
#[repr(C)]
pub enum RenderMode {
    RayTracing,
    DotNShading,
    Iteration,
}

pub const RENDER_MODE: RenderMode = RenderMode::Iteration;

const MAX_RAY_DEPTH: u32 = 15;

pub const MAX_ITERATION: u32 = 256;
pub const MAX_HEAT_ITERATION: u32 = 128;

// from Ray Tracing in One Weekend
pub fn skybox0(ray: &Ray3) -> Vec3 {
    let unit = ray.dir;
    let t = 0.5 * (unit.y + 1.0) as f32;
    (1.0 - t) * Vec3::new(1.0, 1.0, 1.0) + t * Vec3::new(0.5, 0.7, 1.0)
}
