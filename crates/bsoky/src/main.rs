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
    DefaultPlugins,
};
use common::math::*;

use bevy_common::{create_debug_cube, MovementSettings};
use bevy_inspector_egui::WorldInspectorPlugin;
use debug_create_svt::debug_create_sdf;

pub mod debug_create_svt;
pub mod map_render;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
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
            width: 1080.0,
            height: 720.0,
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

        .add_plugin(bevy_common::camera::PlayerPlugin)
        .add_plugin(bevy_common::free_camera::PlayerPlugin)
        .insert_resource(MovementSettings {
            speed: 120.,
            ..Default::default()
        })
        .insert_resource(bevy_common::free_camera::CameraSetupParameter {
            // position: Vec3::new(215.0, 394.0, 27.0),
            // look_at: Vec3::new(215.0, 374.0, 100.0),
            position: Vec3::ONE * (MySvt::TOTAL_DIM as f32),
            look_at: Vec3::ONE * (MySvt::TOTAL_DIM as f32 / 0.5),
        })

        .add_plugin(map_render::VoxelMapRenderPlugin)
        .insert_resource(map_render::VoxelMapRenderData { data: debug_create_sdf() })

        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(create_debug_cube)
        .run();
}
