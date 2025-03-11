use crate::config;
use crate::map::provinces::Province;
use bevy::color::Alpha;
use bevy::prelude::*;
use bevy_prototype_lyon::draw::Stroke;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::geometry::GeometryBuilder;
use bevy_prototype_lyon::shapes;
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::fs;

mod provinces;

#[derive(Debug, Deserialize)]
struct MapConfig {
    provinces: ProvincesConfig,
}

#[derive(Debug, Deserialize)]
struct ProvincesConfig {
    #[serde(deserialize_with = "deserialize_colors_to_tags")]
    colors_to_tags: HashMap<[u8; 3], String>,
}

pub fn map_startup_sytem(mut commands: Commands) {
    let color_to_tag_toml_content = fs::read_to_string(config::MAP_CONFIG_FILE).unwrap();
    let map_config: MapConfig = toml::from_str(&color_to_tag_toml_content).unwrap();

    let map_provinces = provinces::provinces_from_bmp(config::BMP_FILE, map_config.provinces);

    for province in map_provinces {
        let shape = shapes::Polygon {
            points: province.limit_points.clone(),
            closed: true,
        };

        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                ..default()
            },
            Stroke::new(
                config::PROVINCE_BORDER_COLOR.with_alpha(0.0), // Opacité initiale à 0 (invisible)
                4.0,
            ),
            province,
        ));
    }
}

pub fn map_update_system(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    query: Query<(&Province, &mut Stroke)>,
) {
    provinces::province_hovered_update_system(windows, camera_query, query)
}

fn deserialize_colors_to_tags<'de, D>(deserializer: D) -> Result<HashMap<[u8; 3], String>, D::Error>
where
    D: Deserializer<'de>,
{
    // On va d'abord désérialiser en un HashMap<String, String>,
    // puis convertir la clé "255,0,0" en [255,0,0].
    let raw_map: HashMap<String, String> = HashMap::deserialize(deserializer)?;

    let mut result = HashMap::new();
    for (key, value) in raw_map {
        // key = "255,0,0"
        let arr = parse_key_to_color(&key).map_err(|e| de::Error::custom(e))?;
        result.insert(arr, value);
    }
    Ok(result)
}

// Petit parseur qui split sur ',' et parse chaque composante en u8.
fn parse_key_to_color(s: &str) -> Result<[u8; 3], String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 3 {
        return Err(format!("Clé couleur invalide: {}", s));
    }
    let mut arr = [0u8; 3];
    for (i, p) in parts.iter().enumerate() {
        arr[i] = p
            .trim()
            .parse()
            .map_err(|_| format!("Composante non valide dans {}", s))?;
    }
    Ok(arr)
}
