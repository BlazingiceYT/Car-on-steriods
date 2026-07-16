use bevy::asset::LoadState;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use bevy::world_serialization::{WorldAsset, WorldAssetRoot};

pub const TRACK_SIZE: f32 = 300.0;
const GRID_SPACING: f32 = 20.0;

const MAP_MODEL_SCALE: f32 = 1.0;
const MAP_MODEL_Y_OFFSET: f32 = 0.0;
const MAP_MODEL_YAW_OFFSET: f32 = 0.0;

pub fn terrain_height_at(_x: f32, _z: f32) -> f32 {
    0.0
}

#[derive(Component)]
pub struct MapModel;

#[derive(Component)]
pub struct FallbackGround;

pub fn setup_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let map_scene: Handle<WorldAsset> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("map.glb"));
    commands.spawn((
        MapModel,
        WorldAssetRoot(map_scene),
        Transform::from_xyz(0.0, MAP_MODEL_Y_OFFSET, 0.0)
            .with_scale(Vec3::splat(MAP_MODEL_SCALE))
            .with_rotation(Quat::from_rotation_y(MAP_MODEL_YAW_OFFSET)),
    ));

    commands.spawn((
        FallbackGround,
        Visibility::Hidden,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(TRACK_SIZE, TRACK_SIZE))),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
    ));

    let line_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..default()
    });
    let line_along_z = meshes.add(Cuboid::new(0.3, 0.05, TRACK_SIZE));
    let line_along_x = meshes.add(Cuboid::new(TRACK_SIZE, 0.05, 0.3));
    let half_lines = (TRACK_SIZE / 2.0 / GRID_SPACING) as i32;

    for i in -half_lines..=half_lines {
        let offset = i as f32 * GRID_SPACING;
        commands.spawn((
            FallbackGround,
            Visibility::Hidden,
            Mesh3d(line_along_z.clone()),
            MeshMaterial3d(line_material.clone()),
            Transform::from_xyz(offset, 0.03, 0.0),
        ));
        commands.spawn((
            FallbackGround,
            Visibility::Hidden,
            Mesh3d(line_along_x.clone()),
            MeshMaterial3d(line_material.clone()),
            Transform::from_xyz(0.0, 0.03, offset),
        ));
    }

    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.1, 0.5, 0.0)),
    ));
}

pub fn map_model_fallback_system(
    asset_server: Res<AssetServer>,
    map_query: Query<&WorldAssetRoot, With<MapModel>>,
    mut fallback_query: Query<&mut Visibility, With<FallbackGround>>,
) {
    let Ok(scene_root) = map_query.single() else { return; };

    if let Some(LoadState::Failed(_)) = asset_server.get_load_state(&scene_root.0) {
        for mut visibility in fallback_query.iter_mut() {
            *visibility = Visibility::Visible;
        }
    }
}
