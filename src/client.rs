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
use rand::Rng;
use mpsc::TryRecvError;
use std::{
    io::{self, ErrorKind},
    sync::mpsc,
};
use crate::network::*;
const SCALE: f32 = 4.0;
const SCREEN_WIDTH: u32 = 256 * SCALE as u32; 
const SCREEN_HEIGHT: u32 = 144 * SCALE as u32;
const TILE_SIZE: f32 = 64.0;
const LOCAL: &str = "127.0.0.1:8888";
const MSG_SIZE: usize = 1024;
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
    let sprites = HashMap::from([
        (
        "weatherant",
        texture_creator.load_texture("res/player.png")?,
    ),

        (
        "ground",
        texture_creator.load_texture("res/ground.png")?,
    )
    ]);
    let mut rng = rand::thread_rng();
    let player_id = rng.gen();
    let mut entities: HashMap<u64, Entity> = HashMap::from([(player_id, Entity {
        
        x: 0.0,
        y: 0.0,
        dx: 0.0,
        dy: 0.0,
        current_sprite: "weatherant".to_string(),
    })]);
    let mut environment: HashMap<u64, Entity> = HashMap::from([(rng.gen(), Entity {
        
        x: 24.0,
        y: 80.0,
        dx: 0.0,
        dy: 0.0,
        current_sprite: "ground".to_string(),
    })]);
    let mut w = false;
    let mut a = false;
    let mut s = false;
    let mut d = false;
    let mut running = true;
    let mut event_pump = sdl_context.event_pump()?;
    let mut compare_time = SystemTime::now();
    // socket stuff
    

    let mut client = TcpStream::connect(LOCAL).expect("Connection failed...");
    client
        .set_nonblocking(true);

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
               // println!("message recv {:?}", msg);
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection failed with server...");
                break;
            }
        }
        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff);
               // println!("message sent {:?}", msg);
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,
        }

        thread::sleep(Duration::from_millis(64));
    });

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

    canvas.set_draw_color(bg_color);
    canvas.clear();
        for e in entities.values_mut() {
            let texture = &sprites[e.current_sprite.as_str()];
        canvas.copy(
            texture,
            Rect::new(0, 0, texture.query().width, texture.query().height),
            Rect::new(
                (e.x * SCALE as f32) as i32,
                (e.y * SCALE as f32) as i32,
                texture.query().width * SCALE as u32,
                texture.query().height * SCALE as u32,
            ),
        )?;
            e.tick(delta.as_millis());
        }
        for e in environment.values() {
            let texture = &sprites[e.current_sprite.as_str()];
        canvas.copy(
            texture,
            Rect::new(0, 0, texture.query().width, texture.query().height),
            Rect::new(
                (e.x * SCALE as f32) as i32,
                (e.y * SCALE as f32) as i32,
                texture.query().width * SCALE as u32,
                texture.query().height * SCALE as u32,
            ),
        )?;
        }
        canvas.present();
        compare_time = SystemTime::now();

        let msg = serde_json::to_string(&SendState {
            player: entities.get(&player_id).unwrap().clone()
        }).unwrap();
        if tx.send(msg).is_err() {
            break;
        }
        thread::sleep(time::Duration::from_millis(20));
    }

    Ok(())
}
pub fn main() {
    main_loop();
}



