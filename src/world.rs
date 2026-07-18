use bevy::asset::LoadState;
use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;

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

#[derive(Resource)]
pub struct MapAsset {
    handle: Handle<Gltf>,
    scene_spawned: bool,
    fallback_shown: bool,
}

pub fn setup_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(MapAsset {
        handle: asset_server.load("map.glb"),
        scene_spawned: false,
        fallback_shown: false,
    });

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
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gltfs: Res<Assets<Gltf>>,
    mut map_asset: ResMut<MapAsset>,
    mut fallback_query: Query<&mut Visibility, With<FallbackGround>>,
) {
    if map_asset.scene_spawned || map_asset.fallback_shown {
        return;
    }

    match asset_server.get_load_state(&map_asset.handle) {
        Some(LoadState::Loaded) => {
            let Some(gltf) = gltfs.get(&map_asset.handle) else {
                return;
            };
            let Some(scene) = gltf
                .default_scene
                .clone()
                .or_else(|| gltf.scenes.first().cloned())
            else {
                warn!("map.glb loaded, but it does not contain a scene to spawn");
                show_fallback_grid(&mut fallback_query);
                map_asset.fallback_shown = true;
                return;
            };

            commands.spawn((
                MapModel,
                WorldAssetRoot(scene),
                Transform::from_xyz(0.0, MAP_MODEL_Y_OFFSET, 0.0)
                    .with_scale(Vec3::splat(MAP_MODEL_SCALE))
                    .with_rotation(Quat::from_rotation_y(MAP_MODEL_YAW_OFFSET)),
            ));
            map_asset.scene_spawned = true;
            info!("map.glb loaded and spawned");
        }
        Some(LoadState::Failed(error)) => {
            error!("Failed to load map.glb: {error:?}");
            show_fallback_grid(&mut fallback_query);
            map_asset.fallback_shown = true;
        }
        _ => {}
    }
}

fn show_fallback_grid(fallback_query: &mut Query<&mut Visibility, With<FallbackGround>>) {
    for mut visibility in fallback_query.iter_mut() {
        *visibility = Visibility::Visible;
    }
}
