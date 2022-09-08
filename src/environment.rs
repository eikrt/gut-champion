use serde::{Deserialize, Serialize};
use crate::graphics::*;
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


