use spirv_builder::{CompileResult, MetadataPrintout, SpirvBuilder, SpirvMetadata};
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
};

#[derive(Clone)]
pub struct RustGpuBuild {
    pub source: PathBuf,
    pub assets: PathBuf,
    pub cross_to_glsl: bool
}

fn spirv_cross(path_ext: &Path) {
    use std::process::Command;
    Command::new("spirv-cross")
        .args([
            path_ext.to_str().unwrap(),
            "--vulkan-semantics",
            "--output",
            path_ext
                .to_path_buf()
                .with_extension("spv.glsl")
                .to_str()
                .unwrap(),
        ])
        .spawn()
        .unwrap();
}

impl RustGpuBuild {
    fn copy_to_assets(&self, result: CompileResult) {
        match result.module {
            spirv_builder::ModuleResult::SingleModule(path) => {
                let spv_path0 = self.assets.join("shader.spv");
                let spv_path = spv_path0.as_path();
                fs::copy(path, spv_path).unwrap();
                if self.cross_to_glsl {
                    spirv_cross(spv_path);
                }
            }
            spirv_builder::ModuleResult::MultiModule(map) => {
                map.into_iter().for_each(|(name, path)| {
                    let spv_path0 = self.assets.join(name).with_extension("spv");
                    let spv_path = spv_path0.as_path();
                    fs::copy(path, spv_path).unwrap();
                    if self.cross_to_glsl {
                        spirv_cross(spv_path);
                    }
                });
            }
        }
    }

    pub fn spawn(self) {
        thread::spawn(move || {
            self.run();
        });
    }

    pub fn run(self) {
        let self_clone = self.clone();
        let res = SpirvBuilder::new(self.source.clone(), "spirv-unknown-spv1.5")
            .capability(spirv_builder::Capability::Int8)
            .capability(spirv_builder::Capability::Int16)
            .capability(spirv_builder::Capability::Int64)
            .multimodule(true) // this is mostly because it's easier to switch between reference and our own
            .print_metadata(MetadataPrintout::None)
            .spirv_metadata(SpirvMetadata::Full)
            .target_dir(self.source.clone().join("target").as_path().to_str().unwrap())
            .release(true)
            .watch(move |result| {
                self_clone.copy_to_assets(result)
            })
            .unwrap();
        self.copy_to_assets(res);
    }
}
