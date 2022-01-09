use std::{
    default, fs,
    path::{Path, PathBuf},
    time::Duration, sync::Arc,
};

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::system::Command,
    input::system::exit_on_esc_system,
    math::{UVec3, Vec2},
    pbr::{PbrBundle, StandardMaterial, MaterialMeshBundle, MaterialPlugin},
    prelude::{App, Assets, Commands, ResMut, Transform, Msaa},
    render::{
        color::Color,
        mesh::{shape, Mesh}, options::{WgpuOptions, Backends}, render_resource::WgpuLimits,
    },
    utils::Instant,
    window::WindowDescriptor,
    DefaultPlugins,
};

pub mod env_render;
use bevy_common::{RevertBox, create_debug_cube, MovementSettings};
use bsoky_no_std::{BLOCK_DIM, LEVEL_COUNT, MySvoMut};
use common::math::svo::*;
use common::math::*;

use env_render::CustomMaterial;
use rand::{thread_rng, Rng};
use sdfu::{SDF};



fn debug_create_rsvo() -> MySvoMut {
    // download yourself here https://github.com/ephtracy/voxel-model/blob/master/svo/
    let rsvo = std::fs::read( Path::new(env!("CARGO_MANIFEST_DIR")).join("sibenik_8k.rsvo")).unwrap();
    let svo: MySvoMut = load_svo_from_rsvo(&rsvo);
    svo
}

fn debug_create1() -> MySvoMut {
    let mut svo = MySvoMut::init(0);
    svo.set(Usvo3::new(3, 3, 3), 1);
    //println!("{:?}", svo.debug_items());
    //println!("{:?}", mem[0..10].to_vec());
    svo
}
fn debug_create_sdf() -> MySvoMut {
    // 4,4 = 0.21
    // 2,8 = 0.11
    let mut svo = MySvoMut::init(0);
    let sdf = sdfu::Sphere::new(0.45)
        .subtract(sdfu::Box::new(Vec3A::new(0.25, 0.25, 1.5)))
        .union_smooth(
            sdfu::Sphere::new(0.3).translate(Vec3A::new(0.3, 0.3, 0.0)),
            0.1,
        )
        .union_smooth(
            sdfu::Sphere::new(0.3).translate(Vec3A::new(-0.3, 0.3, 0.0)),
            0.1,
        )
        .subtract(
            sdfu::Box::new(Vec3A::new(0.125, 0.125, 1.5)).translate(Vec3A::new(-0.3, 0.3, 0.0)),
        )
        .subtract(
            sdfu::Box::new(Vec3A::new(0.125, 0.125, 1.5)).translate(Vec3A::new(0.3, 0.3, 0.0)),
        )
        .subtract(sdfu::Box::new(Vec3A::new(1.5, 0.1, 0.1)).translate(Vec3A::new(0.0, 0.3, 0.0)))
        .subtract(sdfu::Box::new(Vec3A::new(0.2, 2.0, 0.2)))
        .scale(0.5)
        .translate(Vec3A::new(0.5, 0.5, 0.5));
    let total_size = MySvoMut::total_dim() as f32;
    for level in 0..LEVEL_COUNT as usvo {
        let level_cap = level + 1;
        let level_dim = BLOCK_DIM.pow(level_cap as u32);
        let level_size = BLOCK_DIM.pow(LEVEL_COUNT as u32 - (level_cap as u32));
        for x in 0..level_dim {
            for y in 0..level_dim {
                for z in 0..level_dim {
                    let mut count = 0;
                    for i in 0..level_size {
                        for j in 0..level_size {
                            for k in 0..level_size {
                                let v = Vec3A::new(
                                    (x * level_size + i) as f32,
                                    (y * level_size + j) as f32,
                                    (z * level_size + k) as f32,
                                ) / total_size;
                                if sdf.dist(v) < 0.0 {
                                    count += 1;
                                }
                            }
                        }
                    }
                    let material = if count > level_size * level_size * level_size / 2 {
                        thread_rng().gen_range(1..4)
                    } else {
                        0
                    };
                    svo.set_with_level_cap(
                        level_cap,
                        Usvo3::new(x * level_size, y * level_size, z * level_size),
                        material,
                    );
                }
            }
        }
    }
    svo
}

fn create_simple_debug_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    let svo = debug_create_rsvo();
    println!("total dim {}\nblock count {}\nmemory used {}\nmemory ratio {}", MySvoMut::total_dim(), svo.block_count(), svo.memory_used(), svo.memory_ratio());
    let total_size = MySvoMut::total_dim() as f32;
    let mesh = meshes.add(RevertBox::zero_with_size(Vec3::splat(total_size)).into());
    let material = materials.add(CustomMaterial { svo });
    commands.spawn_bundle(MaterialMeshBundle::<CustomMaterial> {
        mesh,
        material,
        ..Default::default()
    });
}

fn main() {
    //simulation_benchmark();
    let half_size = (MySvoMut::total_dim() / 2) as f32;
    App::new()
        .insert_resource(Msaa {  samples: 1 })
        .insert_resource(WgpuOptions {
            backends: Backends::VULKAN,
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
