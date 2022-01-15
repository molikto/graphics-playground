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
use bsoky_no_std::MySvtMut;
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
            // width: 1920.0,
            // height: 1080.0,
            width: 1080.0,
            height: 720.0,
            title: "codename: bsoky".into(),
            position: Some(Vec2::new(0.0, 24.0)),
            ..Default::default()
        })
        .add_plugin(bevy_common::camera::PlayerPlugin)

        .add_plugin(LogDiagnosticsPlugin {
            wait_duration: Duration::from_secs(5),
            filter: Some(vec![FrameTimeDiagnosticsPlugin::FRAME_TIME]),
            ..Default::default()
        })

        .insert_resource(MovementSettings {
            speed: 120.,
            ..Default::default()
        })
        .insert_resource(bevy_common::camera::CameraSetupParameter {
            position: Vec3::new(total_size / 2.0, total_size / 2.0, 0.0),
            look_at: Vec3::splat(total_size / 2.0),
        })
        .add_plugin(FrameTimeDiagnosticsPlugin::default())

        .add_plugin(voxel_render_compute::EnvRenderPlugin)

        .add_startup_system(create_debug_cube)
        .run();
}
