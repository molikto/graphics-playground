use common_no_std::Vec3;


#[derive(Copy, Clone, Default)]
pub struct ShadeVoxelState {
  pub opaque_visibility: Vec3,
  pub water_depth: f32,
  pub color: Vec3,
}

