use bevy::prelude::*;

pub const TRACK_SIZE: f32 = 300.0;
const GRID_SPACING: f32 = 20.0;

/// Ground height at a given (x, z) world position.
///
/// Stubbed flat for now since you're building your real map later. Once you
/// have real terrain, replace the body of this function with real height
/// sampling (heightmap lookup, raycast against your terrain mesh, etc.) —
/// the car's jump/bump physics in `car.rs` already reads from this function,
/// so nothing else needs to change when you do.
pub fn terrain_height_at(_x: f32, _z: f32) -> f32 {
    0.0
}

/// Placeholder world: a flat plane with a grid painted on it so you have
/// somewhere to drive, plus the one directional light. Swap this whole
/// system out once your real map is ready — nothing in `car.rs` or
/// `camera.rs` depends on it.
pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane — one mesh, one material, one draw call.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(TRACK_SIZE, TRACK_SIZE))),
        MeshMaterial3d(materials.add(Color::srgb(0.13, 0.13, 0.15))), // JS matRoad 0x222226
    ));

    // Grid lines — two shared mesh handles + one shared material, so every
    // line is a cheap instance of the same geometry rather than unique data.
    let line_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true, // flat/unshaded, matching the original's MeshBasicMaterial lines
        ..default()
    });
    let line_along_z = meshes.add(Cuboid::new(0.3, 0.05, TRACK_SIZE));
    let line_along_x = meshes.add(Cuboid::new(TRACK_SIZE, 0.05, 0.3));
    let half_lines = (TRACK_SIZE / 2.0 / GRID_SPACING) as i32;

    for i in -half_lines..=half_lines {
        let offset = i as f32 * GRID_SPACING;
        commands.spawn((
            Mesh3d(line_along_z.clone()),
            MeshMaterial3d(line_material.clone()),
            Transform::from_xyz(offset, 0.03, 0.0),
        ));
        commands.spawn((
            Mesh3d(line_along_x.clone()),
            MeshMaterial3d(line_material.clone()),
            Transform::from_xyz(0.0, 0.03, offset),
        ));
    }

    // Directional light — shadows off on purpose, per the "no heavy shadow
    // mapping" requirement from the original brief.
    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.1, 0.5, 0.0)),
    ));
}
