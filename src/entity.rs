use crate::environment::*;
use crate::graphics::*;
use crate::network::*;
use lerp::Lerp;
use serde::{Deserialize, Serialize};
const GRAVITY: f32 = 5.0;
const JUMP_STRENGTH: f32 = 188.0;
const SMASH_RATIO: f32 = 750.0;
const ENTITY_MARGIN: f32 = 8.0;
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum ActionType {
    Jab,
    Nair,
    Dair,
    Uair,
    Sair,
    Slide,
    SideSmash,
    UpSmash,
    Idle,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClassType {
    Commodore,
    Alchemist,
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
            class: self.action.class.clone(),
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
    pub stocks: i32,
    pub dir: bool,
    pub next_step: (f32, f32),
    pub collide_directions: (bool, bool, bool, bool),
    pub current_sprite: Sprite,
    pub freeze_sprite: Sprite,
    pub hitboxes: Vec<HitBox>,
    pub move_lock: bool,
    pub current_action: Action,
    pub current_class: ClassType,
    pub name: String,
    pub inv_time: f32,
    pub inv_change: f32,
    pub flying: bool,
    pub jump_counter: u8,
    pub collide_sides: bool,
    pub drop: bool,
    pub freeze: bool,
    pub walk_change: i32,
    pub walk_time: i32,
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
            stocks: self.stocks,
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
    pub class: ClassType,
}
impl Action {
    pub fn action(class: ClassType, action: ActionType, smash_change: u128) -> Action {
        let mut hit_ratio = smash_change as f32 / SMASH_RATIO;
        if smash_change == 1 {
            hit_ratio = 1.0;
        }
        match class {
            ClassType::Commodore => match action {
                ActionType::Jab => Action {
                    w: 12.0,
                    h: 12.0,
                    x: -12.0,
                    y: 8.0,
                    knock_x: 10.0 * hit_ratio,
                    knock_y: 5.0 * hit_ratio,
                    damage: 10.0,
                    hit_time: 1000.0,
                    duration: 100.0,
                    action: action,
                    class: class,
                },

                ActionType::Slide => Action {
                    w: 12.0,
                    h: 12.0,
                    x: -8.0,
                    y: 4.0,
                    knock_x: 10.0 * hit_ratio,
                    knock_y: 10.0 * hit_ratio,
                    damage: 15.0 * hit_ratio,
                    hit_time: 1000.0,
                    duration: 750.0,
                    action: action,
                    class: class,
                },
                ActionType::Nair => Action {
                    w: 12.0,
                    h: 12.0,
                    x: -16.0,
                    y: 8.0,
                    knock_x: 20.0 * hit_ratio,
                    knock_y: 20.0 * hit_ratio,
                    damage: 20.0 * hit_ratio,
                    hit_time: 1000.0,
                    duration: 750.0,
                    action: action,
                    class: class,
                },
                ActionType::Uair => Action {
                    w: 12.0,
                    h: 12.0,
                    x: -16.0,
                    y: -4.0,
                    knock_x: 0.0 * hit_ratio,
                    knock_y: 25.0 * hit_ratio,
                    damage: 25.0 * hit_ratio,
                    hit_time: 1000.0,
                    duration: 750.0,
                    action: action,
                    class: class,
                },
                ActionType::Dair => Action {
                    w: 12.0,
                    h: 12.0,
                    x: -16.0,
                    y: 16.0,
                    knock_x: 0.0 * hit_ratio,
                    knock_y: 25.0 * hit_ratio,
                    damage: 25.0 * hit_ratio,
                    hit_time: 1000.0,
                    duration: 750.0,
                    action: action,
                    class: class,
                },
                ActionType::Sair => Action {
                    w: 12.0,
                    h: 12.0,
                    x: -8.0,
                    y: 4.0,
                    knock_x: 25.0 * hit_ratio,
                    knock_y: 15.0 * hit_ratio,
                    damage: 25.0 * hit_ratio,
                    hit_time: 1000.0,
                    duration: 750.0,
                    action: action,
                    class: class,
                },
                ActionType::SideSmash => Action {
                    w: 12.0,
                    h: 12.0,
                    x: -12.0,
                    y: 8.0,
                    knock_x: 30.0 * hit_ratio,
                    knock_y: 15.0 * hit_ratio,
                    damage: 40.0 * hit_ratio,
                    hit_time: 1000.0,
                    duration: 100.0,
                    action: action,
                    class: class,
                },
                ActionType::UpSmash => Action {
                    w: 12.0,
                    h: 12.0,
                    x: -16.0,
                    y: -4.0,
                    knock_x: 0.0 * hit_ratio,
                    knock_y: 40.0 * hit_ratio,
                    damage: 40.0 * hit_ratio,
                    hit_time: 1000.0,
                    duration: 750.0,
                    action: action,
                    class: class,
                },
                ActionType::Idle => Action {
                    w: 0.0,
                    h: 0.0,
                    x: 0.0,
                    y: 0.0,
                    knock_x: 0.0 * hit_ratio,
                    knock_y: 0.0 * hit_ratio,
                    damage: 0.0 * hit_ratio,
                    hit_time: 1000.0,
                    duration: 750.0,
                    action: action,
                    class: class,
                },
            },
            ClassType::Alchemist => match action {
                ActionType::Jab => Action {
                    w: 12.0,
                    h: 12.0,
                    x: -12.0,
                    y: 8.0,
                    knock_x: 10.0 * hit_ratio,
                    knock_y: 10.0 * hit_ratio,
                    damage: 10.0,
                    hit_time: 1000.0,
                    duration: 100.0,
                    action: action,
                    class: class,
                },

                ActionType::Slide => Action {
                    w: 16.0,
                    h: 16.0,
                    x: -18.0,
                    y: 4.0,
                    knock_x: 10.0 * hit_ratio,
                    knock_y: 10.0 * hit_ratio,
                    damage: 5.0 * hit_ratio,
                    hit_time: 1000.0,
                    duration: 750.0,
                    action: action,
                    class: class,
                },
                ActionType::Nair => Action {
                    w: 19.0,
                    h: 6.0,
                    x: -19.0,
                    y: 10.0,
                    knock_x: 30.0 * hit_ratio,
                    knock_y: 30.0 * hit_ratio,
                    damage: 15.0 * hit_ratio,
                    hit_time: 1000.0,
                    duration: 750.0,
                    action: action,
                    class: class,
                },
                ActionType::Uair => Action {
                    w: 12.0,
                    h: 8.0,
                    x: -16.0,
                    y: 0.0,
                    knock_x: 0.0 * hit_ratio,
                    knock_y: 25.0 * hit_ratio,
                    damage: 25.0 * hit_ratio,
                    hit_time: 1000.0,
                    duration: 750.0,
                    action: action,
                    class: class,
                },
                ActionType::Dair => Action {
                    w: 12.0,
                    h: 12.0,
                    x: -16.0,
                    y: 16.0,
                    knock_x: 0.0 * hit_ratio,
                    knock_y: 50.0 * hit_ratio,
                    damage: 25.0 * hit_ratio,
                    hit_time: 1000.0,
                    duration: 750.0,
                    action: action,
                    class: class,
                },
                ActionType::Sair => Action {
                    w: 12.0,
                    h: 12.0,
                    x: -8.0,
                    y: 4.0,
                    knock_x: 25.0 * hit_ratio,
                    knock_y: 15.0 * hit_ratio,
                    damage: 45.0 * hit_ratio,
                    hit_time: 1000.0,
                    duration: 750.0,
                    action: action,
                    class: class,
                },
                ActionType::SideSmash => Action {
                    w: 12.0,
                    h: 12.0,
                    x: -12.0,
                    y: 8.0,
                    knock_x: 30.0 * hit_ratio,
                    knock_y: 20.0 * hit_ratio,
                    damage: 40.0 * hit_ratio,
                    hit_time: 1000.0,
                    duration: 100.0,
                    action: action,
                    class: class,
                },
                ActionType::UpSmash => Action {
                    w: 12.0,
                    h: 8.0,
                    x: -16.0,
                    y: 0.0,
                    knock_x: 0.0 * hit_ratio,
                    knock_y: 40.0 * hit_ratio,
                    damage: 40.0 * hit_ratio,
                    hit_time: 1000.0,
                    duration: 750.0,
                    action: action,
                    class: class,
                },
                ActionType::Idle => Action {
                    w: 0.0,
                    h: 0.0,
                    x: 0.0,
                    y: 0.0,
                    knock_x: 0.0 * hit_ratio,
                    knock_y: 0.0 * hit_ratio,
                    damage: 0.0 * hit_ratio,
                    hit_time: 1000.0,
                    duration: 750.0,
                    action: action,
                    class: class,
                },
            },
        }
    }
}
pub struct Class {}
impl Class {
    pub fn Ant() -> Class {
        Class {}
    }
}
impl Entity {
    pub fn tick(&mut self, delta: u128) {
        if self.stocks < 0 {
            std::process::exit(0);
        }
        if self.current_action.action == ActionType::Idle {
            self.walk_change += delta as i32;
            if self.walk_change > self.walk_time {
                if self.current_sprite == get_sprites(self.current_class.clone(), "2".to_string()) {
                    self.current_sprite = get_sprites(self.current_class.clone(), "1".to_string());
                } else
                {
                    self.current_sprite = get_sprites(self.current_class.clone(), "1".to_string());
                }
                self.walk_change = 0;
            }
        } else {
            self.current_sprite = get_animations(
                self.current_class.clone().clone(),
                self.current_action.action.clone(),
            );
        }
        if self.freeze {
            self.current_sprite = self.freeze_sprite.clone();
        }
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
                self.current_sprite = get_sprites(self.current_class.clone(), "1".to_string());
                hitbox.active = false;
                self.move_lock = false;
            }
        }
        self.hitboxes.retain(|h| h.active);
        if self.hitboxes.len() == 0 {
            self.current_action.action = ActionType::Idle;
        }
        if self.next_step.1 == 0.0 {
            self.jump_counter = 0;
        }
        if self.y > 256.0 {
            self.stocks -= 1;
            self.x = 48.0;
            self.y = 0.0;
            self.hp = 0;
            self.dy = 0.0;
            self.dx = 0.0;
        }
    }
    pub fn execute_action(&mut self, delta: u128, action: Action) {
        if self.freeze {
            return;
        }
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
        self.freeze = true;
        let hitbox_action = Action::action(hitbox.class.clone(), hitbox.action.clone(), 1);
        let hit_multiplier = 1.0 + self.hp as f32 / 80.0;
        let hit_multiplier_knock = 3.0 + self.hp as f32 / 10.0;
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
        self.flying = true;
    }
    pub fn execute_movement(&mut self) {
        self.move_to(self.next_step)
    }
    pub fn jump(&mut self) {
        if self.next_step.1 == 0.0 {
            self.jump_counter = 0;
        }
        if self.jump_counter > 1 {
            return;
        }

        self.dy = -JUMP_STRENGTH;

        self.jump_counter += 1;
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
    pub fn collide_with_obstacle(&mut self, delta: u128, other: &Obstacle) {
        let mut drop_self = true;
        if other.obstacle_type == ObstacleType::Platform {
            if self.drop {
                drop_self = false;
            }
        }
        if self.y + self.next_step.1 + self.h < other.y + other.h
            && self.y + self.next_step.1 + self.h > other.y
            && self.x + self.w - ENTITY_MARGIN > other.x
            && self.x + ENTITY_MARGIN < other.x + other.w
            && self.next_step.1 > 0.0
            && drop_self
        {
            self.next_step.1 = 0.0;
            self.collide_directions.2 = true;
            self.dy = 0.0;
            self.y = other.y - self.h;
            self.flying = false;
        }

        if self.x + self.next_step.0 < other.x + other.w
            && self.x + self.next_step.0 + self.w > other.x
            && self.y + self.h > other.y + 2.0
            && self.y + self.h < other.y + other.h
            && other.obstacle_type == ObstacleType::Stage
        {
            self.next_step.0 = 0.0;
            self.collide_directions.3 = true;
            self.collide_directions.1 = true;
            self.dx = 0.0;

            self.flying = false;
        }
    }
    pub fn collide_with_hitboxes(&mut self, delta: u128, other: &NetworkEntity) {
        for hitbox in &other.hitboxes {
            if self.x < hitbox.x as f32 + hitbox.w as f32
                && self.x + self.w > hitbox.x as f32
                && self.y < hitbox.y as f32 + hitbox.h as f32
                && self.y + self.h > hitbox.y as f32
            {
                self.take_hit(delta, &hitbox);
            }
        }
    }
}
