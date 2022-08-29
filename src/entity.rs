use serde::{Deserialize, Serialize};

const GRAVITY: f32 = 10.0;
#[derive(Serialize, Deserialize, Clone)]
pub struct Entity {
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
    pub current_sprite: String,
}
impl Entity {
    pub fn tick(&mut self, delta: u128) {
        self.dy += GRAVITY; 
        self.y += (self.dy * delta as f32) as f32 / 1000.0;
    }
}
