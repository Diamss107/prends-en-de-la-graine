use crate::*;
use image::Rgba;
use std::collections::{HashMap, VecDeque};

mod build;

#[derive(Component)]
pub struct Province {
    pub points: Vec<Vec2>, // Liste des points qui définissent la frontière de la province
}

pub fn provinces_from_bmp_startup_system(mut commands: Commands) {
    // Charger l'image BMP
    let mut provinces_points = build::provinces_from_bmp(
        image::open(config::BMP_FILE)
            .expect("Impossible de charger l'image")
            .into_rgba8(),
    );

    // Tri final ultra-précis
    sort_provinces_points(&mut provinces_points);

    for (_, points) in provinces_points {
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
                config::PROVINCE_BORDER_COLOR.with_alpha(0.0), // Opacité initiale à 0 (invisible)
                4.0,
            ),
            Province {
                points, // Stocke les points pour la détection
            },
        ));
    }
}

pub fn province_hovered_update_system(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<(&Province, &mut Stroke)>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    // On vérifie que la souris est dans la fenêtre du jeu
    if let Some(cursor_position) = window.cursor_position() {
        // On récupère la position de la souris relativement au monde
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
        {
            // Pour chaque province, on vérifie si la souris est à l'intérieur
            for (province, mut stroke) in query.iter_mut() {
                let is_inside = is_point_inside_polygon(world_position, &province.points);

                // On fait apparaître le contour de la province si la souris est bien à l'intérieur
                // sinon on cache la province si la souris n'est plus à l'intérieur.
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

fn sort_provinces_points(provinces_points: &mut HashMap<Rgba<u8>, Vec<Vec2>>) {
    for (_, points) in provinces_points {
        if points.len() > 1 {
            // On trie part position
            points.sort_by(|a, b| {
                a.y.partial_cmp(&b.y)
                    .unwrap()
                    .then(a.x.partial_cmp(&b.x).unwrap())
            });

            let mut sorted = Vec::new();
            let mut remaining: VecDeque<Vec2> = points.drain(..).collect();

            // On trie par proximité
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
}
