use bevy::app::ManualEventReader;
use bevy::input::*;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use common::math::*;
use dolly::prelude::*;

use super::util::*;

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
    pitch: f32,
    yaw: f32,
}

/// Mouse sensitivity and movement speed
pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 12.,
        }
    }
}

/// Used in queries when you want flycams and not other cameras
#[derive(Component)]
pub struct MainCamera;

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    window.set_cursor_lock_mode(!window.cursor_locked());
    window.set_cursor_visibility(!window.cursor_visible());
}

/// Grabs the cursor when game first starts
fn initial_grab_cursor(mut windows: ResMut<Windows>) {
    toggle_grab_cursor(windows.get_primary_mut().unwrap());
}

pub struct CameraSetupParameter{
    pub position: Vec3,
}

/// Spawns the `Camera3dBundle` to be controlled
fn setup_player(mut commands: Commands, setup: Res<CameraSetupParameter>) {
    let mut camera_pos = setup.position;
    //.looking_at(Vec3::new(0.0, 0.0, center_pos.z / 2.0), Vec3::Y)
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_translation(camera_pos),
            ..Default::default()
        })
        .insert(MainCamera)
        .insert(Name::new("MainCamera"));
    commands.spawn()
        .insert(CameraRigComponent(
            CameraRig::builder()
                .with(Position::new(camera_pos))
                .with(YawPitch::new())
                .with(Smooth::new_position_rotation(1.0, 1.0))
                .build(),
        ));
}

#[allow(clippy::type_complexity)]
fn update_camera_system(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    windows: Res<Windows>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: QuerySet<(
        QueryState<&mut Transform, With<MainCamera>>,
        QueryState<&mut CameraRigComponent>,
    )>,
) {
    let time_delta_seconds: f32 = time.delta_seconds();
    let boost_mult = 5.0f32;
    let sensitivity = Vec2::splat(1.0);

    let mut move_vec = Vec3::ZERO;

    // Q: Is dolly left-handed so z is flipped?
    if keys.pressed(KeyCode::W) {
        move_vec.z -= 1.0;
    }
    if keys.pressed(KeyCode::S) {
        move_vec.z += 1.0;
    }
    if keys.pressed(KeyCode::A) {
        move_vec.x -= 1.0;
    }
    if keys.pressed(KeyCode::D) {
        move_vec.x += 1.0;
    }

    if keys.pressed(KeyCode::E) || keys.pressed(KeyCode::Space) {
        move_vec.y += 1.0;
    }
    if keys.pressed(KeyCode::Q) {
        move_vec.y -= 1.0;
    }

    let boost: f32 = if keys.pressed(KeyCode::LShift) {
        1.
    } else {
        0.
    };

    let mut delta = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        delta += event.delta;
    }

    let mut q1 = query.q1();
    let mut rig = q1.single_mut();

    let move_vec =
        rig.0.final_transform.rotation * move_vec.clamp_length_max(1.0) * boost_mult.powf(boost);

    let window = windows.get_primary().unwrap();
    if window.cursor_locked() {
        rig.0.driver_mut::<YawPitch>().rotate_yaw_pitch(
            -0.1 * delta.x * sensitivity.x,
            -0.1 * delta.y * sensitivity.y,
        );
        rig.0.driver_mut::<Position>()
            .translate(move_vec * time_delta_seconds * 10.0);
    }

    let transform = rig.0.update(time_delta_seconds);

    let mut q0 = query.q0();
    let mut cam = q0.single_mut();

    cam.transform_2_bevy(transform);
}


fn cursor_grab(keys: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    if keys.just_pressed(KeyCode::Escape) {
        toggle_grab_cursor(window);
    }
}

/// Contains everything needed to add first-person fly camera behavior to your game
pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            .add_startup_system(setup_player)
            //.add_startup_system(initial_grab_cursor)
            .add_system(update_camera_system)
            .add_system(cursor_grab);
    }
}
