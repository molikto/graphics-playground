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
use bevy_common::RevertBox;
use bsoky_no_std::*;
use common::math::svt::usvt;
use common::math::vec::Vec3;

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


pub struct EnvRenderPlugin;

impl Plugin for EnvRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<CustomMaterial>::default())
        .add_startup_system(create_simple_debug_objects);
    }
}

fn create_simple_debug_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    let svt = super::create_svt::debug_create_rsvo();
    println!("total dim {}\nblock count {}\nmemory used {}\nmemory ratio {}", MySvtMut::TOTAL_DIM, svt.block_count(), svt.memory_used(), svt.memory_ratio());
    let mesh = meshes.add(RevertBox::zero_with_size(Vec3::splat(MySvtMut::TOTAL_DIM as f32)).into());
    let material = materials.add(CustomMaterial { svt });
    commands.spawn_bundle(MaterialMeshBundle::<CustomMaterial> {
        mesh,
        material,
        ..Default::default()
    });
}
