use std::borrow::Cow;

use bevy::{
    core_pipeline::draw_3d_graph::{self, input::VIEW_ENTITY},
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph, SlotInfo, SlotType},
        render_resource::{
            BindGroup, BindGroupLayout, Buffer, CachedPipelineId, FragmentState,
            RenderPipelineCache, RenderPipelineDescriptor, VertexState,
        },
        renderer::{RenderContext, RenderDevice},
        texture::BevyDefault,
        view::{ExtractedView, ViewTarget},
        RenderApp, RenderStage,
    },
    utils::Instant,
};
use bevy_common::CameraProp;
use bevy_crevice::std140::{AsStd140, Std140};
use common::math::*;
use common::*;
use wgpu::{
    util::BufferInitDescriptor, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, BufferUsages,
    ColorTargetState, ColorWrites, LoadOp, MultisampleState, Operations, PrimitiveState,
    RenderPassDescriptor, ShaderStages, TextureFormat, TextureSampleType, TextureViewDimension,
};

// TODO destory buffer and other resources?
use super::data::VoxelMapRenderData;

const WORKGROUP_SIZE: u32 = 8;

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba32Float;

struct ExtractedMapRenderData {
    pub buffer: Buffer,
}

struct RenderPass {
    pipeline: CachedPipelineId,
    texture_layout: BindGroupLayout,
    uniform_layout: BindGroupLayout,
    map_layout: BindGroupLayout,
}

#[derive(Clone)]
struct MapRenderAssets {
    pub map_textures: Handle<Image>,
}

struct GraphLayout {
    render: RenderPass,
    start_time: Instant,
}

#[derive(Default)]
struct Pipeline {
    size: UVec2,
    camera_prop: CameraProp,
    render_texture_bg: Option<BindGroup>,
    render_uniform_bg: Option<BindGroup>,
    render_map_data_bg: Option<BindGroup>,
}

fn create_render_pipeline(
    module: &Handle<Shader>,
    device: &RenderDevice,
) -> (RenderPipelineDescriptor, BindGroupLayout, BindGroupLayout, BindGroupLayout) {
    let texture_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let uniform_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let map_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
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
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    view_dimension: TextureViewDimension::D2,
                    sample_type: TextureSampleType::Float { filterable: false },
                    multisampled: false,
                },
                count: None,
            },
        ],
    });
    let desc = RenderPipelineDescriptor {
        label: None,
        layout: Some(vec![texture_layout.clone(), uniform_layout.clone(), map_layout.clone()]),
        vertex: VertexState {
            shader: module.clone(),
            shader_defs: vec![],
            entry_point: Cow::Owned("vertex_blit".to_string()),
            buffers: vec![],
        },
        fragment: Some(FragmentState {
            shader: module.clone(),
            entry_point: Cow::Owned("fragment_render_blit".to_string()),
            targets: vec![ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: None,
                write_mask: ColorWrites::ALL,
            }],
            shader_defs: vec![],
        }),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
    };
    (desc, texture_layout.clone(), uniform_layout, map_layout)
}

impl FromWorld for GraphLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        let module: Handle<Shader> = asset_server.load("voxel_render/shader.spv");
        let (pipeline_desc, texture_layout, uniform_layout, map_layout) =
            create_render_pipeline(&module, &render_device);
        let mut pipeline_cache = world.get_resource_mut::<RenderPipelineCache>().unwrap();

        let pipeline = pipeline_cache.queue(pipeline_desc);
        let render = RenderPass {
            pipeline,
            texture_layout,
            uniform_layout,
            map_layout,
        };
        GraphLayout {
            render,
            start_time: Instant::now(),
        }
    }
}

fn setup(mut command: Commands, server: Res<AssetServer>) {
    command.insert_resource(MapRenderAssets {
        map_textures: server.load("voxel_render/coal_ore.png"),
    })
}

pub struct VoxelMapRenderPlugin;

impl Plugin for VoxelMapRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<GraphLayout>()
            .init_resource::<Pipeline>()
            .add_system_to_stage(RenderStage::Extract, extract_phase)
            .add_system_to_stage(RenderStage::Queue, queue_phase);

        let render_node = VoxelMapRenderNode::new(&mut render_app.world);
        let mut render_graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();
        let render_graph = render_graph.get_sub_graph_mut(draw_3d_graph::NAME).unwrap();

        render_graph.add_node("voxel_render", render_node);
        render_graph
            .add_node_edge("voxel_render", draw_3d_graph::node::MAIN_PASS)
            .unwrap();
        let input_node_id = render_graph.input_node().unwrap().id;
        render_graph
            .add_slot_edge(
                input_node_id,
                VIEW_ENTITY,
                "voxel_render",
                VoxelMapRenderNode::IN_VIEW,
            )
            .unwrap();
    }
}

