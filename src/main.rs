#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod level;
mod maze;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(level::LevelPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("textures/icon.png"),
        ..Default::default()
    });
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(11.0, 30.0, 10.0)
            .looking_at(Vec3::new(10.0, 0.0, 10.0), Vec3::Y),
        ..Default::default()
    });

    commands.spawn_bundle(level::LevelLoaderBundle {
        level_loader: level::LevelLoader {
            dimensions: level::DimensionLength::Three([4, 15, 5]),
            ..Default::default()
        },
        ..Default::default()
    });
}
