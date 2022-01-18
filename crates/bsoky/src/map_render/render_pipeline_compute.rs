use bevy::{
    core_pipeline::draw_3d_graph::{self, input::VIEW_ENTITY},
    prelude::*,
    render::{
        render_graph::{self, RenderGraph, SlotInfo, SlotType},
        renderer::{RenderContext, RenderDevice},
        texture::BevyDefault,
        view::{ExtractedView, ViewTarget},
        RenderApp, RenderStage,
    },
    utils::Instant,
};
use bevy_crevice::std140::{AsStd140, Std140};
use bsoky_shader::*;
use common::math::*;
use common::*;
use wgpu::{
    include_spirv_raw,
    util::{BufferInitDescriptor, DeviceExt},
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
    BufferBindingType, BufferUsages, ColorTargetState, ColorWrites, ComputePassDescriptor,
    ComputePipeline, ComputePipelineDescriptor, Device, Extent3d, FilterMode, FragmentState,
    LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor, PrimitiveState,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, SamplerBindingType,
    SamplerDescriptor, ShaderModule, ShaderStages, StorageTextureAccess, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureViewDescriptor, TextureViewDimension, VertexState,
};

use super::data::VoxelMapRenderData;

const WORKGROUP_SIZE: u32 = 8;

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba32Float;

struct ExtractedMapRenderData {
    pub buffer: Buffer,
}

struct ComputePass {
    pipeline: ComputePipeline,
    texture_layout: BindGroupLayout,
    uniform_layout: BindGroupLayout,
    compute_map_data_layout: BindGroupLayout,
}

struct FillPass {
    pipeline: RenderPipeline,
    texture_layout: BindGroupLayout,
}

struct GraphLayout {
    compute: ComputePass,
    fill: FillPass,
    start_time: Instant,
}

type CameraProp = (GlobalTransform, Mat4, u32);

#[derive(Default)]
struct Pipeline {
    size: UVec2,
    camera_prop: CameraProp,
    compute_texture_bg: Option<BindGroup>,
    compute_uniform_bg: Option<BindGroup>,
    compute_map_data_bg: Option<BindGroup>,
    render_texture_bg: Option<BindGroup>,
}

fn create_compute_pipeline(module: &ShaderModule, device: &Device) -> ComputePass {
    let bind_entry0 = BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::StorageTexture {
            access: StorageTextureAccess::ReadWrite,
            format: TEXTURE_FORMAT,
            view_dimension: TextureViewDimension::D2,
        },
        count: None,
    };
    let mut bind_entry1 = bind_entry0.clone();
    bind_entry1.binding = 1;
    let texture_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[bind_entry0, bind_entry1],
    });

    let uniform_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let compute_map_data_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&texture_layout, &uniform_layout, &compute_map_data_layout],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        module,
        entry_point: "compute",
    });
    ComputePass {
        pipeline,
        texture_layout,
        uniform_layout,
        compute_map_data_layout,
    }
}

fn create_render_pipeline(module: &ShaderModule, render_device: &Device) -> FillPass {
    let bind_group_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    multisampled: false,
                    // Should filterable be false if we want nearest-neighbor?
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let pipeline_layout = render_device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });
    let pipeline = render_device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module,
            entry_point: "compute_vertex",
            buffers: &vec![],
        },
        fragment: Some(FragmentState {
            module,
            entry_point: "compute_fragment",
            targets: &vec![ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: None,
                write_mask: ColorWrites::ALL,
            }],
        }),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
        multiview: None,
    });
    FillPass {
        pipeline,
        texture_layout: bind_group_layout,
    }
}

impl FromWorld for GraphLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap().wgpu_device();

        let module = include_spirv_raw!("../../assets/voxel_render/shader.spv");
        let module = unsafe { render_device.create_shader_module_spirv(&module) };
        // let module_spv= include_spirv_raw!("../assets/voxel_render/svt.spv");
        // let module_spv= unsafe { render_device.create_shader_module_spirv(&module_spv) };
        GraphLayout {
            compute: create_compute_pipeline(&module, &render_device),
            fill: create_render_pipeline(&module, &render_device),
            start_time: Instant::now(),
        }
    }
}

pub struct VoxelMapRenderPlugin;

