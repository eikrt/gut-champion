use crate::button::*;
use crate::client_threads::*;
use crate::entity::*;
use crate::environment::*;
use crate::graphics::*;
use crate::graphics::{get_animations, Sprite};
use crate::network::*;
use bincode;
use lerp::Lerp;
use mpsc::TryRecvError;
use rand::Rng;
use sdl2::event::Event;
use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Texture;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::ttf::Font;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{
    env, fs,
    io::{self, ErrorKind},
    process::exit,
    sync::mpsc,
    thread, time,
};
const SCALE: f32 = 4.0;
const RESOLUTION_X: u32 = 256;
const RESOLUTION_Y: u32 = 144;
const SCREEN_WIDTH: u32 = 256 * SCALE as u32;
const SCREEN_HEIGHT: u32 = 144 * SCALE as u32;
const SHOW_HITBOXES: bool = false;
const SHOW_BACKGROUND: bool = true;
const STATUS_FONT_SIZE: u16 = 200;
const STATUS_PERCENTAGE_COLOR: Color = Color::RGBA(255, 255, 195, 255);
const NEUTRAL_COLOR: Color = Color::RGBA(255, 255, 195, 255);
const HOVERED_COLOR: Color = Color::RGBA(255, 155, 95, 255);
const PRESSED_COLOR: Color = Color::RGBA(255, 55, 55, 255);
const TILT_TIME_SIDE: u128 = 186;
const TILT_TIME_UP: u128 = 48;
const CONF_PATH: &str = "./conf/conf";
#[derive(PartialEq)]
enum MenuState {
    Network,
    Character,
    Game,
}
#[derive(PartialEq)]
enum ControlMode {
    Auto,
    Player,
}
fn main_loop() -> Result<(), String> {
    let mut ip: &str = "";

    let mut rng = rand::thread_rng();
    let args: Vec<String> = env::args().collect();
    let player_id = rng.gen();
    let mut player_class = ClassType::Commodore;
    let player_name = "Player".to_string();
    let player_sprite = match player_class {
        ClassType::Commodore => Sprite::Commodore,
        ClassType::Alchemist => Sprite::Alchemist,
    };
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
    let mut camera = Camera {
        x: 0.0,
        y: 0.0,
        dx: 0.0,
        dy: 0.0,
    };
    let mut control_mode = ControlMode::Player;
    if args.len() == 2 {
        if args[1] == "auto" {
            control_mode = ControlMode::Auto;
        }
    }
    let custom_server_ip =
        fs::read_to_string(CONF_PATH).expect("Couldn't find configuration file...");
    let mut menu_state = MenuState::Network;
    let _image_context = image::init(InitFlag::PNG | InitFlag::JPG)?;

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let mut status_font = ttf_context.load_font("fonts/PixelOperator.ttf", STATUS_FONT_SIZE)?;

    let texture_creator = canvas.texture_creator();
    let bg_color = Color::RGB(0, 0, 0);
    let tile_color = Color::RGB(128, 64, 55);
    let floor_color = Color::RGB(64, 32, 30);
    let player_color = Color::RGB(128, 128, 0);
    let mut hit_change = 0.0;
    let mut drop_change = 0;
    let mut drop_time = 300;
    let sprites = HashMap::from([
        (
            Sprite::Commodore,
            texture_creator
                .load_texture("res/commodore/commodore.png")
                .unwrap(),
        ),
        (
            Sprite::CommodoreJab,
            texture_creator
                .load_texture("res/commodore/commodore_jab.png")
                .unwrap(),
        ),
        (
            Sprite::CommodoreNair,
            texture_creator
                .load_texture("res/commodore/commodore_nair.png")
                .unwrap(),
        ),
        (
            Sprite::CommodoreDair,
            texture_creator
                .load_texture("res/commodore/commodore_dair.png")
                .unwrap(),
        ),
        (
            Sprite::CommodoreUair,
            texture_creator
                .load_texture("res/commodore/commodore_uair.png")
                .unwrap(),
        ),
        (
            Sprite::CommodoreSair,
            texture_creator
                .load_texture("res/commodore/commodore_sair.png")
                .unwrap(),
        ),
        (
            Sprite::CommodoreSlide,
            texture_creator
                .load_texture("res/commodore/commodore_slide.png")
                .unwrap(),
        ),
        (
            Sprite::CommodoreSideSmash,
            texture_creator
                .load_texture("res/commodore/commodore_side_smash.png")
                .unwrap(),
        ),
        (
            Sprite::CommodoreUpSmash,
            texture_creator
                .load_texture("res/commodore/commodore_up_smash.png")
                .unwrap(),
        ),
        (
            Sprite::Alchemist,
            texture_creator
                .load_texture("res/alchemist/alchemist.png")
                .unwrap(),
        ),
        (
            Sprite::AlchemistJab,
            texture_creator
                .load_texture("res/alchemist/alchemist_jab.png")
                .unwrap(),
        ),
        (
            Sprite::AlchemistNair,
            texture_creator
                .load_texture("res/alchemist/alchemist_nair.png")
                .unwrap(),
        ),
        (
            Sprite::AlchemistDair,
            texture_creator
                .load_texture("res/alchemist/alchemist_dair.png")
                .unwrap(),
        ),
        (
            Sprite::AlchemistUair,
            texture_creator
                .load_texture("res/alchemist/alchemist_uair.png")
                .unwrap(),
        ),
        (
            Sprite::AlchemistSair,
            texture_creator
                .load_texture("res/alchemist/alchemist_sair.png")
                .unwrap(),
        ),
        (
            Sprite::AlchemistSlide,
            texture_creator
                .load_texture("res/alchemist/alchemist_slide.png")
                .unwrap(),
        ),
        (
            Sprite::AlchemistSideSmash,
            texture_creator
                .load_texture("res/alchemist/alchemist_side_smash.png")
                .unwrap(),
        ),
        (
            Sprite::AlchemistUpSmash,
            texture_creator
                .load_texture("res/alchemist/alchemist_up_smash.png")
                .unwrap(),
        ),
        (
            Sprite::AlchemistFreeze,
            texture_creator
                .load_texture("res/alchemist/alchemist_freeze.png")
                .unwrap(),
        ),
        (
            Sprite::Alchemist2,
            texture_creator
                .load_texture("res/alchemist/alchemist_2.png")
                .unwrap(),
        ),
        (
            Sprite::Commodore2,
            texture_creator
                .load_texture("res/commodore/commodore_2.png")
                .unwrap(),
        ),
        (
            Sprite::CommodoreFreeze,
            texture_creator
                .load_texture("res/commodore/commodore_freeze.png")
                .unwrap(),
        ),
        (
            Sprite::Basement,
            texture_creator
                .load_texture("res/bgs/basement_bg.png")
                .unwrap(),
        ),
        (
            Sprite::Ground,
            texture_creator
                .load_texture("res/environment/ground.png")
                .unwrap(),
        ),
        (
            Sprite::Placeholder,
            texture_creator
                .load_texture("res/commodore/commodore.png")
                .unwrap(),
        ),
        (
            Sprite::LongButtonMain,
            texture_creator
                .load_texture("res/button/long_button_main.png")
                .unwrap(),
        ),
        (
            Sprite::LongButtonHovered,
            texture_creator
                .load_texture("res/button/long_button_hovered.png")
                .unwrap(),
        ),
        (
            Sprite::LongButtonPressed,
            texture_creator
                .load_texture("res/button/long_button_pressed.png")
                .unwrap(),
        ),
        (
            Sprite::Platform,
            texture_creator
                .load_texture("res/environment/platform.png")
                .unwrap(),
        ),
    ]);
    let long_button_sprite = sprites.get(&Sprite::LongButtonMain).unwrap();
    let mut character_buttons = vec![
        Button {
            x: 8,
            y: 8,
            w: long_button_sprite.query().width as i32,
            h: long_button_sprite.query().height as i32,
            main_sprite: Sprite::LongButtonMain,
            hovered_sprite: Sprite::LongButtonHovered,
            pressed_sprite: Sprite::LongButtonPressed,
            action: ButtonAction::Connect,
            hovered: false,
            pressed: false,
            text: "Alchemist".to_string(),
            index: 0,
        },
        Button {
            x: 8,
            y: 44,
            w: long_button_sprite.query().width as i32,
            h: long_button_sprite.query().height as i32,
            main_sprite: Sprite::LongButtonMain,
            hovered_sprite: Sprite::LongButtonHovered,
            pressed_sprite: Sprite::LongButtonPressed,
            action: ButtonAction::Connect,
            hovered: false,
            pressed: false,
            text: "Commodore".to_string(),
            index: 1,
        },
    ];
    let mut network_buttons = vec![
        Button {
            x: 8,
            y: 8,
            w: long_button_sprite.query().width as i32,
            h: long_button_sprite.query().height as i32,
            main_sprite: Sprite::LongButtonMain,
            hovered_sprite: Sprite::LongButtonHovered,
            pressed_sprite: Sprite::LongButtonPressed,
            action: ButtonAction::Connect,
            hovered: false,
            pressed: false,
            text: "Local".to_string(),
            index: 0,
        },
        Button {
            x: 8,
            y: 44,
            w: long_button_sprite.query().width as i32,
            h: long_button_sprite.query().height as i32,
            main_sprite: Sprite::LongButtonMain,
            hovered_sprite: Sprite::LongButtonHovered,
            pressed_sprite: Sprite::LongButtonPressed,
            action: ButtonAction::Connect,
            hovered: false,
            pressed: false,
            text: "Custom Server".to_string(),
            index: 1,
        },
        Button {
            x: 8,
            y: 44 + 36,
            w: long_button_sprite.query().width as i32,
            h: long_button_sprite.query().height as i32,
            main_sprite: Sprite::LongButtonMain,
            hovered_sprite: Sprite::LongButtonHovered,
            pressed_sprite: Sprite::LongButtonPressed,
            action: ButtonAction::Connect,
            hovered: false,
            pressed: false,
            text: "Gut Arena".to_string(),
            index: 2,
        },
    ];
    let mut tilt_change = 0;
    let mut tilt_time = 186;
    let mut tilting = false;
    let mut smashing = false;
    let mut entities: Arc<Mutex<HashMap<u64, Entity>>> = Arc::new(Mutex::new(HashMap::from([(
        player_id,
        Entity {
            x: 48.0,
            y: 0.0,
            h: 0.0,
            w: 0.0,
            dx: 0.0,
            dy: 0.0,
            dir: true,
            hp: 0,
            flying: false,
            next_step: (0.0, 0.0),
            collide_directions: (false, false, false, false),
            current_sprite: get_sprites(player_class.clone(), "1".to_string()),
            freeze_sprite: get_sprites(player_class.clone(), "freeze".to_string()),
            hitboxes: Vec::new(),
            move_lock: false,
            current_action: Action::action(player_class.clone(), ActionType::Idle, 1),
            current_class: player_class.clone(),
            name: player_name.to_string(),
            inv_change: 0.0,
            inv_time: 1000.0,
            jump_counter: 0,
            collide_sides: false,
            drop: false,
            freeze: false,
            stocks: 3,
            walk_time: 250,
            walk_change: 0,
        },
    )])));
    let mut time_from_last_packet: Arc<Mutex<u128>> = Arc::new(Mutex::new(0));
    let mut time_from_last_packet_main: Arc<Mutex<u128>> = time_from_last_packet.clone();
    let mut time_from_last_packet_compare = SystemTime::now();
    let mut network_entities: Arc<Mutex<HashMap<u64, NetworkEntity>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let mut network_entities_thread = network_entities.clone();
    let mut entities_send = entities.clone();

    let mut entities_thread = entities.clone();
    let mut environment: HashMap<u64, Obstacle> = HashMap::from([
        (
            rng.gen(),
            Obstacle {
                x: 24.0,
                y: 70.0,
                h: 0.0,
                w: 0.0,
                current_sprite: Sprite::Ground,
                obstacle_type: ObstacleType::Stage,
            },
        ),
        (
            rng.gen(),
            Obstacle {
                x: 50.0,
                y: 40.0,
                h: 0.0,
                w: 0.0,
                current_sprite: Sprite::Platform,
                obstacle_type: ObstacleType::Platform,
            },
        ),
        (
            rng.gen(),
            Obstacle {
                x: 108.0,
                y: 10.0,
                h: 0.0,
                w: 0.0,
                current_sprite: Sprite::Platform,
                obstacle_type: ObstacleType::Platform,
            },
        ),
        (
            rng.gen(),
            Obstacle {
                x: 170.0,
                y: 40.0,
                h: 0.0,
                w: 0.0,
                current_sprite: Sprite::Platform,
                obstacle_type: ObstacleType::Platform,
            },
        ),
    ]);
    let mut entities_client: HashMap<u64, Entity> = HashMap::new();
    let mut w = false;
    let mut a = false;
    let mut s = false;
    let mut d = false;
    let mut space = false;
    let mut w_released = false;
    let mut a_released = false;
    let mut s_released = false;
    let mut d_released = false;
    let mut j_released = false;
    let mut j = false;
    let mut k = false;
    let mut do_not_move = false;
    let mut smash_change = 0;
    let mut freeze_change = 0;
    let mut freeze_time = 64;
    let mut running = true;
    let mut jump = false;
    let mut event_pump = sdl_context.event_pump()?;
    let mut compare_time = SystemTime::now();
    // socket stuff
    let mut select_index: i32 = 0;
    let mut select_top = network_buttons.len() as i32;
    let mut auto_walk_change = 0;
    let mut auto_walk_time = 500;
    while running {
        let delta = SystemTime::now().duration_since(compare_time).unwrap();
        if control_mode == ControlMode::Auto {
            auto_walk_change += delta.as_millis();
            let player_dir = entities.lock().unwrap().get_mut(&player_id).unwrap().dir;

            if auto_walk_change > auto_walk_time {
                entities.lock().unwrap().get_mut(&player_id).unwrap().dir = !player_dir;

                entities
                    .lock()
                    .unwrap()
                    .get_mut(&player_id)
                    .unwrap()
                    .execute_action(
                        delta.as_millis(),
                        Action::action(player_class.clone(), ActionType::Jab, 100),
                    );
                auto_walk_change = 0;
            }
            if player_dir {
                entities.lock().unwrap().get_mut(&player_id).unwrap().dx = 60.0;
            } else {
                entities.lock().unwrap().get_mut(&player_id).unwrap().dx = -60.0;
            }
        }
        if entities.lock().unwrap().get(&player_id).unwrap().drop {
            drop_change += delta.as_millis();
            if drop_change > drop_time {
                drop_change = 0;
                entities.lock().unwrap().get_mut(&player_id).unwrap().drop = false;
            }
        }
        if entities.lock().unwrap().get(&player_id).unwrap().freeze {
            freeze_change += delta.as_millis();
            if freeze_change > freeze_time {
                entities.lock().unwrap().get_mut(&player_id).unwrap().freeze = false;
                freeze_change = 0;
            }
        }
        canvas.set_draw_color(bg_color);
        canvas.clear();
        hit_change += delta.as_millis() as f32;
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
                    if menu_state == MenuState::Game {
                        if !w
                            && entities
                                .lock()
                                .unwrap()
                                .get_mut(&player_id)
                                .unwrap()
                                .next_step
                                .1
                                == 0.0
                        {
                            tilting = true;

                            do_not_move = false;
                        }
                        if !w {
                            jump = true;
                        }
                    }
                    if !w {
                        select_index -= 1;
                    }
                    w = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    if menu_state == MenuState::Game {
                        if !a
                            && entities
                                .lock()
                                .unwrap()
                                .get_mut(&player_id)
                                .unwrap()
                                .next_step
                                .1
                                == 0.0
                        {
                            tilting = true;
                            do_not_move = false;
                        }
                        entities.lock().unwrap().get_mut(&player_id).unwrap().dir = false;
                    }
                    a = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    if !w {
                        select_index += 1;
                    }
                    s = true;
                    entities.lock().unwrap().get_mut(&player_id).unwrap().drop = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::J),
                    ..
                } => {
                    jump = false;
                    j = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::K),
                    ..
                } => {
                    k = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    if !space {
                        if menu_state == MenuState::Character {
                            if select_index == 0 {
                                player_class = ClassType::Alchemist;
                            } else if select_index == 1 {
                                player_class = ClassType::Commodore;
                            }

                            entities
                                .lock()
                                .unwrap()
                                .get_mut(&player_id)
                                .unwrap()
                                .current_class = player_class.clone();
                            entities
                                .lock()
                                .unwrap()
                                .get_mut(&player_id)
                                .unwrap()
                                .current_sprite =
                                get_sprites(player_class.clone(), "1".to_string());
                            client_threads(
                                player_id,
                                ip.to_string(),
                                time_from_last_packet.clone(),
                                entities_send.clone(),
                                network_entities_thread.clone(),
                            );
                            menu_state = MenuState::Game;
                        }
                        if menu_state == MenuState::Network {
                            if select_index == 0 {
                                if network_buttons[0].action == ButtonAction::Connect {
                                    ip = "localhost:8888";
                                }
                            } else if select_index == 1 {
                                if network_buttons[1].action == ButtonAction::Connect {
                                    ip = &custom_server_ip.trim();
                                }
                            } else if select_index == 2 {
                                if network_buttons[2].action == ButtonAction::Connect {
                                    ip = "165.22.86.59:8888";
                                }
                            }

                            menu_state = MenuState::Character;
                        }
                        select_index = 0;
                    }

                    if !space && !smashing && !do_not_move {
                        jump = true;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    if menu_state == MenuState::Game {
                        if !d
                            && entities
                                .lock()
                                .unwrap()
                                .get_mut(&player_id)
                                .unwrap()
                                .next_step
                                .1
                                == 0.0
                        {
                            tilting = true;
                            do_not_move = false;
                        }
                        entities.lock().unwrap().get_mut(&player_id).unwrap().dir = true;
                    }
                    d = true;
                }

                // WASD
                Event::KeyUp {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    w = false;
                    w_released = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    a = false;
                    a_released = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    s = false;
                    s_released = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    d = false;
                    d_released = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::J),
                    ..
                } => {
                    j = false;
                    j_released = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    space = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::K),
                    ..
                } => {
                    k = false;
                }
                _ => {}
            }
        }
        if select_index > select_top - 1 {
            select_index = select_top - 1;
        }
        match menu_state {
            MenuState::Network => {
                select_top = network_buttons.len() as i32;
                for b in network_buttons.iter_mut() {
                    b.hovered = false;
                }
                if select_index < 0 {
                    select_index = 0;
                }
                for b in network_buttons.iter_mut() {
                    if b.index == select_index {
                        b.hovered = true;
                    }
                    let texture = &sprites.get(&b.main_sprite).unwrap();
                    b.w = texture.query().width as i32;
                    b.h = texture.query().height as i32;
                    /*canvas.copy(
                        texture,
                        Rect::new(0, 0, texture.query().width, texture.query().height),
                        Rect::new(
                            (b.x as f32 * SCALE as f32) as i32,
                            (b.y as f32 * SCALE as f32) as i32 - (2.0 * SCALE) as i32,
                            texture.query().width * SCALE as u32,
                            texture.query().height * SCALE as u32,
                        ),
                    )?;*/

                    let text_color = match b.hovered {
                        true => match b.pressed {
                            true => PRESSED_COLOR,
                            false => HOVERED_COLOR,
                        },
                        false => NEUTRAL_COLOR,
                    };
                    let text = get_text(
                        b.text.clone(),
                        text_color,
                        STATUS_FONT_SIZE,
                        &status_font,
                        &texture_creator,
                    )
                    .unwrap();
                    let position = (
                        ((0.0 + b.x as f32) * SCALE) as i32,
                        ((0.0 + b.y as f32) * SCALE) as i32,
                    );
                    render_text(
                        &mut canvas,
                        &text.text_texture,
                        position,
                        text.text_sprite,
                        SCALE,
                        SCALE,
                    );
                }
            }
            MenuState::Character => {
                select_top = character_buttons.len() as i32;
                for b in character_buttons.iter_mut() {
                    b.hovered = false;
                }
                if select_index < 0 {
                    select_index = 0;
                }

                for b in character_buttons.iter_mut() {
                    if b.index == select_index {
                        b.hovered = true;
                    }
                    let texture = &sprites.get(&b.main_sprite).unwrap();
                    b.w = texture.query().width as i32;
                    b.h = texture.query().height as i32;
                    /*canvas.copy(
                        texture,
                        Rect::new(0, 0, texture.query().width, texture.query().height),
                        Rect::new(
                            (b.x as f32 * SCALE as f32) as i32,
                            (b.y as f32 * SCALE as f32) as i32 - (2.0 * SCALE) as i32,
                            texture.query().width * SCALE as u32,
                            texture.query().height * SCALE as u32,
                        ),
                    )?;*/

                    let text_color = match b.hovered {
                        true => match b.pressed {
                            true => PRESSED_COLOR,
                            false => HOVERED_COLOR,
                        },
                        false => NEUTRAL_COLOR,
                    };
                    let text = get_text(
                        b.text.clone(),
                        text_color,
                        STATUS_FONT_SIZE,
                        &status_font,
                        &texture_creator,
                    )
                    .unwrap();
                    let position = (
                        ((0.0 + b.x as f32) * SCALE) as i32,
                        ((0.0 + b.y as f32) * SCALE) as i32,
                    );
                    render_text(
                        &mut canvas,
                        &text.text_texture,
                        position,
                        text.text_sprite,
                        SCALE,
                        SCALE,
                    );
                }
            }
            MenuState::Game => {
                if tilting && j {
                    smashing = true;
                }
                if smashing {
                    smash_change += delta.as_millis();
                } else {
                    smash_change = 1;
                }
                canvas.set_draw_color(bg_color);
                canvas.clear();

                let player_flying = entities.lock().unwrap().get(&player_id).unwrap().flying;
                if true {
                    if a && !smashing && !do_not_move {
                        let acc_ratio =
                            match player_flying {
                                true => 0.02,
                                false => 0.5,
                            };
                        let dx = entities.lock().unwrap().get_mut(&player_id).unwrap().dx;

                        entities.lock().unwrap().get_mut(&player_id).unwrap().dx =
                            dx.lerp(-60.0, acc_ratio);
                    }
                    if d && !smashing && !do_not_move {
                        let acc_ratio =
                            match player_flying {
                                true => 0.02,
                                false => 0.5,
                            };
                        let dx = entities.lock().unwrap().get_mut(&player_id).unwrap().dx;
                        entities.lock().unwrap().get_mut(&player_id).unwrap().dx =
                            dx.lerp(60.0, acc_ratio);
                    }
                }
                if tilting {
                    tilt_change += delta.as_millis();
                    if tilt_change > tilt_time {
                        tilt_change = 0;
                        tilting = false;
                    }
                }
                let next_step_y = entities
                    .lock()
                    .unwrap()
                    .get_mut(&player_id)
                    .unwrap()
                    .next_step
                    .1;
                if !a && !d && next_step_y == 0.0 || smashing {
                    let slow_ratio = match entities.lock().unwrap().get(&player_id).unwrap().flying
                    {
                        true => 0.87,
                        false => 0.87,
                    };
                    let dx = entities.lock().unwrap().get_mut(&player_id).unwrap().dx;
                    entities.lock().unwrap().get_mut(&player_id).unwrap().dx -=
                        dx.lerp(0.0, slow_ratio);
                }
                if j_released {
                    if (a || d)
                        && hit_change
                            > Action::action(player_class.clone(), ActionType::Slide, smash_change)
                                .hit_time
                        && smashing
                    {
                        let hit_type = ActionType::SideSmash;
                        smashing = false;
                        do_not_move = true;

                        entities
                            .lock()
                            .unwrap()
                            .get_mut(&player_id)
                            .unwrap()
                            .execute_action(
                                delta.as_millis(),
                                Action::action(player_class.clone(), hit_type, smash_change),
                            );
                        hit_change = 0.0;
                        j_released = false;
                    }
                    if w && hit_change
                        > Action::action(player_class.clone(), ActionType::UpSmash, smash_change)
                            .hit_time
                        && smashing
                    {
                        let mut hit_type = ActionType::UpSmash;
                        hit_type = ActionType::UpSmash;
                        smashing = false;
                        do_not_move = true;
                        entities
                            .lock()
                            .unwrap()
                            .get_mut(&player_id)
                            .unwrap()
                            .execute_action(
                                delta.as_millis(),
                                Action::action(
                                    player_class.clone(),
                                    ActionType::UpSmash,
                                    smash_change,
                                ),
                            );
                        hit_change = 0.0;
                        w_released = false;
                        j_released = false;
                    }
                    j_released = false;
                }
                if j && !smashing {
                    if !a
                        && !d
                        && !w
                        && !s
                        && entities
                            .lock()
                            .unwrap()
                            .get_mut(&player_id)
                            .unwrap()
                            .next_step
                            .1
                            == 0.0
                        && hit_change
                            > Action::action(player_class.clone(), ActionType::Jab, 1).hit_time
                    {
                        let mut hit_type = ActionType::Jab;
                        entities
                            .lock()
                            .unwrap()
                            .get_mut(&player_id)
                            .unwrap()
                            .execute_action(
                                delta.as_millis(),
                                Action::action(player_class.clone(), hit_type, 1),
                            );
                        hit_change = 0.0;
                    }
                    if !a
                        && !d
                        && !w
                        && !s
                        && entities
                            .lock()
                            .unwrap()
                            .get_mut(&player_id)
                            .unwrap()
                            .next_step
                            .1
                            != 0.0
                        && hit_change
                            > Action::action(player_class.clone(), ActionType::Nair, 1).hit_time
                    {
                        entities
                            .lock()
                            .unwrap()
                            .get_mut(&player_id)
                            .unwrap()
                            .execute_action(
                                delta.as_millis(),
                                Action::action(player_class.clone(), ActionType::Nair, 1),
                            );
                        hit_change = 0.0;
                    }

                    let mut hit_type = ActionType::Slide;
                    if entities
                        .lock()
                        .unwrap()
                        .get_mut(&player_id)
                        .unwrap()
                        .next_step
                        .1
                        != 0.0
                    {
                        hit_type = ActionType::Sair;
                    }
                    if (a || d)
                        && hit_change
                            > Action::action(player_class.clone(), hit_type.clone(), 1).hit_time
                    {
                        entities
                            .lock()
                            .unwrap()
                            .get_mut(&player_id)
                            .unwrap()
                            .execute_action(
                                delta.as_millis(),
                                Action::action(player_class.clone(), hit_type, 1),
                            );
                        hit_change = 0.0;
                    }
                    if w && hit_change
                        > Action::action(player_class.clone(), ActionType::Uair, 1).hit_time
                    {
                        let mut hit_type = ActionType::Uair;
                        entities
                            .lock()
                            .unwrap()
                            .get_mut(&player_id)
                            .unwrap()
                            .execute_action(
                                delta.as_millis(),
                                Action::action(player_class.clone(), ActionType::Uair, 1),
                            );
                        hit_change = 0.0;
                        w_released = false;
                    }
                    if s && entities
                        .lock()
                        .unwrap()
                        .get_mut(&player_id)
                        .unwrap()
                        .next_step
                        .1
                        > 0.0
                        && hit_change
                            > Action::action(player_class.clone(), ActionType::Dair, 1).hit_time
                    {
                        let mut hit_type = ActionType::Dair;
                        entities
                            .lock()
                            .unwrap()
                            .get_mut(&player_id)
                            .unwrap()
                            .execute_action(
                                delta.as_millis(),
                                Action::action(player_class.clone(), hit_type, 1),
                            );
                        hit_change = 0.0;
                    }
                }
                if !a && !d && !w {
                    smashing = false;
                    do_not_move = false;
                }
                for (id, e) in entities.lock().unwrap().iter_mut() {
                    e.tick(delta.as_millis());
                }

                for (id, e) in network_entities.lock().unwrap().iter_mut() {
                    // e.tick(*time_from_last_packet_main.lock().unwrap());
                }
                let mut entities_network_clone = network_entities.lock().unwrap().clone();

                for (id, e) in entities.lock().unwrap().iter_mut() {
                    for env in environment.values_mut() {
                        e.collide_with_obstacle(delta.as_millis(), env);
                    }
                    for (o_id, o_e) in entities_network_clone.iter() {
                        e.collide_with_hitboxes(delta.as_millis(), o_e);
                    }
                }
                /*for (id, e) in network_entities.lock().unwrap().iter_mut() {
                    for env in environment.values_mut() {
                        e.collide_with(delta.as_millis(), env);
                    }
                }*/
                for (id, e) in entities.lock().unwrap().iter_mut() {
                    e.execute_movement();
                }
                /*for (id, e) in network_entities.lock().unwrap().iter_mut() {
                    e.execute_movement();
                }*/
                // draw bg
                if SHOW_BACKGROUND {
                    let texture = &sprites.get(&Sprite::Basement).unwrap();
                    canvas.copy(
                        texture,
                        Rect::new(0, 0, texture.query().width, texture.query().height),
                        Rect::new(
                            (0.0 * SCALE as f32) as i32,
                            (0.0 * SCALE as f32) as i32,
                            texture.query().width * SCALE as u32,
                            texture.query().height * SCALE as u32,
                        ),
                    )?;
                }
                for (id, e) in entities.lock().unwrap().iter_mut() {
                    if !&sprites.contains_key(&e.current_sprite) {
                        continue;
                    }
                    let texture = &sprites.get(&e.current_sprite).unwrap();
                    e.w = texture.query().width as f32;
                    e.h = texture.query().height as f32;
                }

                for e in environment.values_mut() {
                    let texture = &sprites.get(&e.current_sprite).unwrap();

                    e.w = texture.query().width as f32;
                    e.h = texture.query().height as f32;
                    canvas.copy(
                        texture,
                        Rect::new(0, 0, texture.query().width, texture.query().height),
                        Rect::new(
                            ((-camera.x + e.x) * SCALE as f32) as i32,
                            ((-camera.y + e.y) * SCALE as f32) as i32 - (2.0 * SCALE) as i32,
                            texture.query().width * SCALE as u32,
                            texture.query().height * SCALE as u32,
                        ),
                    )?;
                }
                for (id, e) in entities.lock().unwrap().iter_mut() {
                    let texture = &sprites.get(&e.current_sprite).unwrap();
                    canvas.copy_ex(
                        texture,
                        Rect::new(0, 0, texture.query().width, texture.query().height),
                        Rect::new(
                            ((-camera.x + e.x) * SCALE as f32) as i32,
                            ((-camera.y + e.y) * SCALE as f32) as i32,
                            texture.query().width * SCALE as u32,
                            texture.query().height * SCALE as u32,
                        ),
                        0.0,
                        Point::new((e.x * SCALE) as i32, (e.y * SCALE) as i32),
                        !e.dir,
                        false,
                    )?;
                    if SHOW_HITBOXES {
                        for hitbox in &e.hitboxes {
                            canvas.set_draw_color(Color::RGB(255, 130, 210));
                            canvas.draw_rect(Rect::new(
                                ((-camera.x + hitbox.x) * SCALE) as i32,
                                ((-camera.y + hitbox.y) * SCALE) as i32,
                                (hitbox.w * SCALE) as u32,
                                (hitbox.h * SCALE) as u32,
                            ));
                        }
                    }
                }

                for (id, e) in network_entities.lock().unwrap().iter_mut() {
                    if !&sprites.contains_key(&e.current_sprite) {
                        continue;
                    }

                    let texture = &sprites.get(&e.current_sprite).unwrap();
                    canvas.copy_ex(
                        texture,
                        Rect::new(0, 0, texture.query().width, texture.query().height),
                        Rect::new(
                            ((-camera.x + e.x) * SCALE as f32) as i32,
                            ((-camera.y + e.y) * SCALE as f32) as i32,
                            texture.query().width * SCALE as u32,
                            texture.query().height * SCALE as u32,
                        ),
                        0.0,
                        Point::new((e.x * SCALE) as i32, (e.y * SCALE) as i32),
                        !e.dir,
                        false,
                    )?;
                    if SHOW_HITBOXES {
                        for hitbox in &e.hitboxes {
                            canvas.set_draw_color(Color::RGB(255, 130, 210));
                            canvas.draw_rect(Rect::new(
                                (-camera.x + hitbox.x as f32 * SCALE) as i32,
                                (-camera.y + hitbox.y as f32 * SCALE) as i32,
                                (hitbox.w as f32 * SCALE) as u32,
                                (hitbox.h as f32 * SCALE) as u32,
                            ));
                        }
                    }
                }
                for (i, e) in network_entities.lock().unwrap().iter().enumerate() {
                    let hp_percentage_visual = e.1.hp as f32 / 200.0;
                    let percentage_color = Color::RGBA(
                        STATUS_PERCENTAGE_COLOR.r,
                        (STATUS_PERCENTAGE_COLOR.g as f32).lerp(0.0, hp_percentage_visual) as u8,
                        (STATUS_PERCENTAGE_COLOR.b as f32).lerp(0.0, hp_percentage_visual) as u8,
                        STATUS_PERCENTAGE_COLOR.a,
                    );
                    let name_text = get_text(
                        e.1.name.clone(),
                        STATUS_PERCENTAGE_COLOR,
                        STATUS_FONT_SIZE,
                        &status_font,
                        &texture_creator,
                    )
                    .unwrap();
                    let position = (
                        ((i as f32 * 200.0 + 200.0) * SCALE) as i32,
                        (SCALE * SCREEN_HEIGHT as f32 - 128.0 * SCALE) as i32,
                    );
                    render_text(
                        &mut canvas,
                        &name_text.text_texture,
                        position,
                        name_text.text_sprite,
                        SCALE,
                        SCALE,
                    );
                    let stock_string = match e.1.stocks {
                        0 => "".to_string(),
                        1 => "*".to_string(),
                        2 => "**".to_string(),
                        3 => "***".to_string(),
                        _ => "".to_string(),
                    };
                    let hp_text = get_text(
                        format!("{}%", e.1.hp),
                        percentage_color,
                        STATUS_FONT_SIZE,
                        &status_font,
                        &texture_creator,
                    )
                    .unwrap();
                    let position = (
                        ((i as f32 * 200.0 + 200.0) * SCALE) as i32,
                        (SCALE * SCREEN_HEIGHT as f32 - 85.0 * SCALE) as i32,
                    );
                    render_text(
                        &mut canvas,
                        &hp_text.text_texture,
                        position,
                        hp_text.text_sprite,
                        SCALE,
                        SCALE,
                    );
                    let stock_text = get_text(
                        stock_string,
                        NEUTRAL_COLOR,
                        STATUS_FONT_SIZE,
                        &status_font,
                        &texture_creator,
                    )
                    .unwrap();
                    let position = (
                        ((i as f32 * 200.0 + 200.0) * SCALE) as i32,
                        ((SCREEN_HEIGHT as f32 - 40.0) * SCALE) as i32,
                    );
                    render_text(
                        &mut canvas,
                        &stock_text.text_texture,
                        position,
                        stock_text.text_sprite,
                        SCALE,
                        SCALE,
                    );
                }
                for (i, e) in entities.lock().unwrap().iter().enumerate() {
                    let hp_percentage_visual = e.1.hp as f32 / 200.0;
                    let percentage_color = Color::RGBA(
                        STATUS_PERCENTAGE_COLOR.r,
                        (STATUS_PERCENTAGE_COLOR.g as f32).lerp(0.0, hp_percentage_visual) as u8,
                        (STATUS_PERCENTAGE_COLOR.b as f32).lerp(0.0, hp_percentage_visual) as u8,
                        STATUS_PERCENTAGE_COLOR.a,
                    );
                    let name_text = get_text(
                        e.1.name.clone(),
                        STATUS_PERCENTAGE_COLOR,
                        STATUS_FONT_SIZE,
                        &status_font,
                        &texture_creator,
                    )
                    .unwrap();
                    let position = (
                        ((i as f32 * 200.0 + 14.0) * SCALE) as i32,
                        (SCALE * SCREEN_HEIGHT as f32 - 128.0 * SCALE) as i32,
                    );
                    render_text(
                        &mut canvas,
                        &name_text.text_texture,
                        position,
                        name_text.text_sprite,
                        SCALE,
                        SCALE,
                    );
                    let stock_string = match e.1.stocks {
                        0 => "".to_string(),
                        1 => "*".to_string(),
                        2 => "**".to_string(),
                        3 => "***".to_string(),
                        _ => "".to_string(),
                    };
                    let hp_text = get_text(
                        format!("{}%", e.1.hp),
                        percentage_color,
                        STATUS_FONT_SIZE,
                        &status_font,
                        &texture_creator,
                    )
                    .unwrap();

                    let position = (
                        ((i as f32 * 200.0 + 14.0) * SCALE) as i32,
                        (SCALE * SCREEN_HEIGHT as f32 - 85.0 * SCALE) as i32,
                    );
                    render_text(
                        &mut canvas,
                        &hp_text.text_texture,
                        position,
                        hp_text.text_sprite,
                        SCALE,
                        SCALE,
                    );
                    let stock_text = get_text(
                        stock_string,
                        NEUTRAL_COLOR,
                        STATUS_FONT_SIZE,
                        &status_font,
                        &texture_creator,
                    )
                    .unwrap();
                    let position = (
                        ((i as f32 * 200.0 + 14.0) * SCALE) as i32,
                        ((SCREEN_HEIGHT as f32 - 40.0) * SCALE) as i32,
                    );
                    render_text(
                        &mut canvas,
                        &stock_text.text_texture,
                        position,
                        stock_text.text_sprite,
                        SCALE,
                        SCALE,
                    );
                }
                if w || space {
                    tilt_time = TILT_TIME_UP;
                } else {
                    tilt_time = TILT_TIME_SIDE;
                }
                if jump && !smashing && !do_not_move {
                    entities.lock().unwrap().get_mut(&player_id).unwrap().jump();
                    jump = false;
                }
                let p_x = entities.lock().unwrap().get_mut(&player_id).unwrap().x;
                let p_y = entities.lock().unwrap().get_mut(&player_id).unwrap().y;
                camera.move_towards_point(p_x, p_y);
                camera.tick(delta.as_millis());
            }
        };
        canvas.present();
        compare_time = SystemTime::now();

        thread::sleep(time::Duration::from_millis(10));
    }

    Ok(())
}
pub fn main() {
    main_loop();
}
