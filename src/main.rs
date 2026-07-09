//! BlazingIce – City Racer, Bevy/WASM port.
//!
//! Ported 1:1 from the original Three.js physics: acceleration, top speed,
//! braking, friction/drag, steering ramp + return-to-center, drift-adjusted
//! turning, jump/bump vertical physics, and the (rigid, non-lerped) chase
//! camera. Loads car.glb with an automatic box-car fallback.
//!
//! Still to come (tell Claude which one you want next):
//! AI traffic cars, nitro particle trail, tyre marks, on-screen HUD
//! (speed/FPS/drift/air readouts), and your real map/track.

mod camera;
mod car;
mod world;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "BlazingIce – City Racer".into(),
                canvas: Some("#gameCanvas".into()), // binds to the canvas in index.html
                fit_canvas_to_parent: true,         // replaces the manual JS resize listener
                prevent_default_event_handling: true, // stops WASD/arrows from scrolling the page
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.706, 0.839, 0.980))) // JS sky color 0xb4d6fa
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 300.0,
            ..default()
        })
        .add_systems(
            Startup,
            (world::setup_world, car::setup_car, camera::setup_camera),
        )
        .add_systems(
            Update,
            (
                car::movement_system,
                car::car_model_fallback_system,
                camera::camera_follow_system,
            )
                .chain(),
        )
        .run();
}