fn extract_phase(
    mut commands: Commands,
    map: Res<VoxelMapRenderData>,
    static_assets: Res<MapRenderAssets>,
    device: Res<RenderDevice>,
) {
    let buffer: &[u8] = unsafe { map.data.mem.as_slice().align_to::<u8>().1 };
    let buffer = device.create_buffer_with_data(&BufferInitDescriptor {
        contents: buffer,
        label: None,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    });
    commands.insert_resource(ExtractedMapRenderData { buffer });
    commands.insert_resource(static_assets.clone());
}

fn create_render_uniform_bg(
    device: &RenderDevice,
    layout: &GraphLayout,
    camera_prop: CameraProp,
    size: UVec2,
    frame_index: u32,
) -> BindGroup {
    let uniform = camera_prop.get_ray_tracing_uniform(size, 0.0, 0);
    let compute_uniform_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
        label: None,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        contents: uniform.as_std140().as_bytes(),
    });
    device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &layout.render.uniform_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: compute_uniform_buffer.as_entire_binding(),
        }],
    })
}

fn create_texture_bg(device: &RenderDevice, layout: &GraphLayout, size: UVec2) -> BindGroup {
    let uniform_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
        label: None,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        contents: size.as_std140().as_bytes(),
    });
    device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &layout.render.texture_layout,
        entries: &[BindGroupEntry {
            binding: 1,
            resource: uniform_buffer.as_entire_binding(),
        }],
    })
}

fn queue_phase(
    device: Res<RenderDevice>,
    pipeline_layout: Res<GraphLayout>,
    mut pipeline: ResMut<Pipeline>,
    extracted_data: Res<ExtractedMapRenderData>,
    view: Query<(&ExtractedView, &ViewTarget)>,
    static_assets: Res<MapRenderAssets>,
    gpu_assets: Res<RenderAssets<Image>>,
) {
    let gpu_texture = unwrap_or_return!(gpu_assets.get(&static_assets.map_textures));
    let (extracted_view, view_target) = view.iter().next().unwrap();
    let size = view_target.size;
    let transform = extracted_view.transform;
    let view_projection = extracted_view.projection;
    let camera_prop = CameraProp {
        transform,
        view_projection,
    };
    pipeline.camera_prop = camera_prop;
    let compute_uniform_bg: BindGroup =
        create_render_uniform_bg(&device, &pipeline_layout, camera_prop, size, 0);
    pipeline.render_uniform_bg = Some(compute_uniform_bg);

    pipeline.render_map_data_bg = Some(device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline_layout.render.map_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: extracted_data.buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&gpu_texture.texture_view),
            },
        ],
    }));
    if size != pipeline.size {
        let texture_bg = create_texture_bg(&device, &pipeline_layout, size);
        pipeline.size = size;
        pipeline.render_texture_bg = Some(texture_bg);
    };
}

struct VoxelMapRenderNode {
    query: QueryState<(&'static ViewTarget, &'static ExtractedView)>,
}

impl VoxelMapRenderNode {
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl render_graph::Node for VoxelMapRenderNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(VoxelMapRenderNode::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let (target, _) = match self.query.get_manual(world, view_entity) {
            Ok(query) => query,
            Err(_) => return Ok(()), // No window
        };
        let graph_layout = world.get_resource::<GraphLayout>().unwrap();
        let pipeline_cache = world.get_resource::<RenderPipelineCache>().unwrap();
        let layout_pipeline =
            unwrap_or_return_val!(pipeline_cache.get(graph_layout.render.pipeline), Ok(()));
        let pipeline = world.get_resource::<Pipeline>().unwrap();
        let render_uniform_bg = unwrap_or_return_val!(pipeline.render_uniform_bg.as_ref(), Ok(()));
        let mut render_pass =
            render_context
                .command_encoder
                .begin_render_pass(&RenderPassDescriptor {
                    label: None,
                    color_attachments: &[target.get_color_attachment(Operations {
                        load: LoadOp::Load,
                        store: true,
                    })],
                    depth_stencil_attachment: None,
                });
        render_pass.set_pipeline(layout_pipeline);
        render_pass.set_bind_group(0, pipeline.render_texture_bg.as_ref().unwrap(), &[]);
        render_pass.set_bind_group(1, render_uniform_bg, &[]);
        render_pass.set_bind_group(2, pipeline.render_map_data_bg.as_ref().unwrap(), &[]);
        render_pass.draw(0..3, 0..2);
        Ok(())
    }
}
