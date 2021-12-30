use super::super::math::*;

#[cfg(not(target_arch = "spirv"))]
use bevy_crevice::std140::AsStd140;

#[cfg_attr(not(target_arch = "spirv"), derive(AsStd140))]
#[derive(Copy, Clone, Default)]
pub struct ViewUniform {
  pub view_proj: Mat4,
  pub inverse_view: Mat4,
  pub projection: Mat4,
  pub world_position: Vec3,
  pub near: f32,
  pub far: f32,
  pub width: f32,
  pub height: f32,
}

#[cfg_attr(not(target_arch = "spirv"), derive(AsStd140))]
#[derive(Copy, Clone, Default)]
pub struct MeshUniform {
  pub transform: Mat4,
}


#[cfg_attr(not(target_arch = "spirv"), derive(AsStd140))]
#[derive(Copy, Clone, Default)]
pub struct MapRenderData {
    pub size: UVec3,
    pub aabb: Aabb3,
}
