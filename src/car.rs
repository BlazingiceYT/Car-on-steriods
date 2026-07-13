use bevy::asset::LoadState;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use bevy::world_serialization::{WorldAsset, WorldAssetRoot};

// ─── Speed/acceleration constants — units are km/h (and km/h-per-second) ───
pub const BASE_MAX_SPEED: f32 = 125.0;
pub const NITRO_MAX_SPEED: f32 = 175.0;
const ACCEL: f32 = 49.0;         // normal acceleration (lowered 25% from prior tuning)
const NITRO_ACCEL: f32 = 40.0;   // slower than normal — nitro ramps up, doesn't snap
const BRAKE: f32 = 105.0;
const FRICTION: f32 = 35.0;

const STEER_SPEED: f32 = 2.5;
const MAX_STEER: f32 = 0.46;
const TURN_DRIFT: f32 = 0.025;   // heading change rate while actually drifting
const TURN_GRIP: f32 = 0.014;    // heading change rate for normal (non-drift) turning — gentler, not sharp
const DRIFT_MIN_SPEED: f32 = 55.0;
const GRAVITY: f32 = 20.0;

const NITRO_DRAIN_PER_SEC: f32 = 40.0;
const NITRO_RECHARGE_PER_SEC: f32 = 15.0;

pub const SPAWN_X: f32 = 0.0;
pub const SPAWN_Z: f32 = 15.0;

const CAR_MODEL_SCALE: f32 = 1.0;
const CAR_MODEL_YAW_OFFSET: f32 = std::f32::consts::FRAC_PI_2;
const CAR_MODEL_Y_OFFSET: f32 = 0.0;

#[derive(Component, Debug, Default)]
pub struct Car {
    pub speed: f32,
    pub heading: f32,
    pub steer_angle: f32,
    pub drift_angle: f32,
    pub vertical_velocity: f32,
    pub in_air: bool,
    pub air_time: f32,
    pub pitch: f32,
}

#[derive(Resource)]
pub struct NitroState {
    pub amount: f32, // 0.0 to 100.0
}

impl Default for NitroState {
    fn default() -> Self {
        Self { amount: 100.0 }
    }
}

#[derive(Resource)]
pub struct NitroParticleTimer(pub Timer);

