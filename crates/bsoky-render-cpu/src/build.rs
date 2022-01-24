use spirv_builder::{MetadataPrintout, SpirvBuilder, SpirvMetadata, CompileResult};
use std::{env, fs, path::{Path, PathBuf}, time::Duration};

fn cpu_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf().parent().unwrap().join("bsoky-render-cpu")
}

fn gpu_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf().parent().unwrap().join("bsoky-render-gpu")
}

fn shader_folder() -> PathBuf {
    let path_ext = cpu_root().join("assets").join("voxel_render");
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

fn copy_to_assets(result: CompileResult) {
    let path = result.module.unwrap_single();
    let spv_path0 = shader_folder().join("shader.spv");
    let spv_path = spv_path0.as_path();
    fs::copy(path, spv_path).unwrap();
    //spirv_cross(spv_path);
}

pub fn main() {
    let res = SpirvBuilder::new(
        gpu_root(),
        "spirv-unknown-spv1.5",
    )
    .capability(spirv_builder::Capability::Int8)
    .capability(spirv_builder::Capability::Int16)
    .capability(spirv_builder::Capability::Int64)
    .print_metadata(MetadataPrintout::None)
    .spirv_metadata(SpirvMetadata::Full)
    .release(true)
    .watch(|result| {
        println!("shader changed");
        copy_to_assets(result)
    }).unwrap();
    copy_to_assets(res);
}