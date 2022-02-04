use std::borrow::Cow;

use super::CameraProp;
use bevy::{
    core_pipeline::draw_3d_graph::{self, input::VIEW_ENTITY},
    ecs::schedule::IntoSystemDescriptor,
    prelude::*,
    render::{
        render_graph::{self, RenderGraph, SlotInfo, SlotType},
        render_resource::{
            BindGroup, BindGroupLayout, CachedPipelineId, FragmentState, RenderPipelineCache,
            RenderPipelineDescriptor, VertexState,
        },
        renderer::{RenderContext, RenderDevice},
        texture::BevyDefault,
        view::{ExtractedView, ViewTarget},
        RenderApp, RenderStage,
    },
    utils::Instant
};
use bevy_crevice::std140::{AsStd140, Std140};
use common::math::*;
use common::*;
use wgpu::{
    util::BufferInitDescriptor, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, BufferUsages, ColorTargetState,
    ColorWrites, LoadOp, MultisampleState, Operations, PrimitiveState, RenderPassDescriptor,
    ShaderStages, TextureFormat,
};

pub struct PipelineLayout {
    pipeline: CachedPipelineId,
    texture_layout: BindGroupLayout,
    uniform_layout: BindGroupLayout,
    pub scene_layout: BindGroupLayout,
}

pub struct GraphLayout {
    pub pipeline: PipelineLayout,
    start_time: Instant,
}

fn create_render_uniform_bg(
    device: &RenderDevice,
    layout: &GraphLayout,
    camera_prop: CameraProp,
    size: UVec2,
    frame_index: u32,
) -> BindGroup {
    let uniform =
        camera_prop.get_ray_tracing_uniform(size, layout.start_time.elapsed().as_secs_f32(), 0);
    let compute_uniform_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
        label: None,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        contents: uniform.as_std140().as_bytes(),
    });
    device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &layout.pipeline.uniform_layout,
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
        layout: &layout.pipeline.texture_layout,
        entries: &[BindGroupEntry {
            binding: 1,
            resource: uniform_buffer.as_entire_binding(),
        }],
    })
}

pub fn create_pipeline_layout(
    world: &mut World,
    scene_layout: BindGroupLayoutDescriptor,
    vert: Handle<Shader>,
    frag: Handle<Shader>,
) -> GraphLayout {
    let device = world.get_resource::<RenderDevice>().unwrap();
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

    let scene_layout = device.create_bind_group_layout(&scene_layout);
    let desc = RenderPipelineDescriptor {
        label: None,
        layout: Some(vec![
            texture_layout.clone(),
            uniform_layout.clone(),
            scene_layout.clone(),
        ]),
        vertex: VertexState {
            shader: vert,
            shader_defs: vec![],
            entry_point: Cow::Borrowed("main"),
            buffers: vec![],
        },
        fragment: Some(FragmentState {
            shader: frag,
            entry_point: Cow::Borrowed("main"),
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
    let mut pipeline_cache = world.get_resource_mut::<RenderPipelineCache>().unwrap();
    let pipeline = pipeline_cache.queue(desc);
    let pipeline = PipelineLayout {
        pipeline,
        texture_layout,
        uniform_layout,
        scene_layout,
    };
    GraphLayout {
        pipeline,
        start_time: Instant::now(),
    }
}

#[derive(Default)]
pub struct Pipeline {
    size: UVec2,
    camera_prop: CameraProp,
    texture_bg: Option<BindGroup>,
    uniform_bg: Option<BindGroup>,
    scene_bg: Option<BindGroup>,
}

pub fn queue_pipeline(
    device: Res<RenderDevice>,
    pipeline_layout: Res<GraphLayout>,
    mut pipeline: ResMut<Pipeline>,
    view: Query<(&ExtractedView, &ViewTarget)>,
    scene_bg: BindGroup,
) {
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
    pipeline.uniform_bg = Some(compute_uniform_bg);

    pipeline.scene_bg = Some(scene_bg);
    if size != pipeline.size {
        let texture_bg = create_texture_bg(&device, &pipeline_layout, size);
        pipeline.size = size;
        pipeline.texture_bg = Some(texture_bg);
    };
}

pub fn setup_plugin<Params1, Params2>(
    app: &mut App,
    scene_layout: BindGroupLayoutDescriptor,
    vert: Handle<Shader>,
    frag: Handle<Shader>,
    extract_phase: impl IntoSystemDescriptor<Params1>,
    queue_phase: impl IntoSystemDescriptor<Params2>,
    name: &'static str,
) {
    let render_app = app.sub_app_mut(RenderApp);
    let world = &mut render_app.world;
    let graph_layout = create_pipeline_layout(world, scene_layout, vert, frag);
    render_app
        .insert_resource(graph_layout)
        .init_resource::<Pipeline>()
        .add_system_to_stage(RenderStage::Extract, extract_phase)
        .add_system_to_stage(RenderStage::Queue, queue_phase);

    let render_node = RenderNode::new(&mut render_app.world);
    let mut render_graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();
    let render_graph = render_graph.get_sub_graph_mut(draw_3d_graph::NAME).unwrap();

    render_graph.add_node(name, render_node);
    render_graph
        .add_node_edge(name, draw_3d_graph::node::MAIN_PASS)
        .unwrap();
    let input_node_id = render_graph.input_node().unwrap().id;
    render_graph
        .add_slot_edge(input_node_id, VIEW_ENTITY, name, RenderNode::IN_VIEW)
        .unwrap();
}

struct RenderNode {
    query: QueryState<(&'static ViewTarget, &'static ExtractedView)>,
}

impl RenderNode {
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl render_graph::Node for RenderNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(RenderNode::IN_VIEW, SlotType::Entity)]
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
            unwrap_or_return_val!(pipeline_cache.get(graph_layout.pipeline.pipeline), Ok(()));
        let pipeline = world.get_resource::<Pipeline>().unwrap();
        let render_uniform_bg = unwrap_or_return_val!(pipeline.uniform_bg.as_ref(), Ok(()));
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
        render_pass.set_bind_group(0, pipeline.texture_bg.as_ref().unwrap(), &[]);
        render_pass.set_bind_group(1, render_uniform_bg, &[]);
        render_pass.set_bind_group(2, pipeline.scene_bg.as_ref().unwrap(), &[]);
        render_pass.draw(0..3, 0..2);
        Ok(())
    }
}

/**
 *
 *
 * the simple setup
 */

fn extract_phase_noop() {}

fn queue_phase_noop(
    device: Res<RenderDevice>,
    pipeline_layout: Res<GraphLayout>,
    mut pipeline: ResMut<Pipeline>,
    view: Query<(&ExtractedView, &ViewTarget)>,
) {
    let bg = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline_layout.pipeline.scene_layout,
        entries: &[],
    });
    queue_pipeline(device, pipeline_layout, pipeline, view, bg);
}

pub fn setup_plugin_simple(
    app: &mut App,
    vert: Handle<Shader>,
    frag: Handle<Shader>,
    name: &'static str,
) {
    setup_plugin(
        app,
        BindGroupLayoutDescriptor {
            label: None,
            entries: &[],
        },
        vert,
        frag,
        extract_phase_noop,
        queue_phase_noop,
        name,
    );
}
