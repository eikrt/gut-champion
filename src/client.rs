use crate::entity::Entity;
use sdl2::event::Event;
use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{thread, time};
const SCALE: u32 = 4;
const SCREEN_WIDTH: u32 = 256 * SCALE;
const SCREEN_HEIGHT: u32 = 144 * SCALE;
const TILE_SIZE: f32 = 64.0;
struct Camera {
    x: f32,
    y: f32,
}
fn main_loop() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("Gut Champion", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");
    let mut canvas = window
        .into_canvas()
        .build()
        .expect("could not make a canvas");

    let _image_context = image::init(InitFlag::PNG | InitFlag::JPG)?;
    let texture_creator = canvas.texture_creator();
    let bg_color = Color::RGB(255, 255, 255);
    let tile_color = Color::RGB(128, 64, 55);
    let floor_color = Color::RGB(64, 32, 30);
    let player_color = Color::RGB(128, 128, 0);
    canvas.set_draw_color(bg_color);
    canvas.clear();
    let sprites = HashMap::from([(
        "weatherant",
        texture_creator.load_texture("res/player.png")?,
    )]);
    let mut entities: Vec<Entity> = Vec::from([Entity {
        x: 0,
        y: 0,
        currentSprite: "weatherant".to_string(),
    }]);
    let mut w = false;
    let mut a = false;
    let mut s = false;
    let mut d = false;
    let mut running = true;
    let mut event_pump = sdl_context.event_pump()?;
    let mut compare_time = SystemTime::now();
    while running {
        let delta = SystemTime::now().duration_since(compare_time).unwrap();
        if delta.as_millis() / 10 != 0 {
            //   println!("FPS: {}", 100 / (delta.as_millis()/10));
        }
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    running = false;
                }
                // WASD
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    w = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    a = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    s = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    d = true;
                }

                // WASD
                Event::KeyUp {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    w = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    a = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    s = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    d = false;
                }
                _ => {}
            }
        }
        for e in entities.iter() {
            let texture = &sprites[e.currentSprite.as_str()];
        canvas.copy(
            texture,
            Rect::new(e.x, e.y, texture.query().width, texture.query().height),
            Rect::new(
                0,
                0,
                texture.query().width * SCALE,
                texture.query().height * SCALE,
            ),
        )?;
        }
        canvas.present();
        compare_time = SystemTime::now();
        thread::sleep(time::Duration::from_millis(20));
    }

    println!("Socket connection ended.");
    Ok(())
}
pub fn run() {
    main_loop();
}
