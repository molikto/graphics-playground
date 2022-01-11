use std::{
    time::Duration
};

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{Vec2},
    pbr::{MaterialMeshBundle, MaterialPlugin},
    prelude::{App, Assets, Commands, ResMut, Msaa},
    render::{
        mesh::{Mesh}, options::{WgpuOptions, Backends}, render_resource::WgpuLimits,
    },
    window::WindowDescriptor,
    DefaultPlugins,
};

pub mod env_render;
pub mod create_svt;
use bevy_common::{RevertBox, create_debug_cube, MovementSettings};
use bsoky_no_std::MySvtMut;
use common::math::*;

use env_render::CustomMaterial;


fn create_simple_debug_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    let svt = create_svt::debug_create_rsvo();
    println!("total dim {}\nblock count {}\nmemory used {}\nmemory ratio {}", MySvtMut::total_dim(), svt.block_count(), svt.memory_used(), svt.memory_ratio());
    let total_size = MySvtMut::total_dim() as f32;
    let mesh = meshes.add(RevertBox::zero_with_size(Vec3::splat(total_size)).into());
    let material = materials.add(CustomMaterial { svt });
    commands.spawn_bundle(MaterialMeshBundle::<CustomMaterial> {
        mesh,
        material,
        ..Default::default()
    });
}

fn main() {
    //simulation_benchmark();
    let half_size = (MySvtMut::total_dim() / 2) as f32;
    App::new()
        .insert_resource(Msaa {  samples: 1 })
        .insert_resource(WgpuOptions {
            backends: Some(Backends::VULKAN),
            limits: WgpuLimits {
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
        .add_plugins(DefaultPlugins)
        .insert_resource(MovementSettings { 
            speed: 120.,
            ..Default::default()
        })
        .insert_resource(bevy_common::camera::CameraSetupParameter {
            position: Vec3::new(1.0, 1.0, 0.0) * half_size,
            look_at: Vec3::splat(1.0) * half_size,
        })
        .add_plugin(bevy_common::camera::PlayerPlugin)
        .add_plugin(LogDiagnosticsPlugin {
            wait_duration: Duration::from_secs(5),
            filter: Some(vec![FrameTimeDiagnosticsPlugin::FRAME_TIME]),
            ..Default::default()
        })
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(MaterialPlugin::<CustomMaterial>::default())
        .add_startup_system(create_debug_cube)
        .add_startup_system(create_simple_debug_objects)
        .run();
}
