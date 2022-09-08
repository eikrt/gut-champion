use serde::{Deserialize, Serialize};
use crate::entity::*;
use crate::graphics::Sprite;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SendState {
    pub id: u64,
    pub player: NetworkEntity,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkBare {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub active: bool,
    pub dir: bool,
    pub action: ActionType,
    pub class: ClassType,
}
pub trait AsNetworkBare {
    fn get_as_network_bare(&self) -> NetworkBare;
}
pub trait AsNetworkEntity {
    fn get_as_network_entity(&self) -> NetworkEntity;
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkEntity {
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
    //pub w: f32,
    //pub h: f32,
    //pub next_step: (f32,f32),
    pub hp: i32,
    pub stocks: i32,
    pub dir: bool,
    pub hitboxes: Vec<NetworkBare>,
    pub name: String,
    pub current_sprite: Sprite,
    //pub collide_directions: (bool, bool, bool, bool),
}
impl NetworkEntity {
    pub fn tick(&mut self, delta: u128) {
        //let next_step = self.calculate_step(delta);
        //self.execute_movement(next_step);
        //println!("old {}", self.x);
        //self.x = self.x.lerp(self.x + next_step.0, 10.0 / delta as f32);
        //
        //println!("new {}", self.x);
        //
    }
    pub fn execute_movement(&mut self, next_step: (f32,f32)) {
        self.move_to(next_step)
    }
    pub fn move_to(&mut self, step: (f32, f32)) {
        self.x += step.0;
        self.y += step.1;
    }
    pub fn calculate_step(&mut self, delta: u128) -> (f32, f32) {
        let mut next_step = (0.0, 0.0);
        next_step.0 = (self.dx * delta as f32) as f32 / 1000.0;
        next_step.1 = (self.dy * delta as f32) as f32 / 1000.0;
        next_step
    }
}
/*impl NetworkEntity {
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
    pub fn move_to(&mut self, step: (f32, f32)) {
        self.x += step.0;
        if !self.collide_directions.2 {
            self.y += step.1;
        }
        self.collide_directions = (false, false, false, false);
    }
    pub fn collide_with(&mut self, delta: u128, other: &Entity) {
        if self.y + self.next_step.1 + self.h < other.y + other.h
            && self.y + self.next_step.1 + self.h > other.y
            && self.x > other.x
            && self.x < other.x + other.w
        {
            self.next_step.1 = 0.0;
            self.collide_directions.2 = true;
            self.dy = 0.0;
            self.y = other.y - self.h;
        }

        if self.x + self.next_step.0 < other.x + other.w
            && self.x + self.next_step.0 + self.w > other.x
            && self.y > other.y + 5.0
            && self.y < other.y + other.h
        {
            self.next_step.0 = 0.0;
            self.collide_directions.3 = true;
            self.collide_directions.1 = true;
            self.dx = 0.0;
        }
    }
}*/

pub fn is_zero(buf: &Vec<u8>) -> bool {
    for byte in buf.into_iter() {
        if *byte != 0 {
            return false;
        }
    }
    return true;
}
