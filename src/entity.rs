use lerp::Lerp;
use serde::{Deserialize, Serialize};
use crate::client::Sprite;

const GRAVITY: f32 = 5.0;
const JUMP_STRENGTH: f32 = 188.0;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ActionType {
    jab,
    nair,
    dair,
    uair,
    slide,
    smash,
}
pub enum ClassType {
    ant,
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
}
pub trait AsNetworkBare {
    fn get_as_network_bare(&self) -> NetworkBare;
}
pub trait AsNetworkEntity {
    fn get_as_network_entity(&self) -> NetworkEntity;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HitBox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub duration: f32,
    pub change: f32,
    pub active: bool,
    pub action: Action,
    pub dir: bool,
}

impl AsNetworkBare for HitBox {
    fn get_as_network_bare(&self) -> NetworkBare {
        NetworkBare {
            x: self.x as i32,
            y: self.y as i32,
            w: self.w as i32,
            h: self.h as i32,
            dir: self.dir,
            action: self.action.action.clone(),
            active: self.active,
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entity {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub dx: f32,
    pub dy: f32,
    pub hp: i32,
    pub dir: bool,
    pub next_step: (f32, f32),
    pub collide_directions: (bool, bool, bool, bool),
    pub current_sprite: Sprite,
    pub hitboxes: Vec<HitBox>,
    pub move_lock: bool,
    pub current_action: Action,
    pub name: String,
    pub inv_time: f32,
    pub inv_change: f32,
}
impl AsNetworkEntity for Entity {
    fn get_as_network_entity(&self) -> NetworkEntity {
        NetworkEntity {
            x: self.x,
            y: self.y,
            dx: self.dx,
            dy: self.dy,
            hp: self.hp,
            dir: self.dir,
            //h: self.h,
            // w: self.w,
            //next_step: self.next_step,
            // collide_directions: self.collide_directions,
            hitboxes: self
                .hitboxes
                .clone()
                .into_iter()
                .map(|h| h.get_as_network_bare())
                .collect(),
            name: self.name.clone(),
            current_sprite: self.current_sprite.clone(),
        }
    }
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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Action {
    pub w: f32,
    pub h: f32,
    pub x: f32,
    pub y: f32,
    pub knock_x: f32,
    pub knock_y: f32,
    pub damage: f32,
    pub hit_time: f32,
    pub duration: f32,
    pub action: ActionType, 
}
impl Action {
    pub fn action(class: ClassType, action: ActionType) -> Action {
        match action {
            ActionType::jab => Action {
                w: 12.0,
                h: 12.0,
                x: 2.0,
                y: 4.0,
                knock_x: 2.0,
                knock_y: 2.0,
                damage: 2.0,
                hit_time: 1000.0,
                duration: 100.0,
                action: action,
            },

            ActionType::slide => Action {
                w: 12.0,
                h: 12.0,
                x: -8.0,
                y: 4.0,
                knock_x: 10.0,
                knock_y: 10.0,
                damage: 15.0,
                hit_time: 1000.0,
                duration: 750.0,
                action: action,

            },
            ActionType::nair => Action {
                w: 12.0,
                h: 12.0,
                x: -8.0,
                y: 4.0,
                knock_x: 20.0,
                knock_y: 20.0,
                damage: 20.0,
                hit_time: 1000.0,
                duration: 750.0,
                action: action,
            },
            ActionType::uair => Action {
                w: 12.0,
                h: 12.0,
                x: -8.0,
                y: -8.0,
                knock_x: 25.0,
                knock_y: 25.0,
                damage: 25.0,
                hit_time: 1000.0,
                duration: 750.0,
                action: action,
            },
            ActionType::dair => Action {
                w: 12.0,
                h: 12.0,
                x: -8.0,
                y: 14.0,
                knock_x: 25.0,
                knock_y: 25.0,
                damage: 25.0,
                hit_time: 1000.0,
                duration: 750.0,
                action: action,
            },
            ActionType::smash => Action {
                w: 12.0,
                h: 4.0,
                x: -8.0,
                y: 8.0,
                knock_x: 50.0,
                knock_y: 50.0,
                damage: 40.0,
                hit_time: 1000.0,
                duration: 750.0,
                action: action,
            },
        }
    }
}
pub struct Class {}
impl Class {
    pub fn ant() -> Class {
        Class {}
    }
}
impl Entity {
    pub fn tick(&mut self, delta: u128) {
        self.inv_change += delta as f32;
        for hitbox in &mut self.hitboxes {
            let mut h_x = self.x;
            let mut h_y = self.y + self.current_action.y;
            if self.dir {
                h_x += self.current_action.x + self.w
            } else {
                h_x -= self.current_action.w + self.current_action.x;
            }
            hitbox.x = h_x;
            hitbox.y = h_y;
        }
        self.dy += GRAVITY;
        self.calculate_step(delta);
        for hitbox in &mut self.hitboxes {
            hitbox.change += delta as f32;
            if hitbox.change > hitbox.duration {
                hitbox.active = false;
                self.move_lock = false;
            }
        }
        self.hitboxes.retain(|h| h.active);
    }
    pub fn execute_action(&mut self, delta: u128, action: Action) {
        self.current_action = action.clone();
        let mut h_x = self.x;
        let mut h_y = self.y + action.y;
        if self.dir {
            h_x += action.x + self.w
        } else {
            h_x -= action.w + action.x;
        }
        self.hitboxes.push(HitBox {
            x: h_x,
            y: h_y,
            w: action.w,
            h: action.h,
            dir: self.dir,
            duration: action.duration,
            change: 0.0,
            active: true,
            action: action,
        });
        self.move_lock = true;
    }
    pub fn take_hit(&mut self, delta: u128, hitbox: &NetworkBare) {
        if !hitbox.active {
            return;
        }
        if self.inv_change < self.inv_time {
            return;
        }
        let hitbox_action = Action::action(ClassType::ant, hitbox.action.clone());
        let hit_multiplier = 1.0 + self.hp as f32 / 100.0;
        let hit_multiplier_knock = 3.0 + self.hp as f32 / 50.0;
        if hitbox.dir {
            self.dx += 5.0 + hitbox_action.knock_x * hit_multiplier_knock;
            self.dy -= 5.0 + hitbox_action.knock_y * hit_multiplier_knock;
            self.hp += (hitbox_action.damage * hit_multiplier) as i32;
        }
        if !hitbox.dir {
            self.dx -= 5.0 + hitbox_action.knock_x * hit_multiplier_knock;
            self.dy -= 5.0 + hitbox_action.knock_y * hit_multiplier_knock;
            self.hp += (hitbox_action.damage * hit_multiplier) as i32;
        }

        self.inv_change = 0.0;
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
    pub fn collide_with_hitboxes(&mut self, delta: u128, other: &NetworkEntity) {
        for hitbox in &other.hitboxes {
            if self.x + self.w / 2.0 + self.next_step.0 > hitbox.x as f32
                && self.x + self.w / 2.0 + self.next_step.0 < (hitbox.x + hitbox.w) as f32
                && self.y + self.h / 2.0 + self.next_step.1 > hitbox.y as f32
                && self.y + self.h / 2.0 + self.next_step.1 < (hitbox.y + hitbox.h) as f32
            {
                self.take_hit(delta, &hitbox);
            }
        }
    }
}
