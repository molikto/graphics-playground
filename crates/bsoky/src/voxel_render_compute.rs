use bevy::{
    core_pipeline::draw_3d_graph::{self, input::VIEW_ENTITY},
    math::uvec2,
    prelude::*,
    render::{
        render_graph::{self, RenderGraph, SlotInfo, SlotType},
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{ExtractedView, ViewTarget},
        RenderApp, RenderStage,
    },
    utils::Instant,
};
use bevy_crevice::std140::{AsStd140, Std140};
use bsoky_shader::*;
use common::math::*;
use wgpu::{
    include_spirv_raw,
    util::{BufferInitDescriptor, DeviceExt},
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
    BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites,
    ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, Device, Extent3d,
    FilterMode, FragmentState, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor,
    PrimitiveState, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    SamplerBindingType, SamplerDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource,
    ShaderStages, StorageTextureAccess, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension,
    VertexState,
};

const WORKGROUP_SIZE: u32 = 8;

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba32Float;

pub struct EnvRenderPipelineLayout {
    compute_pipeline: ComputePipeline,
    compute_uniform_bg_layout: BindGroupLayout,
    compute_bg_layout: BindGroupLayout,
    render_pipeline: RenderPipeline,
    render_bg_layout: BindGroupLayout,
    start_time: Instant,
    svt: Buffer,
}

type CameraProp = (GlobalTransform, Mat4, u32);

#[derive(Default)]
pub struct EnvRenderPipeline {
    size: UVec2,
    camera_prop: CameraProp,
    compute_uniform_bg: Option<BindGroup>,
    compute_bg: Option<BindGroup>,
    render_bg: Option<BindGroup>,
}

fn create_compute_pipeline(
    module: &ShaderModule,
    render_device: &Device,
) -> (ComputePipeline, BindGroupLayout, BindGroupLayout) {
    let uniform_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
    let bind_entry2 = BindGroupLayoutEntry {
        binding: 2,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    let texture_bind_group_layout =
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[bind_entry0, bind_entry1, bind_entry2],
        });

    let pipeline_layout = render_device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&uniform_layout, &texture_bind_group_layout],
        push_constant_ranges: &[],
    });
    let pipeline = render_device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        module,
        entry_point: "compute",
    });

    (pipeline, texture_bind_group_layout, uniform_layout)
}

fn create_render_pipeline(
    module: &ShaderModule,
    render_device: &Device,
) -> (RenderPipeline, BindGroupLayout) {
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
    (pipeline, bind_group_layout)
}

impl FromWorld for EnvRenderPipelineLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap().wgpu_device();

        let module = include_spirv_raw!("../assets/env/shader.spv");
        let module = unsafe { render_device.create_shader_module_spirv(&module) };
        // let module_spv= include_spirv_raw!("../assets/env/svt.spv");
        // let module_spv= unsafe { render_device.create_shader_module_spirv(&module_spv) };

        let (compute_pipeline, compute_bg_layout, compute_uniform_bg_layout) =
            create_compute_pipeline(&module, &render_device);

        let (render_pipeline, render_bg_layout) = create_render_pipeline(&module, &render_device);

        let svt = super::create_svt::debug_create_sdf();

        let svt_contents: &[u8] = unsafe { svt.mem.as_slice().align_to::<u8>().1 };
        let svt_buffer = render_device.create_buffer_init(&BufferInitDescriptor {
            contents: svt_contents,
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        EnvRenderPipelineLayout {
            compute_pipeline,
            render_pipeline,
            compute_bg_layout,
            compute_uniform_bg_layout,
            render_bg_layout,
            start_time: Instant::now(),
            svt: svt_buffer,
        }
    }
}

pub struct EnvRenderPlugin;

