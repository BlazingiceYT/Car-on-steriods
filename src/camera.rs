use bevy::prelude::*;

use crate::car::{Car, SPAWN_X, SPAWN_Z};

const CAMERA_DISTANCE: f32 = 13.0; // JS: camDist
const CAMERA_HEIGHT_BASE: f32 = 5.0; // JS: the fixed "5.0" in camH
const CAMERA_LOOKAHEAD: f32 = 9.0; // JS: the `+ 9` in camera.lookAt(...)

#[derive(Component)]
pub struct FollowCamera;

pub fn setup_camera(mut commands: Commands) {
    // Starting pose matches the follow formula below exactly, so there's no
    // snap on the very first frame.
    commands.spawn((
        Camera3d::default(),
        Msaa::Off, // MSAA is a common lag source on WebGL2 — off by default; it's a per-camera component now
        FollowCamera,
        Transform::from_xyz(SPAWN_X, CAMERA_HEIGHT_BASE, SPAWN_Z - CAMERA_DISTANCE)
            .looking_at(Vec3::new(SPAWN_X, 1.2, SPAWN_Z + CAMERA_LOOKAHEAD), Vec3::Y),
    ));
}

/// Faithful 1:1 port: the original camera is a rigid, instant snap to this
/// offset every frame — it does not lerp/smooth, despite the brief
/// describing a "smooth" follow. See the note at the bottom for a drop-in
/// smoothed variant if you'd like that instead.
pub fn camera_follow_system(
    car_query: Query<(&Car, &Transform), Without<FollowCamera>>,
    mut camera_query: Query<&mut Transform, With<FollowCamera>>,
) {
    let Ok((car, car_transform)) = car_query.single() else { return; };
    let Ok(mut camera_transform) = camera_query.single_mut() else { return; };

    let car_pos = car_transform.translation;

    // JS: camH = 5.0 + Math.min(carY * 0.5, 4); camera.position.y = camH + carY;
    // This term is inert while the ground is flat (carY stays 0) but comes
    // alive the moment you wire up real bump/jump terrain in world.rs.
    let camera_height = CAMERA_HEIGHT_BASE + (car_pos.y * 0.5).min(4.0) + car_pos.y;

    camera_transform.translation = Vec3::new(
        car_pos.x - car.heading.sin() * CAMERA_DISTANCE,
        camera_height,
        car_pos.z - car.heading.cos() * CAMERA_DISTANCE,
    );

    let look_target = Vec3::new(
        car_pos.x + car.heading.sin() * CAMERA_LOOKAHEAD,
        car_pos.y + 1.2,
        car_pos.z + car.heading.cos() * CAMERA_LOOKAHEAD,
    );
    *camera_transform = camera_transform.looking_at(look_target, Vec3::Y);

    // ── Drop-in smoothed variant ──
    // Add `time: Res<Time>` to this system's parameters, then replace the
    // `camera_transform.translation = ...` block above with:
    //
    // let target = Vec3::new(
    //     car_pos.x - car.heading.sin() * CAMERA_DISTANCE,
    //     camera_height,
    //     car_pos.z - car.heading.cos() * CAMERA_DISTANCE,
    // );
    // let t = (10.0 * time.delta_secs()).min(1.0);
    // camera_transform.translation = camera_transform.translation.lerp(target, t);
}
