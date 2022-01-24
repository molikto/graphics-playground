use std::time::Duration;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::Vec2,
    prelude::{App, Msaa},
    render::{
        options::{Backends, WgpuOptions},
        render_resource::{WgpuFeatures, WgpuLimits},
    },
    window::WindowDescriptor,
    DefaultPlugins, pbr::wireframe::WireframePlugin,
};
use bsoky_common_cpu::MySvt;
use common::math::*;

pub mod debug_create_svt;
pub mod map_render;
pub mod build;


fn main() {
    build::main();
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(WgpuOptions {
            features: WgpuFeatures::PUSH_CONSTANTS,
            backends: Some(Backends::VULKAN),
            limits: WgpuLimits {
                max_push_constant_size: 256,
                max_storage_buffer_binding_size: 4000000000,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_resource(WindowDescriptor {
            width: 1080.0 / 2.0,
            height: 720.0 / 2.0,
            title: "codename: bsoky".into(),
            position: Some(Vec2::new(0.0, 24.0)),
            ..Default::default()
        })

        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin {
            wait_duration: Duration::from_secs(5),
            filter: Some(vec![FrameTimeDiagnosticsPlugin::FRAME_TIME]),
            ..Default::default()
        })
        
        .add_plugin(bevy_editor_pls::prelude::EditorPlugin)
        .add_startup_system(bevy_common::create_debug_cube)
        .add_startup_system(bevy_common::watch_for_changes)


        .insert_resource(map_render::VoxelMapRenderData { data: crate::debug_create_svt::debug_create_sdf() })
        .add_plugin(bevy_common::free_camera::PlayerPlugin)
        .insert_resource(bevy_common::MovementSettings {
            speed: 120.,
            ..Default::default()
        })
        .insert_resource(bevy_common::free_camera::CameraSetupParameter {
            // position: Vec3::new(215.0, 394.0, 27.0),
            // look_at: Vec3::new(215.0, 374.0, 100.0),
            position: vec3(0.5, 1.0, 0.5) * MySvt::TOTAL_DIM as f32,
            look_at: Vec3::ONE * (MySvt::TOTAL_DIM as f32),
        })

        .add_plugin(map_render::VoxelMapRenderPlugin)
        .run();
}
