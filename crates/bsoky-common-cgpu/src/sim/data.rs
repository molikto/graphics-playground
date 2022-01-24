use common::{svt::*, Face3};

use super::config::VoxelMaterial;



pub type Seed = (u64, u64);



#[derive(PartialEq, Eq, Clone, Copy)]
pub struct VoxelData(u32);


impl VoxelData {
    pub const fn new(material: VoxelMaterial) -> Self {
        let mut data = 0;
        data |= material as u32;
        VoxelData(data)
    }
    pub const EMPTY: Self = VoxelData::new(VoxelMaterial::Air);
    pub const DEBUG_STONE: Self = VoxelData::new(VoxelMaterial::Stone);

    pub fn as_raw(self) -> u32 {
        self.0
    }
}


impl From<usvt> for VoxelData {
    fn from(u: usvt) -> Self {
        Self(u)
    }
}

impl Into<usvt> for VoxelData {
    fn into(self) -> usvt {
        self.0
    }
}
impl SvtData for VoxelData {
    const EMPTY: Self = Self::EMPTY;
}