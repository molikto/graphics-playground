use std::{path::Path};

use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    pbr::MaterialPipeline,
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::{PrepareAssetError, RenderAsset},
        render_resource::*,
        renderer::RenderDevice, render_phase::TrackedRenderPass,
    }, utils::Instant,
};
use bsoky_no_std::*;
use bevy_crevice::std430::*;
use common::math::svt::usvt;

#[derive(TypeUuid)]
#[uuid = "4ee9c363-1124-4113-890e-199d81b00281"]
pub struct CustomMaterial {
    pub svt: MySvtMut,
}

#[derive(Clone, TypeUuid)]
#[uuid = "4ee9c363-1124-4113-890e-199d81b00281"]
pub struct ExtractedCustomMaterial {
    pub svt: Vec<usvt>,
}

#[derive(Clone)]
pub struct GpuCustomMaterial {
    bind_group: BindGroup,
}

impl RenderAsset for CustomMaterial {
    type ExtractedAsset = ExtractedCustomMaterial;
    type PreparedAsset = GpuCustomMaterial;
    type Param = (SRes<RenderDevice>, SRes<MaterialPipeline<Self>>);
    fn extract_asset(&self) -> Self::ExtractedAsset {
        ExtractedCustomMaterial {
            svt: self.svt.mem.clone(),
        }
    }


    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, material_pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let svt_contents: &[u8] = unsafe { extracted_asset.svt.as_slice().align_to::<u8>().1 };
        let svt_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: svt_contents,
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: svt_buffer.as_entire_binding(),
                },
            ],
            label: None,
            layout: &material_pipeline.material_layout,
        });

        Ok(GpuCustomMaterial {
            bind_group,
        })
    }
}

impl Material for CustomMaterial {
    fn vertex_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load(Path::new("env").join("shader.spv")))
    }
    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load(Path::new("env").join("shader.spv")))
    }

    fn before_render(pass: &mut TrackedRenderPass) {
        let constant_content = EnvShaderUniform {
            time: Instant::now().elapsed().as_millis() as f32
        };
        pass.set_push_constants(ShaderStages::FRAGMENT, 0, &constant_content.as_std430().as_bytes());
    }


    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: None,
        })
    }
}
