pub use common_no_std::svo::*;
use std::path::Path;


#[cfg(feature="svo-vox")]
pub fn load_svo_from_vox<const BLOCK_DIM: usvo, const LEVEL_COUNT: usize>(path: &Path) -> Svo<Vec<usvo>, BLOCK_DIM, LEVEL_COUNT> {
    let data = dot_vox::load(path.to_str().unwrap()).unwrap();
    todo!()
}
