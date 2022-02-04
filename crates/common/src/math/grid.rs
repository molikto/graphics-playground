use glam::UVec3;

use super::aabb::*;
use super::sign::*;


#[derive(Copy, Clone)]
pub struct BlockRayIntersectionInfo {
    pub mask: Sign3,
    pub t: f32,
}

pub trait Grid3 {
}

// #[derive(Copy, Clone)]
// pub struct GidRayHitInfo<T : Copy + Clone> {
//     pub in_at: BlockRayIntersectionInfo,
//     pub data: T,
//     pub out_at: BlockRayIntersectionInfo
// }

#[derive(Copy, Clone)]
pub struct TraverseRayInfo<T : Copy + Clone> {
    pub in_info: BlockRayIntersectionInfo,
    pub pos: UVec3,
    pub data: T
}

impl <T : Copy + Clone> TraverseRayInfo<T> {
    pub fn first_face_aap(self) -> Aap3 {
        Aap3::from_unit_pos_face(self.pos.as_vec3(), self.in_info.mask.first_face())
    }
}