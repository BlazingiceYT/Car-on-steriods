use bevy::asset::LoadState;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use bevy::world_serialization::{WorldAsset, WorldAssetRoot};

// ─── Physics constants — copied 1:1 from the original JS ───
const BASE_MAX_SPEED: f32 = 50.0; // JS: BASE_MAX_SPEED
const NITRO_MAX_SPEED: f32 = 62.5; // JS: NITRO_MAX_SPEED
const ACCEL: f32 = 26.0; // JS: ACCEL
const NITRO_ACCEL: f32 = 48.0; // JS: NITRO_ACCEL
const BRAKE: f32 = 42.0; // JS: BRAKE
const FRICTION: f32 = 14.0; // JS: FRIC
const STEER_SPEED: f32 = 2.5; // JS: STEER_SPD (rad/s ramp rate)
const MAX_STEER: f32 = 0.46; // JS: MAX_STEER (radians, ~26°)
const TURN_BASE: f32 = 0.025; // JS: TURN_BASE (steer angle -> heading change)
const GRAVITY: f32 = 20.0; // JS: GRAVITY

pub const SPAWN_X: f32 = 0.0; // JS: carX = 0
pub const SPAWN_Z: f32 = 15.0; // JS: carZ = 15

// Tweak these three if car.glb looks wrong once it loads — same tunable
// constants your original JS exposed for exactly this purpose.
const CAR_MODEL_SCALE: f32 = 1.0;
const CAR_MODEL_YAW_OFFSET: f32 = std::f32::consts::FRAC_PI_2;
const CAR_MODEL_Y_OFFSET: f32 = 0.0;

#[derive(Component, Debug, Default)]
pub struct Car {
    pub speed: f32,             // JS: speed
    pub heading: f32,           // JS: carAngle (radians around Y)
    pub steer_angle: f32,       // JS: steerAngle
    pub drift_angle: f32,       // JS: driftAngle
    pub vertical_velocity: f32, // JS: carVY
    pub in_air: bool,           // JS: inAir
    pub air_time: f32,          // JS: airTime
    pub pitch: f32,             // JS: carGroup.rotation.x
}

#[derive(Component)]
pub struct GlbCarModel;

#[derive(Component)]
pub struct FallbackCarModel;

/// Spawns the car entity with two children: your car.glb (shown by default)
/// and a low-poly box car (hidden by default, shown automatically if the
/// glb fails to load). Mirrors the original's GLTFLoader + fallback logic.
pub fn setup_car(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let glb_scene: Handle<WorldAsset> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("car.glb"));

    commands
        .spawn((
            Car::default(),
            Transform::from_xyz(SPAWN_X, 0.0, SPAWN_Z),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                GlbCarModel,
                WorldAssetRoot(glb_scene),
                Transform::from_xyz(0.0, CAR_MODEL_Y_OFFSET, 0.0)
                    .with_scale(Vec3::splat(CAR_MODEL_SCALE))
                    .with_rotation(Quat::from_rotation_y(CAR_MODEL_YAW_OFFSET)),
            ));

            parent.spawn((
                FallbackCarModel,
                Visibility::Hidden,
                Mesh3d(meshes.add(Cuboid::new(2.0, 0.62, 4.2))), // JS body dims
                MeshMaterial3d(materials.add(Color::srgb(0.90, 0.24, 0.24))), // JS 0xe53e3e
                Transform::from_xyz(0.0, 0.5, 0.0),
            ));
            parent.spawn((
                FallbackCarModel,
                Visibility::Hidden,
                Mesh3d(meshes.add(Cuboid::new(1.38, 0.5, 2.0))), // JS cabin dims
                MeshMaterial3d(materials.add(Color::srgb(0.10, 0.13, 0.17))), // JS 0x1a202c
                Transform::from_xyz(0.0, 1.01, -0.2),
            ));
        });
}

/// If car.glb fails to load (missing file, wrong path/name, bad export),
/// swap to the box car instead of leaving the player invisible. Mirrors the
/// original's GLTFLoader error-callback fallback.
pub fn car_model_fallback_system(
    asset_server: Res<AssetServer>,
    mut glb_query: Query<(&WorldAssetRoot, &mut Visibility), (With<GlbCarModel>, Without<FallbackCarModel>)>,
    mut fallback_query: Query<&mut Visibility, (With<FallbackCarModel>, Without<GlbCarModel>)>,
) {
    let Ok((scene_root, mut glb_visibility)) = glb_query.single_mut() else { return; };

    if let Some(LoadState::Failed(_)) = asset_server.get_load_state(&scene_root.0) {
        *glb_visibility = Visibility::Hidden;
        for mut fallback_visibility in fallback_query.iter_mut() {
            *fallback_visibility = Visibility::Visible;
        }
    }
}