impl Plugin for VoxelMapRenderPlugin {
    fn build(&self, app: &mut App) {
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

fn extract_phase(mut commands: Commands, map: Res<VoxelMapRenderData>, device: Res<RenderDevice>) {
    let device = device.wgpu_device();
    let buffer: &[u8] = unsafe { map.data.mem.as_slice().align_to::<u8>().1 };
    let buffer = device.create_buffer_init(&BufferInitDescriptor {
        contents: buffer,
        label: None,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    });
    commands.insert_resource(ExtractedMapRenderData { buffer });
}

fn create_compute_uniform_bg(
    device: &Device,
    layout: &GraphLayout,
    camera_prop: CameraProp,
    size: UVec2,
    frame_index: u32,
) -> BindGroup {
    let transform = camera_prop.0.compute_matrix();
    let inverse_projection = camera_prop.1.inverse();
    let mut top_left = inverse_projection * vec4(-1.0, 1.0, -1.0, 1.0);
    let bottom_right = inverse_projection * vec4(1.0, -1.0, -1.0, 1.0);
    let mut camera_h = (bottom_right - top_left) / (size.x as f32);
    camera_h.y = 0.0;
    let mut camera_v = (bottom_right - top_left) / (size.y as f32);
    camera_v.x = 0.0;
    top_left.w = 0.0;
    let uniform = VoxelMapRenderUniform {
        size,
        camera_pos: camera_prop.0.translation,
        camera_look: (transform * top_left).truncate(),
        camera_h: (transform * camera_h).truncate(),
        camera_v: (transform * camera_v).truncate(),
        time: layout.start_time.elapsed().as_micros() as f32 * 1e-6,
        frame_index,
    };
    let compute_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        contents: uniform.as_std140().as_bytes(),
    });

    device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &layout.compute.uniform_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: compute_uniform_buffer.as_entire_binding(),
        }],
    })
}

fn create_texture_bg(device: &Device, layout: &GraphLayout, size: UVec2) -> (BindGroup, BindGroup) {
    let tex_desc = TextureDescriptor {
        label: None,
        size: Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TEXTURE_FORMAT,
        usage: TextureUsages::COPY_DST
            | TextureUsages::STORAGE_BINDING
            | TextureUsages::TEXTURE_BINDING,
    };
    let tex_view_desc = TextureViewDescriptor {
        aspect: TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
        ..Default::default()
    };
    let texture0 = device.create_texture(&tex_desc);
    let texture1 = device.create_texture(&tex_desc);
    let texture_view0 = texture0.create_view(&tex_view_desc);
    let texture_view1 = texture1.create_view(&tex_view_desc);

    let compute_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &layout.compute.texture_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&texture_view0),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&texture_view1),
            },
        ],
    });
    let sampler = device.create_sampler(&SamplerDescriptor {
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        mipmap_filter: FilterMode::Nearest,
        ..Default::default()
    });

    let render_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &layout.fill.texture_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&texture_view0),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&sampler),
            },
        ],
    });
    (compute_bind_group, render_bind_group)
}

fn queue_phase(
    device: Res<RenderDevice>,
    pipeline_layout: Res<GraphLayout>,
    mut pipeline: ResMut<Pipeline>,
    extracted_data: Res<ExtractedMapRenderData>,
    view: Query<(&ExtractedView, &ViewTarget)>,
) {
    let device = device.wgpu_device();
    let (extracted_view, view_target) = view.iter().next().unwrap();
    let size = view_target.size;
    let transform = extracted_view.transform;
    let projection = extracted_view.projection;
    let camera_changd = pipeline.camera_prop.0 == transform && pipeline.camera_prop.1 == projection;
    let frame_index = ifelse!(camera_changd, pipeline.camera_prop.2 + 1, 0);
    //let frame_index = 0u32;
    let camera_prop = (transform, projection, frame_index);
    pipeline.camera_prop = camera_prop;
    let compute_uniform_bg: BindGroup =
        create_compute_uniform_bg(device, &pipeline_layout, camera_prop, size, frame_index);
    pipeline.compute_uniform_bg = Some(compute_uniform_bg);

    pipeline.compute_map_data_bg = Some(device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline_layout.compute.compute_map_data_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: extracted_data.buffer.as_entire_binding(),
        }],
    }));

    if size != pipeline.size {
        let (compute_bg, render_bg) = create_texture_bg(device, &pipeline_layout, size);
        pipeline.size = size;
        pipeline.compute_texture_bg = Some(compute_bg);
        pipeline.render_texture_bg = Some(render_bg);
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
        let pipeline = world.get_resource::<Pipeline>().unwrap();

        {
            let mut compute_pass = render_context
                .command_encoder
                .begin_compute_pass(&ComputePassDescriptor::default());
            compute_pass.set_pipeline(&graph_layout.compute.pipeline);
            compute_pass.set_bind_group(0, pipeline.compute_texture_bg.as_ref().unwrap(), &[]);
            compute_pass.set_bind_group(1, pipeline.compute_uniform_bg.as_ref().unwrap(), &[]);
            compute_pass.set_bind_group(2, pipeline.compute_map_data_bg.as_ref().unwrap(), &[]);
            compute_pass.dispatch(
                (pipeline.size.x + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE,
                (pipeline.size.y + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE,
                1,
            );
        }
        {
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
            render_pass.set_pipeline(&graph_layout.fill.pipeline);
            render_pass.set_bind_group(0, pipeline.render_texture_bg.as_ref().unwrap(), &[]);
            render_pass.draw(0..3, 0..2);
        }
        Ok(())
    }
}
