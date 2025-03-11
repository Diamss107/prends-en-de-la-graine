use bevy::math::Vec2;
use image::{Rgba, RgbaImage};
use std::collections::HashMap;

pub fn provinces_from_bmp(color_bmp: RgbaImage) -> HashMap<Rgba<u8>, Vec<Vec2>> {
    let mut provinces_limit_points: HashMap<Rgba<u8>, Vec<Vec2>> = HashMap::new();

    for y in 0..color_bmp.height() {
        for x in 0..color_bmp.width() {
            let pixel = BmpProvincePixel::new(x, y, *color_bmp.get_pixel(x, y));

            if pixel.is_gray() {
                continue;
            }

            if pixel.is_limit_of_a_province(&color_bmp) {
                provinces_limit_points
                    .entry(pixel.color)
                    .or_insert_with(Vec::new)
                    .push(pixel.get_world_pos(&color_bmp));
            }
        }
    }

    provinces_limit_points
}

struct BmpProvincePixel {
    x: u32,
    y: u32,
    color: Rgba<u8>,
}

impl BmpProvincePixel {
    pub fn new(x: u32, y: u32, color: Rgba<u8>) -> BmpProvincePixel {
        BmpProvincePixel { x, y, color }
    }

    pub fn is_gray(&self) -> bool {
        if self.color[0] == self.color[1] && self.color[1] == self.color[2] {
            return true;
        }
        false
    }

    fn get_neighbors(&self, color_bmp: &RgbaImage) -> [(Option<u32>, Option<u32>); 4] {
        [
            // Haut
            (Some(self.x), self.y.checked_sub(1)),
            // Droite
            (
                self.x.checked_add(1).filter(|&v| v < color_bmp.width()),
                Some(self.y),
            ),
            // Bas
            (
                Some(self.x),
                self.y.checked_add(1).filter(|&v| v < color_bmp.height()),
            ),
            // Gauche
            (self.x.checked_sub(1), Some(self.y)),
        ]
    }

    fn get_world_pos(&self, color_bmp: &RgbaImage) -> Vec2 {
        Vec2::new(
            (self.x as f32) - ((color_bmp.width() / 2) as f32),
            ((color_bmp.height() as f32) - (self.y as f32)) - ((color_bmp.height() / 2) as f32),
        )
    }

    fn is_limit_of_a_province(&self, color_bmp: &RgbaImage) -> bool {
        let neighbors = self.get_neighbors(color_bmp);

        for n_pos in neighbors {
            if let (Some(nx), Some(ny)) = n_pos {
                let neighbor_color = color_bmp.get_pixel(nx, ny);
                if *neighbor_color != self.color {
                    return true;
                }
            }
        }

        false
    }
}
