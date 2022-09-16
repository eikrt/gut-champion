use crate::environment::*;
use crate::graphics::*;
use crate::network::*;
use lerp::Lerp;
use serde::{Deserialize, Serialize};
const GRAVITY: f32 = 5.0;
const JUMP_STRENGTH: f32 = 188.0;
const SMASH_RATIO: f32 = 750.0;
const ENTITY_MARGIN: f32 = 8.0;

const TILT_TIME_SIDE: u128 = 186;
const TILT_TIME_UP: u128 = 48;
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct EntityHitbox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
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
    Dodge,
    Idle,
}
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum StatusType {
    Shield,
    Freeze,
    Stun,
    One,
    Two,
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
    pub stunned_sprite: Sprite,
    pub shield_sprite: Sprite,
    pub hitboxes: Vec<HitBox>,
    pub move_lock: bool,
    pub current_action: Action,
    pub current_class: ClassType,
    pub name: String,
    pub inv_time: f32,
    pub inv_change: f32,
    pub flying: bool,
    pub jump_counter: u8,
    pub drop: bool,
    pub freeze: bool,
    pub stunned: bool,
    pub walk_change: i32,
    pub walk_time: i32,
    pub smashing: bool,
    pub tilting: bool,
    pub up: bool,
    pub left: bool,
    pub down: bool,
    pub right: bool,
    pub up_released: bool,
    pub left_released: bool,
    pub down_released: bool,
    pub right_released: bool,
    pub hit_released: bool,
    pub hit: bool,
    pub special: bool,
    pub shield: bool,
    pub do_not_move: bool,
    pub smash_time: i32,
    pub smash_change: i32,
    pub hit_change: i32,
    pub tilt_change: i32,
    pub tilt_time: i32,
    pub drop_change: i32,
    pub drop_time: i32,
    pub freeze_change: i32,
    pub freeze_time: i32,
    pub stun_change: i32,
    pub stun_time: i32,
    pub shield_change: i32,
    pub shield_time: i32,
    pub ai_controlled: bool,
    pub target_entity: Option<Box<Entity>>,
    pub h_x: f32,
    pub h_y: f32,
    pub h_w: f32,
    pub h_h: f32,
    pub dodge: bool,
    pub dodge_change: i32,
    pub dodge_time: i32,
}
impl AsNetworkEntity for Entity {
    fn get_as_network_entity(&self) -> NetworkEntity {
        NetworkEntity {
            x: self.x,
            y: self.y,
            dx: self.dx,
            dy: self.dy,
            hp: self.hp,
            shield_percentage: self.shield_change as f32 / self.shield_time as f32,
            dir: self.dir,
            stocks: self.stocks,
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
impl Entity {
    pub fn new(
        x: f32,
        y: f32,
        current_class: ClassType,
        name: String,
        ai_controlled: bool,
        h_x: f32,
        h_y: f32,
        h_w: f32,
        h_h: f32,
    ) -> Entity {
        Entity {
            x: x,
            y: y,
            h: 0.0,
            w: 0.0,
            dx: 0.0,
            dy: 0.0,
            dir: true,
            hp: 0,
            flying: false,
            up: false,
            left: false,
            down: false,
            right: false,
            up_released: false,
            left_released: false,
            down_released: false,
            right_released: false,
            hit_released: false,
            special: false,
            hit: false,
            shield: false,
            smashing: false,
            tilting: false,
            stunned: false,
            do_not_move: false,
            next_step: (0.0, 0.0),
            collide_directions: (false, false, false, false),
            current_sprite: get_sprites(current_class.clone(), StatusType::One),
            freeze_sprite: get_sprites(current_class.clone(), StatusType::Freeze),
            stunned_sprite: get_sprites(current_class.clone(), StatusType::Stun),
            shield_sprite: get_sprites(current_class.clone(), StatusType::Shield),
            hitboxes: Vec::new(),
            move_lock: false,
            current_action: Action::action(current_class.clone(), ActionType::Idle, 1),
            current_class: current_class,
            name: name,
            inv_change: 0.0,
            inv_time: 1000.0,
            jump_counter: 0,
            drop: false,
            freeze: false,
            stocks: 3,
            walk_time: 250,
            walk_change: 0,
            smash_time: 0,
            smash_change: 0,
            drop_change: 0,
            drop_time: 300,
            freeze_change: 0,
            freeze_time: 64,
            stun_change: 0,
            stun_time: 3000,
            hit_change: 0,
            tilt_change: 0,
            shield_time: 2000,
            shield_change: 0,
            tilt_time: TILT_TIME_SIDE as i32,
            ai_controlled: ai_controlled,
            target_entity: None,
            dodge: false,
            h_x: h_x,
            h_y: h_y,
            h_w: h_w,
            h_h: h_h,
            dodge_time: 500,
            dodge_change: 0,
        }
    }
    pub fn ai_tick(&mut self, delta: u128) {
        let t_e = self.target_entity.as_ref().unwrap();
        if self.x > t_e.x + 8.0 {
            self.left = true;
            self.right = false;
            self.dir = false;
        } else if self.x < t_e.x - 8.0 {
            self.right = true;
            self.left = false;
            self.dir = true;
        } else {
            self.right = false;
            self.left = false;
        }
        if self.y < t_e.y {
            self.drop = true;
        }
        if self.y > t_e.y && self.next_step.1 >= 0.0 {
            self.jump();
        }
        self.hit = true;
    }
    pub fn tick(&mut self, delta: u128) {
        if self.ai_controlled {
            self.ai_tick(delta);
        }

        if self.drop {
            self.drop_change += delta as i32;
            if self.drop_change > self.drop_time {
                self.drop_change = 0;
                self.drop = false;
            }
        }
        if self.freeze {
            self.freeze_change += delta as i32;
            if self.freeze_change > self.freeze_time {
                self.freeze = false;
                self.freeze_change = 0;
            }
        }
        if self.stunned {
            self.stun_change += delta as i32;
            if self.stun_change > self.stun_time {
                self.stunned = false;
                self.stun_change = 0;
            }
        }
        self.accel_movement();
        self.process_hit_actions(delta);
        if self.current_action.action == ActionType::Idle
            && (self.next_step.0 > 0.1 || self.next_step.0 < -0.1)
        {
            self.walk_change += delta as i32;
            if self.walk_change > self.walk_time {
                if self.current_sprite == get_sprites(self.current_class.clone(), StatusType::One) {
                    self.current_sprite = get_sprites(self.current_class.clone(), StatusType::Two);
                } else {
                    self.current_sprite = get_sprites(self.current_class.clone(), StatusType::Two);
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
        if self.stunned {
            self.current_sprite = self.stunned_sprite.clone();
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
                self.current_sprite = get_sprites(self.current_class.clone(), StatusType::One);
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
        if self.stocks > 1
            && (self.y > 256.0 || self.y < -200.0 || self.x < -200.0 || self.x > 256.0 + 200.0)
        {
            self.die();
        }
    }
    pub fn die(&mut self) {
        self.stocks -= 1;
        self.x = 48.0;
        self.y = 0.0;
        self.hp = 0;
        self.dy = 0.0;
        self.dx = 0.0;
        self.stunned = false;
        self.stun_change = 0;
        self.freeze_change = 0;
        self.freeze = false;
    }
    pub fn process_hit_actions(&mut self, delta: u128) {
        if self.shield {
            self.shield_change += delta as i32;
            if self.shield_change > self.shield_time {
                self.shield_change = 0;
                self.shield = false;
                self.stunned = true;
            }
        } else {
            if self.shield_change > 0 {
                self.shield_change -= delta as i32;
            }
        }
        self.hit_change += delta as i32;
        if self.tilting && self.hit {
            self.smashing = true;
        }
        if self.smashing {
            self.smash_change += delta as i32;
        } else {
            self.smash_change = 1;
        }
        if (self.left || self.right) && self.shield {
            self.shield = false;
            self.dodge = true;
        }
        if self.tilting {
            self.tilt_change += delta as i32;
            if self.tilt_change > self.tilt_time {
                self.tilt_change = 0;
                self.tilting = false;
            }
        }
        if self.hit_released {
            if (self.left || self.right)
                && self.hit_change
                    > Action::action(
                        self.current_class.clone(),
                        ActionType::SideSmash,
                        self.smash_change.try_into().unwrap(),
                    )
                    .hit_time as i32
                && self.smashing
            {
                let hit_type = ActionType::SideSmash;
                self.smashing = false;
                self.do_not_move = true;

                self.execute_action(
                    delta,
                    Action::action(
                        self.current_class.clone(),
                        hit_type,
                        self.smash_change.try_into().unwrap(),
                    ),
                );
                self.hit_change = 0;
                self.hit_released = false;
            }
            if self.up
                && self.hit_change
                    > Action::action(
                        self.current_class.clone(),
                        ActionType::UpSmash,
                        self.smash_change.try_into().unwrap(),
                    )
                    .hit_time as i32
                && self.smashing
            {
                let mut hit_type = ActionType::UpSmash;
                hit_type = ActionType::UpSmash;
                self.smashing = false;
                self.do_not_move = true;
                self.execute_action(
                    delta,
                    Action::action(
                        self.current_class.clone(),
                        hit_type,
                        self.smash_change.try_into().unwrap(),
                    ),
                );
                self.hit_change = 0;
                self.up_released = false;
                self.hit_released = false;
            }
            self.hit_released = false;
        }

        if self.hit && !self.smashing {
            if !self.up
                && !self.down
                && !self.left
                && !self.right
                && self.next_step.1 == 0.0
                && self.hit_change
                    > Action::action(self.current_class.clone(), ActionType::Jab, 1).hit_time as i32
            {
                let mut hit_type = ActionType::Jab;
                self.execute_action(
                    delta,
                    Action::action(self.current_class.clone(), hit_type, 1),
                );
                self.hit_change = 0;
            }
            if !self.up
                && !self.down
                && !self.right
                && !self.left
                && self.next_step.1 != 0.0
                && self.hit_change
                    > Action::action(self.current_class.clone(), ActionType::Nair, 1).hit_time
                        as i32
            {
                self.execute_action(
                    delta,
                    Action::action(self.current_class.clone(), ActionType::Nair, 1),
                );
                self.hit_change = 0;
            }

            let mut hit_type = ActionType::Slide;
            if self.next_step.1 != 0.0 {
                hit_type = ActionType::Sair;
            }
            if (self.right || self.left)
                && self.hit_change
                    > Action::action(self.current_class.clone(), hit_type.clone(), 1).hit_time
                        as i32
            {
                self.execute_action(
                    delta,
                    Action::action(self.current_class.clone(), hit_type, 1),
                );
                self.hit_change = 0;
            }
            if self.up
                && self.hit_change
                    > Action::action(self.current_class.clone(), ActionType::Uair, 1).hit_time
                        as i32
            {
                let mut hit_type = ActionType::Uair;
                self.execute_action(
                    delta,
                    Action::action(self.current_class.clone(), ActionType::Uair, 1),
                );
                self.hit_change = 0;
                self.up_released = false;
            }
            if self.down
                && self.next_step.1 > 0.0
                && self.hit_change
                    > Action::action(self.current_class.clone(), ActionType::Dair, 1).hit_time
                        as i32
            {
                let mut hit_type = ActionType::Dair;
                self.execute_action(
                    delta,
                    Action::action(self.current_class.clone(), hit_type, 1),
                );
                self.hit_change = 0;
            }
        }
        if !self.right && !self.left && !self.up {
            self.smashing = false;
            self.do_not_move = false;
        }

        if self.up {
            self.tilt_time = TILT_TIME_UP as i32;
        } else {
            self.tilt_time = TILT_TIME_SIDE as i32;
        }
        self.dodge_change += delta as i32;
        if self.dodge && self.dodge_change > self.dodge_time {
            let mut hit_type = ActionType::Dodge;
            self.execute_action(
                delta,
                Action::action(self.current_class.clone(), hit_type, 1),
            );
            self.dodge = false;
            self.dodge_change = 0;
        }
    }
    pub fn release_shield(&mut self) {
        self.shield = false;
    }
    pub fn accel_movement(&mut self) {
        let speed = match self.current_action.action {
            ActionType::Dodge => 120.0,
            _ => 60.0,
        };
        if self.stunned {
            self.shield_change = 0;
            if self.next_step.1 == 0.0 {
                self.dx = 0.0;
            }
            return;
        }

        let acc_ratio = match self.flying {
            true => 0.02,
            false => 0.5,
        };
        if self.left && !self.smashing && !self.do_not_move {
            self.dx = self.dx.lerp(-speed, acc_ratio);
        } else if self.right && !self.smashing && !self.do_not_move {
            self.dx = self.dx.lerp(speed, acc_ratio);
        } else if !self.flying
            && self.next_step.1 == 0.0
            && self.current_action.action != ActionType::Dodge
        {
            self.dx = self.dx.lerp(0.0, 0.2);
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
    pub fn start_shield(&mut self) {
        if self.next_step.1 < -0.1 && self.next_step.1 > 0.1 {
            return;
        }
        if self.next_step.0 > 0.1 && self.next_step.0 < -0.1 {
            return;
        }

        self.shield = true;
    }
    pub fn take_hit(&mut self, delta: u128, hitbox: &NetworkBare) {
        if !hitbox.active {
            return;
        }
        if self.inv_change < self.inv_time {
            return;
        }

        if self.shield {
            self.shield_change += 20;
            return;
        }
        if self.current_action.action == ActionType::Dodge {
            return;
        }
        self.freeze = true;
        let hitbox_action = Action::action(hitbox.class.clone(), hitbox.action.clone(), 1);
        let hit_multiplier = 1.0 + self.hp as f32 / 90.0;
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
        if self.stunned {
            return;
        }
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
            if self.x + self.h_x < hitbox.x as f32 + hitbox.w as f32
                && self.x + self.h_x + self.h_w > hitbox.x as f32
                && self.y + self.h_y < hitbox.y as f32 + hitbox.h as f32
                && self.y + self.h_y + self.h_h > hitbox.y as f32
            {
                self.take_hit(delta, &hitbox);
            }
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

                ActionType::Dodge => Action {
                    w: 0.0,
                    h: 0.0,
                    x: -0.0,
                    y: 0.0,
                    knock_x: 0.0 * hit_ratio,
                    knock_y: 0.0 * hit_ratio,
                    damage: 0.0 * hit_ratio,
                    hit_time: 1000.0,
                    duration: 200.0,
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
                ActionType::Dodge => Action {
                    w: 0.0,
                    h: 0.0,
                    x: -0.0,
                    y: 0.0,
                    knock_x: 0.0 * hit_ratio,
                    knock_y: 0.0 * hit_ratio,
                    damage: 0.0 * hit_ratio,
                    hit_time: 1000.0,
                    duration: 200.0,
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
                    knock_y: -50.0 * hit_ratio,
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
