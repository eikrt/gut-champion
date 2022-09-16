use crate::entity::{ActionType, ClassType, StatusType};
use lerp::Lerp;
use sdl2::event::Event;
use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::ttf::Font;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
const CAMERA_SPEED: f32 = 32.0;
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Sprite {
    Ground,
    Platform,
    Alchemist,
    Alchemist2,
    AlchemistJab,
    AlchemistNair,
    AlchemistUair,
    AlchemistDair,
    AlchemistSair,
    AlchemistSlide,
    AlchemistSideSmash,
    AlchemistUpSmash,
    AlchemistFreeze,
    AlchemistStun,
    AlchemistShield,
    AlchemistDodge,
    Commodore,
    Commodore2,
    CommodoreJab,
    CommodoreNair,
    CommodoreUair,
    CommodoreDair,
    CommodoreSair,
    CommodoreSlide,
    CommodoreSideSmash,
    CommodoreUpSmash,
    CommodoreFreeze,
    CommodoreStun,
    CommodoreShield,
    CommodoreDodge,
    Basement,
    LongButtonMain,
    LongButtonHovered,
    LongButtonPressed,
    Placeholder,
    Shield,
}
pub fn get_animations(class: ClassType, action: ActionType) -> Sprite {
    match class {
        ClassType::Commodore => match action {
            ActionType::Jab => Sprite::CommodoreJab,
            ActionType::Nair => Sprite::CommodoreNair,
            ActionType::Dair => Sprite::CommodoreDair,
            ActionType::Uair => Sprite::CommodoreUair,
            ActionType::Sair => Sprite::CommodoreSair,
            ActionType::Slide => Sprite::CommodoreSlide,
            ActionType::SideSmash => Sprite::CommodoreSideSmash,
            ActionType::UpSmash => Sprite::CommodoreUpSmash,
            ActionType::Idle => Sprite::Commodore,
            ActionType::Dodge => Sprite::CommodoreDodge,
        },
        ClassType::Alchemist => match action {
            ActionType::Jab => Sprite::AlchemistJab,
            ActionType::Nair => Sprite::AlchemistNair,
            ActionType::Dair => Sprite::AlchemistDair,
            ActionType::Uair => Sprite::AlchemistUair,
            ActionType::Sair => Sprite::AlchemistSair,
            ActionType::Slide => Sprite::AlchemistSlide,
            ActionType::SideSmash => Sprite::AlchemistSideSmash,
            ActionType::UpSmash => Sprite::AlchemistUpSmash,
            ActionType::Idle => Sprite::Alchemist,
            ActionType::Dodge => Sprite::AlchemistDodge,
        },
    }
}
pub fn get_sprites(class: ClassType, key: StatusType) -> Sprite {
    match class {
        ClassType::Commodore => match key {
            StatusType::Freeze => Sprite::CommodoreFreeze,
            StatusType::Stun => Sprite::CommodoreStun,
            StatusType::Shield => Sprite::CommodoreShield,
            StatusType::One => Sprite::Commodore,
            StatusType::Two => Sprite::Commodore2,
            _ => Sprite::Commodore,
        },
        ClassType::Alchemist => match key {
            StatusType::Freeze => Sprite::AlchemistFreeze,
            StatusType::Stun => Sprite::AlchemistStun,
            StatusType::Shield => Sprite::AlchemistShield,
            StatusType::One => Sprite::Alchemist,
            StatusType::Two => Sprite::Alchemist2,
            _ => Sprite::Alchemist,
        },
    }
}

pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
}
impl Camera {
    pub fn tick(&mut self, delta: u128) {
        self.x += (self.dx * delta as f32) as f32 / 1000.0;
        self.y += (self.dy * delta as f32) as f32 / 1000.0;
    }
    pub fn move_towards_point(&mut self, target_x: f32, target_y: f32) {
        let s_x = self.x + 256.0 / 2.0 - 8.0;
        let s_y = self.y + 144.0 / 2.0 - 16.0;
        let angle = (target_y - s_y).atan2(target_x - s_x);
        // let angle = target_y.atan2(self.y) - target_x.atan2(self.x);
        let dist = ((target_y - s_y).powf(2.0) + (target_x - s_x).powf(2.0)).sqrt();
        if dist < 8.0 {
            self.dx = self.dx.lerp(0.0, 0.3);
            self.dy = self.dy.lerp(0.0, 0.3);
            return;
        }
        self.dx = (angle.cos() - 0.0 * 3.14 / 1.0) * CAMERA_SPEED;
        self.dy = (angle.sin() - 0.0 * 3.14 / 1.0) * CAMERA_SPEED;
    }
}
pub struct Text<'a> {
    pub text_surface: Surface<'a>,
    pub text_texture: Texture<'a>,
    pub text_sprite: Rect,
}
pub fn get_text<'a, T>(
    msg: String,
    color: Color,
    font_size: u16,
    font: &Font,
    texture_creator: &'a TextureCreator<T>,
) -> Option<Text<'a>> {
    let mut message = msg.clone();
    if msg.is_empty() {
        message = "unknown".to_string();
    }
    let text_surface = font
        .render(&message)
        .blended(color)
        .map_err(|e| e.to_string())
        .ok()
        .unwrap();
    let text_texture = texture_creator
        .create_texture_from_surface(&text_surface)
        .map_err(|e| e.to_string())
        .ok()
        .unwrap();
    let text_sprite = Rect::new(
        0,
        0,
        (font_size as f32 / 2.0) as u32 * msg.len() as u32,
        (font_size as f32) as u32,
    );

    let text = Text {
        text_surface: text_surface,
        text_texture: text_texture,
        text_sprite: text_sprite,
    };
    return Some(text);
}
pub fn render_text(
    canvas: &mut WindowCanvas,
    texture: &Texture,
    position: (i32, i32),
    sprite: Rect,
    ratio_x: f32,
    ratio_y: f32,
) {
    let screen_rect = Rect::new(
        (position.0 as f32 / ratio_x) as i32,
        (position.1 as f32 / ratio_y) as i32,
        (sprite.width() as f32 / ratio_x) as u32,
        (sprite.height() as f32 / ratio_y) as u32,
    );
    canvas.copy(texture, None, screen_rect);
}
