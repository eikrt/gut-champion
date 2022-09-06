use crate::graphics::{get_animations, Sprite};
#[derive(PartialEq)]
pub enum ButtonAction {
    Connect,
}
pub struct Button {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub main_sprite: Sprite,
    pub hovered_sprite: Sprite,
    pub pressed_sprite: Sprite,
    pub pressed: bool,
    pub hovered: bool,
    pub action: ButtonAction,
    pub text: String,
    pub index: i32,

}
