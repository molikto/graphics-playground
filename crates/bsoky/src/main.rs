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

pub mod create_svt;
pub mod voxel_render_fragment;
pub mod voxel_render_compute;
use bevy_common::{create_debug_cube, MovementSettings};
use bevy_inspector_egui::WorldInspectorPlugin;
use bsoky_shader::MySvtMut;
use common::math::*;

fn main() {
    let total_size = MySvtMut::TOTAL_DIM as f32;
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
        .insert_resource(MovementSettings {
            speed: 120.,
            ..Default::default()
        })
        .insert_resource(bevy_common::camera::CameraSetupParameter {
            // position: Vec3::new(215.0, 394.0, 27.0),
            // look_at: Vec3::new(215.0, 374.0, 100.0),
            position: Vec3::new(27.0,394.0,  215.0),
            look_at: Vec3::new( 100.0,374.0,  215.0),
        })

        .add_plugin(voxel_render_compute::EnvRenderPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(create_debug_cube)
        .run();
}
