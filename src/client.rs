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
const ENABLE_BOTS: bool = true;
const SHOW_BACKGROUND: bool = true;
const STATUS_FONT_SIZE: u16 = 200;
const STATUS_PERCENTAGE_COLOR: Color = Color::RGBA(255, 255, 195, 255);
const NEUTRAL_COLOR: Color = Color::RGBA(255, 255, 195, 255);
const HOVERED_COLOR: Color = Color::RGBA(255, 155, 95, 255);
const PRESSED_COLOR: Color = Color::RGBA(255, 55, 55, 255);
const CONF_PATH: &str = "./conf/conf";
#[derive(PartialEq)]
enum MenuState {
    Network,
    Level,
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
            Sprite::AlchemistGrab,
            texture_creator
                .load_texture("res/alchemist/alchemist_grab.png")
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
            Sprite::AlchemistStun,
            texture_creator
                .load_texture("res/alchemist/alchemist_stun.png")
                .unwrap(),
        ),
        (
            Sprite::AlchemistShield,
            texture_creator
                .load_texture("res/alchemist/alchemist_shield.png")
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
            Sprite::CommodoreStun,
            texture_creator
                .load_texture("res/commodore/commodore_stun.png")
                .unwrap(),
        ),
        (
            Sprite::CommodoreShield,
            texture_creator
                .load_texture("res/commodore/commodore_shield.png")
                .unwrap(),
        ),
        (
            Sprite::CommodoreDodge,
            texture_creator
                .load_texture("res/commodore/commodore_dodge.png")
                .unwrap(),
        ),
        (
            Sprite::CommodoreGrab,
            texture_creator
                .load_texture("res/commodore/commodore_grab.png")
                .unwrap(),
        ),
        (
            Sprite::AlchemistDodge,
            texture_creator
                .load_texture("res/alchemist/alchemist_dodge.png")
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
        (
            Sprite::Shield,
            texture_creator.load_texture("res/misc/shield.png").unwrap(),
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
    let mut level_buttons = vec![
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
            text: "Arena".to_string(),
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
            text: "Campaign".to_string(),
            index: 1,
        },
    ];
    let mut tilt_change = 0;
    let mut tilt_time = 186;
    let mut entities: Arc<Mutex<HashMap<u64, Entity>>> = Arc::new(Mutex::new(HashMap::new()));
    entities.lock().unwrap().insert(
        player_id,
        Entity::new(
            98.0,
            0.0,
            player_class.clone(),
            "Player".to_string(),
            true,
            4.0,
            0.0,
            8.0,
            12.0,
        ),
    );
    let mut time_from_last_packet: Arc<Mutex<u128>> = Arc::new(Mutex::new(0));
    let mut time_from_last_packet_main: Arc<Mutex<u128>> = time_from_last_packet.clone();
    let mut time_from_last_packet_compare = SystemTime::now();
    let mut network_entities: Arc<Mutex<HashMap<u64, NetworkEntity>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let mut network_entities_thread = network_entities.clone();
    let mut entities_send = entities.clone();

    let mut entities_thread = entities.clone();
    let mut environment: HashMap<u64, Obstacle> = get_environment();
    let mut entities_client: HashMap<u64, Entity> = HashMap::new();
    let mut space = false;
    let mut do_not_move = false;
    let mut smash_change = 0;
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
        let player_entity = entities
            .lock()
            .unwrap()
            .get_mut(&player_id)
            .unwrap()
            .clone();
        let delta = SystemTime::now().duration_since(compare_time).unwrap();
        if control_mode == ControlMode::Auto {
            auto_walk_change += delta.as_millis();
            let player_dir = player_entity.dir;

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
        canvas.set_draw_color(bg_color);
        canvas.clear();
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
                        if !player_entity.up
                            && entities
                                .lock()
                                .unwrap()
                                .get_mut(&player_id)
                                .unwrap()
                                .next_step
                                .1
                                == 0.0
                        {
                            entities
                                .lock()
                                .unwrap()
                                .get_mut(&player_id)
                                .unwrap()
                                .tilting = true;

                            do_not_move = false;
                        }
                        if !player_entity.up {
                            jump = true;
                        }
                    }
                    if !player_entity.up {
                        select_index -= 1;
                    }
                    entities.lock().unwrap().get_mut(&player_id).unwrap().up = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    if menu_state == MenuState::Game {
                        if !player_entity.left
                            && entities
                                .lock()
                                .unwrap()
                                .get_mut(&player_id)
                                .unwrap()
                                .next_step
                                .1
                                == 0.0
                        {
                            entities
                                .lock()
                                .unwrap()
                                .get_mut(&player_id)
                                .unwrap()
                                .tilting = true;
                            do_not_move = false;
                        }
                        entities.lock().unwrap().get_mut(&player_id).unwrap().dir = false;
                    }

                    if !player_entity.left {
                        entities
                            .lock()
                            .unwrap()
                            .get_mut(&player_id)
                            .unwrap()
                            .grab_counter += 1;
                    }
                    entities.lock().unwrap().get_mut(&player_id).unwrap().left = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    if !player_entity.up {
                        select_index += 1;
                    }
                    entities.lock().unwrap().get_mut(&player_id).unwrap().down = true;
                    entities.lock().unwrap().get_mut(&player_id).unwrap().drop = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::J),
                    ..
                } => {
                    jump = false;
                    entities.lock().unwrap().get_mut(&player_id).unwrap().hit = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::K),
                    ..
                } => {
                    entities
                        .lock()
                        .unwrap()
                        .get_mut(&player_id)
                        .unwrap()
                        .special = true;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::I),
                    ..
                } => {
                    if !player_entity.shield && !player_entity.stunned {
                        entities
                            .lock()
                            .unwrap()
                            .get_mut(&player_id)
                            .unwrap()
                            .start_shield();
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::U),
                    ..
                } => {
                    entities.lock().unwrap().get_mut(&player_id).unwrap().grab = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    if !space {
                        match menu_state {
                            MenuState::Character => {
                                if select_index == 0 {
                                    player_class = ClassType::Alchemist;
                                } else if select_index == 1 {
                                    player_class = ClassType::Commodore;
                                }
                                entities.lock().unwrap().clear();
                                entities.lock().unwrap().insert(
                                    player_id,
                                    Entity::new(
                                        48.0,
                                        0.0,
                                        player_class.clone(),
                                        "Player".to_string(),
                                        false,
                                        7.0,
                                        8.0,
                                        8.0,
                                        12.0,
                                    ),
                                );

                                if ENABLE_BOTS {
                                    entities.lock().unwrap().insert(
                                        rng.gen(),
                                        Entity::new(
                                            98.0,
                                            0.0,
                                            player_class.clone(),
                                            "Bot".to_string(),
                                            true,
                                            4.0,
                                            0.0,
                                            8.0,
                                            12.0,
                                        ),
                                    );
                                }
                                client_threads(
                                    ip.to_string(),
                                    time_from_last_packet.clone(),
                                    entities_send.clone(),
                                    network_entities_thread.clone(),
                                );
                                menu_state = MenuState::Game;
                            }
                            MenuState::Network => {
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

                                menu_state = MenuState::Level;
                            }
                            MenuState::Level => {
                                if select_index == 0 {
                                } else if select_index == 1 {
                                } else if select_index == 2 {
                                }

                                menu_state = MenuState::Character;
                            }
                            MenuState::Game => {}
                        }
                        select_index = 0;

                        if !space && !player_entity.smashing && !player_entity.do_not_move {
                            jump = true;
                        }
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    if menu_state == MenuState::Game {
                        if !player_entity.right
                            && entities
                                .lock()
                                .unwrap()
                                .get_mut(&player_id)
                                .unwrap()
                                .next_step
                                .1
                                == 0.0
                        {
                            entities
                                .lock()
                                .unwrap()
                                .get_mut(&player_id)
                                .unwrap()
                                .tilting = true;
                            do_not_move = false;
                        }
                        entities.lock().unwrap().get_mut(&player_id).unwrap().dir = true;
                    }
                    if !player_entity.right {
                        entities
                            .lock()
                            .unwrap()
                            .get_mut(&player_id)
                            .unwrap()
                            .grab_counter += 1;
                    }
                    entities.lock().unwrap().get_mut(&player_id).unwrap().right = true;
                }

                Event::KeyUp {
                    keycode: Some(Keycode::U),
                    ..
                } => {
                    entities.lock().unwrap().get_mut(&player_id).unwrap().grab = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::I),
                    ..
                } => {
                    entities
                        .lock()
                        .unwrap()
                        .get_mut(&player_id)
                        .unwrap()
                        .release_shield();
                }
                Event::KeyUp {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    entities.lock().unwrap().get_mut(&player_id).unwrap().up = false;
                    entities
                        .lock()
                        .unwrap()
                        .get_mut(&player_id)
                        .unwrap()
                        .up_released = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    entities.lock().unwrap().get_mut(&player_id).unwrap().left = false;
                    entities
                        .lock()
                        .unwrap()
                        .get_mut(&player_id)
                        .unwrap()
                        .left_released = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    entities.lock().unwrap().get_mut(&player_id).unwrap().down = false;
                    entities
                        .lock()
                        .unwrap()
                        .get_mut(&player_id)
                        .unwrap()
                        .down_released = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    entities.lock().unwrap().get_mut(&player_id).unwrap().right = false;
                    entities
                        .lock()
                        .unwrap()
                        .get_mut(&player_id)
                        .unwrap()
                        .right_released = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::J),
                    ..
                } => {
                    entities.lock().unwrap().get_mut(&player_id).unwrap().hit = false;
                    entities
                        .lock()
                        .unwrap()
                        .get_mut(&player_id)
                        .unwrap()
                        .hit_released = true;
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
                    entities
                        .lock()
                        .unwrap()
                        .get_mut(&player_id)
                        .unwrap()
                        .special = false;
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
            MenuState::Level => {
                select_top = level_buttons.len() as i32;
                for b in level_buttons.iter_mut() {
                    b.hovered = false;
                }
                if select_index < 0 {
                    select_index = 0;
                }
                for b in level_buttons.iter_mut() {
                    if b.index == select_index {
                        b.hovered = true;
                    }
                    let texture = &sprites.get(&b.main_sprite).unwrap();
                    b.w = texture.query().width as i32;
                    b.h = texture.query().height as i32;

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
                canvas.set_draw_color(bg_color);
                canvas.clear();

                for (id, e) in entities.lock().unwrap().iter_mut() {
                    if id != &player_id {
                        e.target_entity = Some(Box::new(player_entity.clone()));
                    }
                    e.tick(delta.as_millis());
                }

                let mut entities_network_clone = network_entities.lock().unwrap().clone();
                let entities_clone = entities.lock().unwrap().clone();
                for (id, e) in entities.lock().unwrap().iter_mut() {
                    for env in environment.values_mut() {
                        e.collide_with_obstacle(delta.as_millis(), env);
                    }
                    for (o_id, o_e) in entities_network_clone.iter() {
                        e.collide_with_hitboxes(delta.as_millis(), o_e);
                    }
                    for (o_id, o_e) in entities_clone.iter() {
                        if id == o_id {
                            continue;
                        }
                        e.collide_with_hitboxes(delta.as_millis(), &o_e.get_as_network_entity());
                    }
                }
                for (id, e) in entities.lock().unwrap().iter_mut() {
                    e.execute_movement();
                }
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
                    if e.shield {
                        let w = 20.0
                            - (e.shield_change as f32 / e.shield_time as f32
                                * texture.query().width as f32)
                                as f32;
                        let h = 20.0
                            - (e.shield_change as f32 / e.shield_time as f32
                                * texture.query().height as f32)
                                as f32;
                        canvas.copy_ex(
                            sprites.get(&Sprite::Shield).unwrap(),
                            Rect::new(0, 0, texture.query().width, texture.query().height),
                            Rect::new(
                                ((-camera.x + e.x - w as f32 / 2.0 + 20 as f32 / 2.0)
                                    * SCALE as f32) as i32,
                                ((-camera.y + e.y - h as f32 / 2.0 + 20 as f32 / 2.0)
                                    * SCALE as f32) as i32,
                                (w * SCALE) as u32,
                                (h * SCALE) as u32,
                            ),
                            0.0,
                            Point::new((e.x * SCALE) as i32, (e.y * SCALE) as i32),
                            !e.dir,
                            false,
                        )?;
                    }
                }

                if SHOW_HITBOXES {
                    canvas.set_draw_color(Color::RGB(255, 130, 210));
                    canvas.draw_rect(Rect::new(
                        ((-camera.x + player_entity.x + player_entity.h_x) as f32 * SCALE) as i32,
                        ((-camera.y + player_entity.y + player_entity.h_y) as f32 * SCALE) as i32,
                        (player_entity.h_w as f32 * SCALE) as u32,
                        (player_entity.h_h as f32 * SCALE) as u32,
                    ));
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
                    if e.shield_percentage > 0.0 {
                        let w = 20.0
                            - (e.shield_percentage as f32 * texture.query().width as f32) as f32;
                        let h = 20.0
                            - (e.shield_percentage as f32 * texture.query().height as f32) as f32;
                        canvas.copy_ex(
                            sprites.get(&Sprite::Shield).unwrap(),
                            Rect::new(0, 0, texture.query().width, texture.query().height),
                            Rect::new(
                                ((-camera.x + e.x - w as f32 / 2.0 + 20 as f32 / 2.0)
                                    * SCALE as f32) as i32,
                                ((-camera.y + e.y - h as f32 / 2.0 + 20 as f32 / 2.0)
                                    * SCALE as f32) as i32,
                                (w * SCALE) as u32,
                                (h * SCALE) as u32,
                            ),
                            0.0,
                            Point::new((e.x * SCALE) as i32, (e.y * SCALE) as i32),
                            !e.dir,
                            false,
                        )?;
                    }
                }
                let entities_len = entities.lock().unwrap().len();
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
                        ((i as f32 * 200.0 + 200.0 * entities_len as f32) * SCALE) as i32,
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
                        ((i as f32 * 200.0 + 200.0 * entities_len as f32) * SCALE) as i32,
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
                        ((i as f32 * 200.0 + 200.0 * entities_len as f32) * SCALE) as i32,
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
                if jump && !player_entity.smashing && !do_not_move {
                    entities.lock().unwrap().get_mut(&player_id).unwrap().jump();
                    jump = false;
                }
                let p_x = player_entity.x;
                let p_y = player_entity.y;
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