#[derive(Component)]
pub struct NitroParticle {
    pub velocity: Vec3,
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct GlbCarModel;

#[derive(Component)]
pub struct FallbackCarModel;

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
                Mesh3d(meshes.add(Cuboid::new(2.0, 0.62, 4.2))),
                MeshMaterial3d(materials.add(Color::srgb(0.90, 0.24, 0.24))),
                Transform::from_xyz(0.0, 0.5, 0.0),
            ));
            parent.spawn((
                FallbackCarModel,
                Visibility::Hidden,
                Mesh3d(meshes.add(Cuboid::new(1.38, 0.5, 2.0))),
                MeshMaterial3d(materials.add(Color::srgb(0.10, 0.13, 0.17))),
                Transform::from_xyz(0.0, 1.01, -0.2),
            ));
        });
}

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
    mut nitro: ResMut<NitroState>,
    mut query: Query<(&mut Car, &mut Transform)>,
) {
    let Ok((mut car, mut transform)) = query.single_mut() else { return; };

    let dt = time.delta_secs().min(0.0333);

    let gas = keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp);
    let brake = keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown);
    let left = keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft);
    let right = keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight);

    // ── Nitro ──
    let nitro_held = keys.pressed(KeyCode::KeyN);
    let nitro_active = nitro_held && nitro.amount > 0.0;

    if nitro_active {
        nitro.amount = (nitro.amount - NITRO_DRAIN_PER_SEC * dt).max(0.0);
    } else {
        nitro.amount = (nitro.amount + NITRO_RECHARGE_PER_SEC * dt).min(100.0);
    }

    let max_speed = if nitro_active { NITRO_MAX_SPEED } else { BASE_MAX_SPEED };
    let accel = if nitro_active { NITRO_ACCEL } else { ACCEL };

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
        // If nitro was pushing you over the normal cap and you let off nitro,
        // let friction pull you back down toward the normal max naturally —
        // no special-case needed since friction already reduces speed.
    }
    // If nitro just ran out mid-boost, don't let speed sit above the normal
    // cap forever — bleed it down toward BASE_MAX_SPEED under friction.
    if !nitro_active && car.speed > BASE_MAX_SPEED {
        car.speed = (car.speed - FRICTION * dt).max(BASE_MAX_SPEED);
    }

    // Steering — ramps toward MAX_STEER while held, springs back to center.
    if left {
        car.steer_angle = (car.steer_angle + STEER_SPEED * dt).min(MAX_STEER);
    } else if right {
        car.steer_angle = (car.steer_angle - STEER_SPEED * dt).max(-MAX_STEER);
    } else {
        car.steer_angle *= 1.0 - 11.0 * dt;
    }

    // Drift only happens if you brake AND turn. Turning without braking
    // uses the gentler TURN_GRIP rate instead — not sharp.
    let is_drifting =
        brake && (left || right) && car.speed.abs() > DRIFT_MIN_SPEED && car.steer_angle.abs() > 0.16;

    if car.speed.abs() > 0.5 {
        let direction = if car.speed > 0.0 { 1.0 } else { -1.0 };
        let turn_rate = if is_drifting { TURN_DRIFT * 1.65 } else { TURN_GRIP };
        car.heading += car.steer_angle * direction * turn_rate;
    }

    if is_drifting {
        let target_drift = car.steer_angle * 1.15;
        car.drift_angle += (target_drift - car.drift_angle) * 9.0 * dt;
    } else {
        car.drift_angle += (0.0 - car.drift_angle) * 6.0 * dt;
    }

    let move_angle = car.heading - car.drift_angle * 0.32;
    transform.translation.x += move_angle.sin() * car.speed * dt;
    transform.translation.z += move_angle.cos() * car.speed * dt;

    // ── Vertical / jump physics over terrain bumps ──
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

/// Spawns a couple of small glowing particles behind the car each tick while
/// nitro is active. Purely cosmetic — despawn themselves after a short life.
pub fn nitro_particle_spawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keys: Res<ButtonInput<KeyCode>>,
    nitro: Res<NitroState>,
    mut spawn_timer: ResMut<NitroParticleTimer>,
    query: Query<&Transform, With<Car>>,
) {
    let nitro_active = keys.pressed(KeyCode::KeyN) && nitro.amount > 0.0;
    if !nitro_active {
        return;
    }
    spawn_timer.0.tick(time.delta());
    if !spawn_timer.0.finished() {
        return;
    }

    let Ok(car_transform) = query.single() else { return; };

    for side in [-0.4_f32, 0.4] {
        let offset = car_transform.rotation * Vec3::new(side, 0.3, 2.0);
        let pos = car_transform.translation + offset;
        let backward = car_transform.rotation * Vec3::new(0.0, 0.0, 1.0);

        commands.spawn((
            NitroParticle {
                velocity: backward * 6.0 + Vec3::new(0.0, 1.0, 0.0),
                lifetime: Timer::from_seconds(0.4, TimerMode::Once),
            },
            Mesh3d(meshes.add(Sphere::new(0.15))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.4, 0.8, 1.0, 0.9),
                emissive: Color::srgb(0.3, 0.6, 1.0).into(),
                unlit: true,
                ..default()
            })),
            Transform::from_translation(pos),
        ));
    }
}

pub fn nitro_particle_update_system(
    mut commands: Commands,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(Entity, &mut Transform, &mut NitroParticle, &MeshMaterial3d<StandardMaterial>)>,
) {
    for (entity, mut transform, mut particle, mat_handle) in query.iter_mut() {
        particle.lifetime.tick(time.delta());
        transform.translation += particle.velocity * time.delta_secs();
        let t = particle.lifetime.fraction();
        transform.scale = Vec3::splat((1.0 - t).max(0.05));
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.base_color.set_alpha(1.0 - t);
        }
        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
