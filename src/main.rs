use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use std::collections::{HashMap, VecDeque};

const MAP_FILE: &str = "map.png"; // Chemin vers le fichier de la map
const BMP_FILE: &str = "assets/provinces.bmp"; // Chemin vers le fichier BMP contenant les provinces
const PROVINCE_BORDER_COLOR: Color = Color::srgb(250.0, 200.0, 0.0);

#[derive(Component)]
struct Province {
    color: [u8; 3],    // Identifiant unique basé sur la couleur
    points: Vec<Vec2>, // Liste des points qui définissent la frontière de la province
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ShapePlugin)
        .insert_resource(ClearColor(Color::WHITE))
        .add_systems(Startup, setup)
        .add_systems(Update, update_province_visibility) // Gestion de l'opacité avec la souris
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Charger l'image de fond
    let texture_handle = asset_server.load(MAP_FILE);

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

    // Charger l'image BMP
    let bmp = image::open(BMP_FILE)
        .expect("Impossible de charger l'image")
        .into_rgba8();
    let (width, height) = bmp.dimensions();

    let mut provinces_points: HashMap<[u8; 3], Vec<Vec2>> = HashMap::new();
    for y in 0..height {
        for x in 0..width {
            let pixel = bmp.get_pixel(x, y);
            let pixel_color = [pixel[0], pixel[1], pixel[2]];

            if pixel_color == [255, 255, 255] {
                continue;
            }

            let mut is_limit_pixel = false;

            let neighbors = [
                (x.checked_sub(1), Some(y)),                         // Gauche
                (x.checked_add(1).filter(|&v| v < width), Some(y)),  // Droite
                (Some(x), y.checked_sub(1)),                         // Haut
                (Some(x), y.checked_add(1).filter(|&v| v < height)), // Bas
            ];

            for (nx, ny) in neighbors {
                if let (Some(nx), Some(ny)) = (nx, ny) {
                    let neighbor_pixel = bmp.get_pixel(nx, ny);
                    let neighbor_color = [neighbor_pixel[0], neighbor_pixel[1], neighbor_pixel[2]];
                    if neighbor_color == [255, 255, 255] || neighbor_color != pixel_color {
                        is_limit_pixel = true;
                        break;
                    }
                }
            }

            if is_limit_pixel {
                let pos = Vec2::new(
                    (x as f32) - ((width / 2) as f32),
                    ((height as f32) - (y as f32)) - ((height / 2) as f32),
                );
                provinces_points
                    .entry(pixel_color)
                    .or_insert_with(Vec::new)
                    .push(pos);
            }
        }
    }

    // Tri final ultra-précis
    for (color, points) in &mut provinces_points {
        if points.len() > 1 {
            points.sort_by(|a, b| {
                a.y.partial_cmp(&b.y)
                    .unwrap()
                    .then(a.x.partial_cmp(&b.x).unwrap())
            });

            let mut sorted = Vec::new();
            let mut remaining: VecDeque<Vec2> = points.drain(..).collect();

            if let Some(start) = remaining.pop_front() {
                sorted.push(start);

                while !remaining.is_empty() {
                    let last = *sorted.last().unwrap();

                    if let Some((index, _)) = remaining
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| last.distance(**p) < 10.0)
                        .min_by(|(_, a), (_, b)| {
                            last.distance(**a).partial_cmp(&last.distance(**b)).unwrap()
                        })
                    {
                        sorted.push(remaining.remove(index).unwrap());
                    } else {
                        break;
                    }
                }
            }

            *points = sorted;
        }
    }

    for (color, points) in provinces_points {
        let shape = shapes::Polygon {
            points: points.clone(),
            closed: true,
        };

        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                ..default()
            },
            Stroke::new(
                PROVINCE_BORDER_COLOR.with_alpha(0.0), // Opacité initiale à 0 (invisible)
                4.0,
            ),
            Province {
                color,
                points, // Stocke les points pour la détection
            },
        ));
    }
}

fn update_province_visibility(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<(&Province, &mut Stroke)>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    if let Some(cursor_position) = window.cursor_position() {
        if let Some(world_position) =
            screen_to_world(cursor_position, window, camera, camera_transform)
        {
            for (province, mut stroke) in query.iter_mut() {
                // Vérifier si la souris est À L’INTÉRIEUR de la province
                let is_inside = is_point_inside_polygon(world_position, &province.points);

                // Modifier l'opacité si la souris est dans la province
                if is_inside {
                    stroke.color = stroke.color.with_alpha(1.0);
                } else {
                    stroke.color = stroke.color.with_alpha(0.0);
                }
            }
        }
    }
}

/// Vérifie si un point (ex: souris) est À L’INTÉRIEUR d'un polygone (ex: une province)
fn is_point_inside_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    let mut crossings = 0;
    let mut j = polygon.len() - 1; // Dernier point du polygone

    for i in 0..polygon.len() {
        let v1 = polygon[i];
        let v2 = polygon[j];

        if ((v1.y > point.y) != (v2.y > point.y)) // Vérifier si on traverse le segment en y
            && (point.x < (v2.x - v1.x) * (point.y - v1.y) / (v2.y - v1.y) + v1.x)
        {
            crossings += 1;
        }
        j = i;
    }

    crossings % 2 == 1 // Si impair → le point est à l'intérieur
}

/// Convertit la position de la souris en coordonnées monde en prenant en compte le viewport
fn screen_to_world(
    cursor_position: Vec2,
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    // Convertit la position écran → monde en utilisant `viewport_to_world_2d`
    camera
        .viewport_to_world_2d(camera_transform, cursor_position)
        .ok()
}

/// Vérifie si un point (souris) est proche d'une ligne définie par deux points
fn point_near_line(cursor: Vec2, p1: Vec2, p2: Vec2, threshold: f32) -> bool {
    let line_vec = p2 - p1;
    let point_vec = cursor - p1;
    let projection = point_vec.dot(line_vec) / line_vec.length_squared();

    if projection < 0.1 || projection > 1.0 {
        return false; // La projection est en dehors du segment
    }

    let closest_point = p1 + projection * line_vec;
    cursor.distance(closest_point) < threshold
}
