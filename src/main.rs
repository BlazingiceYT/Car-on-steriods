mod camera;
mod car;
mod hud;
mod world;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "BlazingIce – City Racer".into(),
                canvas: Some("#gameCanvas".into()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.706, 0.839, 0.980)))
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 300.0,
            ..default()
        })
        .insert_resource(car::NitroState::default())
        .insert_resource(car::NitroParticleTimer(Timer::from_seconds(
            0.03,
            TimerMode::Repeating,
        )))
        .add_systems(
            Startup,
            (world::setup_world, car::setup_car, camera::setup_camera),
        )
        .add_systems(
            Update,
            (
                car::movement_system,
                car::nitro_particle_spawn_system,
                car::nitro_particle_update_system,
                car::car_model_fallback_system,
                camera::camera_follow_system,
                hud::hud_bridge_system,
            )
                .chain(),
        )
        .run();
}
