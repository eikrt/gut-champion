use serde::{Deserialize, Serialize};

const GRAVITY: f32 = 5.0;
const JUMP_STRENGTH: f32 = 128.0;
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
    pub current_sprite: String,
    pub hitboxes: Vec<HitBox>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Action {
    pub w: f32,
    pub h: f32,
    pub x: f32,
    pub y: f32,
    pub knock_x: f32,
    pub knock_y: f32,
    pub damage: f32,
}
impl Action {
    pub fn jab() -> Action {
        Action {
            w: 12.0,
            h: 12.0,
            x: 2.0,
            y: 4.0,
            knock_x: 2.0,
            knock_y: 2.0,
            damage: 2.0,
        }
    }
}
pub struct Class {}
impl Entity {
    pub fn tick(&mut self, delta: u128) {
        self.dy += GRAVITY;
        self.calculate_step(delta);
        for hitbox in &mut self.hitboxes {
            hitbox.change += delta as f32;
            if hitbox.change > hitbox.duration {
                hitbox.active = false;
            }
        }
        self.hitboxes.retain(|h| h.active);
    }
    pub fn execute_action(&mut self, delta: u128, action: Action) {
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
            duration: 100.0,
            change: 0.0,
            active: true,
            action: action,
        })
    }
    pub fn take_hit(&mut self, delta: u128, hitbox: &HitBox) {
        let hit_multiplier = 1.0 + self.hp as f32 / 100.0;
        if hitbox.dir {
            self.dx += hitbox.action.knock_x * hit_multiplier;
            self.dy -= hitbox.action.knock_y * hit_multiplier;
            self.hp += (hitbox.action.damage * hit_multiplier) as i32;
        }
        if !hitbox.dir {
            self.dx -= hitbox.action.knock_x * hit_multiplier;
            self.dy -= hitbox.action.knock_y * hit_multiplier;
            self.hp += (hitbox.action.damage * hit_multiplier) as i32;
        }
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
        for hitbox in &other.hitboxes {
            if self.x + self.w / 2.0 + self.next_step.0 > hitbox.x
                && self.x + self.w / 2.0 + self.next_step.0 < hitbox.x + hitbox.w
                && self.y + self.h / 2.0 + self.next_step.1 > hitbox.y
                && self.y + self.h / 2.0 + self.next_step.1 < hitbox.y + hitbox.h
            {
                self.take_hit(delta, hitbox);
            }
        }
    }
}
