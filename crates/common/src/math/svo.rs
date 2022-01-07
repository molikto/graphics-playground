pub use common_no_std::svo::*;
use std::path::Path;

use common_no_std::material::Material;



#[cfg(feature="svo-vox")]
pub fn load_svo_from_vox<const BLOCK_DIM: usvo, const LEVEL_COUNT: usize>(path: &Path) -> (SvoMut<BLOCK_DIM, LEVEL_COUNT>, Vec<Material>) {
    let mut svo: SvoMut<BLOCK_DIM, LEVEL_COUNT> = Svo::init(0);
    let data = dot_vox::load(path.to_str().unwrap()).unwrap();
    let mut materials: Vec<Material> = Vec::new();
    for model in data.models {
        for v in model.voxels {
            svo.set(Usvo3::new(v.x as usvo, v.y as usvo, v.z as usvo), (v.i + 1) as usvo);
        }
    }
    (svo, materials)
}
