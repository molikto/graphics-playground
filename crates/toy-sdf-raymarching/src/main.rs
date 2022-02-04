use std::time::Duration;

use bevy::{
    asset::AssetServerSettings,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::{App, Msaa, Plugin},
    render::options::{Backends, WgpuOptions},
    DefaultPlugins,
};
use bevy_common::{full_screen_render, load_shader};
use common::{math::*, cargo_manifest_dir};
use rust_gpu_builder::{RustGpuBuild};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        let vert = load_shader(app, "vert.spv");
        let frag = load_shader(app, "frag.spv");
        full_screen_render::setup_plugin_simple(app, vert, frag, "toy");
    }
}

fn main() {
    RustGpuBuild {
        source: cargo_manifest_dir!().join("shader"),
        assets: cargo_manifest_dir!().join("assets"),
        cross_to_glsl: true,
    }
    .spawn();
    App::new()
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(WgpuOptions {
            backends: Some(Backends::VULKAN),
            ..Default::default()
        })
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin {
            wait_duration: Duration::from_secs(5),
            filter: Some(vec![FrameTimeDiagnosticsPlugin::FRAME_TIME]),
            ..Default::default()
        })
        .add_plugin(bevy_common::free_camera::PlayerPlugin)
        .insert_resource(bevy_common::MovementSettings {
            speed: 120.,
            ..Default::default()
        })
        .insert_resource(bevy_common::free_camera::CameraSetupParameter {
            position: Vec3::new(4.5, 1.3 + 2.0, 4.5),
            look_at: Vec3::ZERO,
        })
        .add_plugin(RenderPlugin)
        .run();
}
