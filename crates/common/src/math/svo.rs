pub use common_no_std::svo::*;
use std::path::Path;


#[cfg(feature="svo-vox")]
pub fn load_svo_from_vox(path: &Path) -> Svo<> {
    let data = dot_vox::load(path.to_str().unwrap()).unwrap();
}
