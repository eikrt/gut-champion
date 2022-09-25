use crate::graphics::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
const TILE_SIZE: usize = 8;
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
pub fn get_environment() -> HashMap<u64, Obstacle> {
    let mut rng = rand::thread_rng();
    let mut map: HashMap<u64, Obstacle> = HashMap::new();
    let contents = fs::read_to_string("maps/arena").expect("Couldn't read file...");

    for (i, row) in contents.split("\n").enumerate() {
        for (j, element) in row.chars().enumerate() {
            let mut sprite = Sprite::Ground;
            let mut obstacle_type = ObstacleType::Stage;
            if element == '1' {
                sprite = Sprite::Platform;
                obstacle_type = ObstacleType::Platform;
            }
            else if element == '2' {
                sprite = Sprite::Ground;
                obstacle_type = ObstacleType::Stage;
            }
            else {
                continue;
            }
            map.insert(
                rng.gen(),
                Obstacle {
                    x: (j * TILE_SIZE) as f32,
                    y: (i * TILE_SIZE) as f32,
                    w: 0.0,
                    h: 0.0,
                    current_sprite: sprite,
                    obstacle_type: obstacle_type,
                },
            );
        }
    }
    map
    /*HashMap::from([
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
    ]);*/
}
