use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Obstacle {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub current_sprite: String,
}


