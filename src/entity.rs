use serde::{Deserialize, Serialize};

const GRAVITY: f32 = 5.0;
const JUMP_STRENGTH: f32 = 128.0;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entity {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub dx: f32,
    pub dy: f32,
    pub next_step: (f32,f32),
    pub collide_directions: (bool, bool, bool, bool),
    pub current_sprite: String,
}
impl Entity {
    pub fn tick(&mut self, delta: u128) {
        self.dy += GRAVITY; 
        self.calculate_step(delta);
    }
    pub fn execute_movement(&mut self) {
        self.move_to(self.next_step) 
    }
    pub fn jump(&mut self) {
        if self.next_step.1 == 0.0 {
            self.dy -= JUMP_STRENGTH;
        }
    }
    pub fn calculate_step(&mut self, delta: u128) {
        self.next_step.0 = (self.dx * delta as f32) as f32 / 1000.0;
        self.next_step.1 = (self.dy * delta as f32) as f32 / 1000.0;
    }
    pub fn move_to(&mut self, step: (f32,f32)) {
        self.x += step.0;
        if !self.collide_directions.2 {
            self.y += step.1;
        }
        self.collide_directions = (false,false,false,false);
    }
    pub fn collide_with(&mut self, other: &Entity) {
        if self.y + self.next_step.1 + self.h > other.y && self.x > other.x && self.x < other.x + other.w {
            self.next_step.1 = 0.0;
            self.collide_directions.2 = true;
            self.dy = 0.0;
            self.y = other.y - self.h;
        }
    }
}
