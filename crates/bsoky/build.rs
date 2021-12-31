use spirv_builder::{MetadataPrintout, SpirvBuilder};
use std::{env, fs, path::Path};

// fn spirv_cross(path_ext: &Path) {
// use std::{os::unix::prelude::CommandExt, process::Command};
//     let path_frag = PathBuf::from(format!("{}/assets/env/shader.frag.glsl", env!("CARGO_MANIFEST_DIR")));
//     let path_vert = PathBuf::from(format!("{}/assets/env/shader.vert.glsl", env!("CARGO_MANIFEST_DIR")));
//     Command::new("spirv-cross").args([
//         path_ext.to_str().unwrap(),
//         "--vulkan-semantics",
//         "--stage",
//         "frag",
//         "--output",
//         path_frag.to_str().unwrap(),
//     ]).exec();
//     Command::new("spirv-cross").args([
//         path_ext.to_str().unwrap(),
//         "--vulkan-semantics",
//         "--stage",
//         "vert",
//         "--output",
//         path_vert.to_str().unwrap(),
//     ]).exec();
// }
fn main() {
    let cwd = Path::new(env!("CARGO_MANIFEST_DIR"));
    let shader_result = SpirvBuilder::new(
        cwd.parent().unwrap().join("bsoky-shader"),
        "spirv-unknown-spv1.5",
    )
    .capability(spirv_builder::Capability::VariablePointers)
    .capability(spirv_builder::Capability::Int16)
    .print_metadata(MetadataPrintout::Full)
    .build()
    .unwrap();
    let path = shader_result.module.unwrap_single();
    let path_ext = cwd.join("assets").join("env").join("shader.spv");
    fs::copy(path, path_ext).unwrap();
    //spirv_cross(path_ext);
}