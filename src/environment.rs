use crate::graphics::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tiled::{FiniteTileLayer, Loader, Map};
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub enum ObstacleType {
    Platform,
    Stage,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Obstacle {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub current_sprite: Sprite,
    pub obstacle_type: ObstacleType,
}
pub fn generate_map(layer: &FiniteTileLayer) {
    let (width, height) = (layer.width() as usize, layer.height() as usize);
    for x in 0..width as i32 {
        for y in 0..height as i32 {
            if let Some(tile) = layer.get_tile(x, y) {
                if tile.image != None {
                    println!("{:?}", tile);
                }
            }
        }
    }
}
pub fn get_environment() -> HashMap<u64, Obstacle> {
    let mut loader = Loader::new();
    let map = loader.load_tmx_map("./maps/arena.tmx").unwrap();
    let mut tile_map: HashMap<u64, Obstacle> = HashMap::new();
    for layer in map.layers() {
        generate_map(match &layer.layer_type() {
            tiled::LayerType::TileLayer(l) => match l {
                tiled::TileLayer::Finite(f) => f,
                tiled::TileLayer::Infinite(_) => panic!("Infinite maps not supported"),
            },
            tiled::LayerType::ObjectLayer(l) => match l {
                _ => todo!(),
            },
            tiled::LayerType::ImageLayer(l) => match l {
                _ => todo!(),
            },
            tiled::LayerType::GroupLayer(l) => match l {
                _ => todo!(),
            },
        });
    }
    let mut rng = rand::thread_rng();
    HashMap::from([
        (
            rng.gen(),
            Obstacle {
                x: 24.0,
                y: 70.0,
                h: 0.0,
                w: 0.0,
                current_sprite: Sprite::Ground,
                obstacle_type: ObstacleType::Stage,
            },
        ),
        (
            rng.gen(),
            Obstacle {
                x: 50.0,
                y: 40.0,
                h: 0.0,
                w: 0.0,
                current_sprite: Sprite::Platform,
                obstacle_type: ObstacleType::Platform,
            },
        ),
        (
            rng.gen(),
            Obstacle {
                x: 108.0,
                y: 10.0,
                h: 0.0,
                w: 0.0,
                current_sprite: Sprite::Platform,
                obstacle_type: ObstacleType::Platform,
            },
        ),
        (
            rng.gen(),
            Obstacle {
                x: 170.0,
                y: 40.0,
                h: 0.0,
                w: 0.0,
                current_sprite: Sprite::Platform,
                obstacle_type: ObstacleType::Platform,
            },
        ),
    ])
}
