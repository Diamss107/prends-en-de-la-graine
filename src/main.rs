use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

pub mod config;
pub mod map;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ShapePlugin)
        .insert_resource(ClearColor(Color::WHITE))
        .add_systems(
            Startup,
            (setup, map::map_startup_sytem).chain(),
        )
        .add_systems(Update, map::map_update_system) // Gestion de l'opacité avec la souris
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Charger l'image de fond
    let texture_handle = asset_server.load(config::MAP_FILE);

    // Ajouter l'image de fond comme Sprite
    commands.spawn((
        Sprite {
            image: texture_handle,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 0.0, -10.0),
            ..default()
        },
    ));

    commands.spawn((
        Camera2d,
        Transform {
            scale: Vec3::new(2.5, 2.5, 20.0),
            ..default()
        },
    ));
}