pub fn movement_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Car, &mut Transform)>,
) {
    let Ok((mut car, mut transform)) = query.single_mut() else { return; };

    // Same 33ms clamp the original clock.getDelta() used, so a stalled tab
    // doesn't fling the car across the map on the next frame.
    let dt = time.delta_secs().min(0.0333);

    let gas = keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp);
    let brake = keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown);
    let left = keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft);
    let right = keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight);
    let nitro = keys.pressed(KeyCode::KeyN);

    let max_speed = if nitro { NITRO_MAX_SPEED } else { BASE_MAX_SPEED };
    let accel = if nitro { NITRO_ACCEL } else { ACCEL };

    // Speed — accelerate, brake/reverse, or coast down under friction.
    if gas {
        car.speed = (car.speed + accel * dt).min(max_speed);
    } else if brake {
        car.speed = (car.speed - BRAKE * dt).max(-BASE_MAX_SPEED * 0.28);
    } else {
        if car.speed > 0.0 {
            car.speed = (car.speed - FRICTION * dt).max(0.0);
        }
        if car.speed < 0.0 {
            car.speed = (car.speed + FRICTION * dt).min(0.0);
        }
    }

    // Steering — ramps toward MAX_STEER while held, springs back to center
    // when released.
    if left {
        car.steer_angle = (car.steer_angle + STEER_SPEED * dt).min(MAX_STEER);
    } else if right {
        car.steer_angle = (car.steer_angle - STEER_SPEED * dt).max(-MAX_STEER);
    } else {
        car.steer_angle *= 1.0 - 11.0 * dt;
    }

    // Drift detection — identical thresholds to the original.
    let is_drifting =
        car.speed.abs() > 22.0 && car.steer_angle.abs() > 0.16 && (brake || left || right);

    if car.speed.abs() > 0.5 {
        let direction = if car.speed > 0.0 { 1.0 } else { -1.0 };
        let drift_multiplier = if is_drifting { 1.65 } else { 1.0 };
        car.heading += car.steer_angle * direction * TURN_BASE * drift_multiplier;
    }

    if is_drifting {
        let target_drift = car.steer_angle * 1.15;
        car.drift_angle += (target_drift - car.drift_angle) * 9.0 * dt;
    } else {
        car.drift_angle += (0.0 - car.drift_angle) * 6.0 * dt;
    }

    // Horizontal movement. Note the asymmetry carried over from the
    // original: the car SLIDES along (heading - drift_angle * 0.32) but
    // visually POINTS along (heading + drift_angle) — that gap is what
    // sells the drift, and it's easy to accidentally "fix" away.
    let move_angle = car.heading - car.drift_angle * 0.32;
    transform.translation.x += move_angle.sin() * car.speed * dt;
    transform.translation.z += move_angle.cos() * car.speed * dt;

    // ── Vertical / jump physics over terrain bumps ──
    // Inert on today's flat placeholder ground (ground_y is always 0), but
    // fully wired up for the moment you plug in real bumpy terrain.
    let ground_y =
        crate::world::terrain_height_at(transform.translation.x, transform.translation.z);

    if !car.in_air {
        if ground_y > transform.translation.y + 0.35 && car.speed.abs() > 8.0 {
            let steepness = ground_y - transform.translation.y;
            car.vertical_velocity = (steepness * 2.8 + car.speed.abs() * 0.22).min(14.0);
            car.in_air = true;
        } else {
            transform.translation.y = ground_y;
            car.vertical_velocity = 0.0;
        }
    } else {
        car.vertical_velocity -= GRAVITY * dt;
        transform.translation.y += car.vertical_velocity * dt;
        car.air_time += dt;
        if transform.translation.y <= ground_y {
            transform.translation.y = ground_y;
            car.vertical_velocity = 0.0;
            car.in_air = false;
            car.air_time = 0.0;
        }
    }

    if car.in_air {
        car.pitch = (car.air_time * 2.0).sin() * 0.06;
    } else {
        car.pitch *= 0.85;
    }

    transform.rotation =
        Quat::from_rotation_y(car.heading + car.drift_angle) * Quat::from_rotation_x(car.pitch);
}