impl Plugin for EnvRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<EnvRenderPipelineLayout>()
            .init_resource::<EnvRenderPipeline>()
            .add_system_to_stage(RenderStage::Extract, extract_phase)
            .add_system_to_stage(RenderStage::Queue, queue_phase);

        let render_node = EnvRenderNode::new(&mut render_app.world);
        let mut render_graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();
        let render_graph = render_graph.get_sub_graph_mut(draw_3d_graph::NAME).unwrap();

        render_graph.add_node("env_render", render_node);
        render_graph
            .add_node_edge("env_render", draw_3d_graph::node::MAIN_PASS)
            .unwrap();
        let input_node_id = render_graph.input_node().unwrap().id;
        render_graph
            .add_slot_edge(
                input_node_id,
                VIEW_ENTITY,
                "env_render",
                EnvRenderNode::IN_VIEW,
            )
            .unwrap();
    }
}

#[derive(Clone)]
struct EnvRenderObject {}

fn setup(mut commands: Commands) {
    commands.insert_resource(EnvRenderObject {})
}

fn extract_phase(mut commands: Commands, obj: Res<EnvRenderObject>) {
    commands.insert_resource(obj.clone());
}

fn create_compute_uniform_bg(
    device: &Device,
    layout: &EnvRenderPipelineLayout,
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
    let uniform = EnvShaderUniform {
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
        layout: &layout.compute_uniform_bg_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: compute_uniform_buffer.as_entire_binding(),
        }],
    })
}

fn create_texture_bg(device: &Device, layout: &EnvRenderPipelineLayout, size: UVec2) -> (BindGroup, BindGroup) {
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
        layout: &layout.compute_bg_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&texture_view0),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&texture_view1),
            },
            BindGroupEntry {
                binding: 2,
                resource: layout.svt.as_entire_binding(),
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
        layout: &layout.render_bg_layout,
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
    mut queue: ResMut<RenderQueue>,
    device: Res<RenderDevice>,
    pipeline_layout: Res<EnvRenderPipelineLayout>,
    mut pipeline: ResMut<EnvRenderPipeline>,
    view: Query<(&ExtractedView, &ViewTarget)>,
) {
    let device = device.wgpu_device();
    let (extracted_view, view_target) = view.iter().next().unwrap();
    let size = view_target.size;
    let transform = extracted_view.transform;
    let projection = extracted_view.projection;
    let frame_index = if pipeline.camera_prop.0 == transform && pipeline.camera_prop.1 == projection  { pipeline.camera_prop.2 + 1 } else { 0 };
    let camera_prop = (transform, projection, frame_index);
    pipeline.camera_prop = camera_prop;
    let compute_uniform_bg: BindGroup =
        create_compute_uniform_bg(device, &pipeline_layout, camera_prop, size, frame_index);
    pipeline.compute_uniform_bg = Some(compute_uniform_bg);
    
    if size != pipeline.size {
        let (compute_bg, render_bg) = create_texture_bg(device, &pipeline_layout, size);
        pipeline.size = size;
        pipeline.compute_bg = Some(compute_bg);
        pipeline.render_bg = Some(render_bg);
    };
}

struct EnvRenderNode {
    query: QueryState<(&'static ViewTarget, &'static ExtractedView)>,
}

impl EnvRenderNode {
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl render_graph::Node for EnvRenderNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(EnvRenderNode::IN_VIEW, SlotType::Entity)]
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
        let pipeline_layout = world.get_resource::<EnvRenderPipelineLayout>().unwrap();
        let pipeline = world.get_resource::<EnvRenderPipeline>().unwrap();

        {
            let mut compute_pass = render_context
                .command_encoder
                .begin_compute_pass(&ComputePassDescriptor::default());
            compute_pass.set_pipeline(&pipeline_layout.compute_pipeline);
            compute_pass.set_bind_group(0, pipeline.compute_uniform_bg.as_ref().unwrap(), &[]);
            compute_pass.set_bind_group(1, pipeline.compute_bg.as_ref().unwrap(), &[]);
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
            render_pass.set_pipeline(&pipeline_layout.render_pipeline);
            render_pass.set_bind_group(0, pipeline.render_bg.as_ref().unwrap(), &[]);
            render_pass.draw(0..3, 0..2);
        }
        Ok(())
    }
}
