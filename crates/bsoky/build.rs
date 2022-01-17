use spirv_builder::{MetadataPrintout, SpirvBuilder, SpirvMetadata};
use std::{env, fs, path::{Path, PathBuf}};

fn cwd() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn shader_folder() -> PathBuf {
    let path_ext = cwd().join("assets").join("env");
    return path_ext;
}

fn spirv_cross(path_ext: &Path) {
    use std::{process::Command};
    let path_frag = shader_folder().join("shader_frag.glsl");
    let path_vert = shader_folder().join("shader_vert.glsl");
    Command::new("spirv-cross").args([
        path_ext.to_str().unwrap(),
        "--vulkan-semantics",
        "--stage",
        "frag",
        "--output",
        path_frag.to_str().unwrap(),
    ]).spawn().unwrap();
    Command::new("spirv-cross").args([
        path_ext.to_str().unwrap(),
        "--vulkan-semantics",
        "--stage",
        "vert",
        "--output",
        path_vert.to_str().unwrap(),
    ]).spawn().unwrap();
}

fn main() {
    let shader_result = SpirvBuilder::new(
        cwd().parent().unwrap().join("bsoky-shader"),
        "spirv-unknown-spv1.5",
    )
    //.print_metadata(MetadataPrintout::Full)
    //.spirv_metadata(SpirvMetadata::Full)
    .release(true)
    .build()
    .unwrap();
    let path = shader_result.module.unwrap_single();
    let spv_path0 = shader_folder().join("shader.spv");
    let spv_path = spv_path0.as_path();
    fs::copy(path, spv_path).unwrap();
    //spirv_cross(spv_path);
}