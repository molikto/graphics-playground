use super::super::math::*;


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

#[derive(Copy, Clone, Default)]
pub struct MeshUniform {
  pub transform: Mat4,
}


// TODO change order of things in this will make it dont work???!!!
#[cfg_attr(not(target_arch = "spirv"), derive(bevy_crevice::std430::AsStd430, bevy_crevice::std140::AsStd140, Debug))]
pub struct RayTracingViewInfo {
    pub camera_pos: Vec3,
    pub camera_look: Vec3,
    pub camera_h: Vec3,
    pub camera_v: Vec3,
    pub not_used: UVec2,
    pub time: f32,
    pub frame_index: u32,
} 
impl RayTracingViewInfo {
  pub fn get_ray(&self, frag_coord: Vec2) -> Ray3 {
    let pos = self.camera_pos;
    let dir = self.camera_look + frag_coord.x * self.camera_h + frag_coord.y * self.camera_v;
    Ray3 { pos, dir }
  }
}
