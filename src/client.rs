use crate::entity::*;
use crate::environment::*;
use crate::graphics::Sprite;
use crate::graphics::*;
use crate::network::*;
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
use sdl2::rect::Rect;
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
    env,
    io::{self, ErrorKind},
    process::exit,
    sync::mpsc,
};
use std::{thread, time};
const SCALE: f32 = 4.0;
const SCREEN_WIDTH: u32 = 256 * SCALE as u32;
const SCREEN_HEIGHT: u32 = 144 * SCALE as u32;
const SHOW_HITBOXES: bool = true;
const MSG_SIZE: usize = 96;
const STATUS_FONT_SIZE: u16 = 200;

fn main_loop() -> Result<(), String> {
    let mut ip: &str = "127.0.0.1:8888";

    let mut rng = rand::thread_rng();
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        println!("Provide arguments (username, character, ip)");
        exit(0);
    }
    let player_id = rng.gen();
    let player_name = &args[1];
    let player_class = match args[2].as_str(){
        "commodore" => ClassType::Commodore,
        "alchemist" => ClassType::Alchemist,
        _ => ClassType::Commodore,

    };
    let player_sprite = match player_class {
        ClassType::Commodore => Sprite::Commodore,
        ClassType::Alchemist => Sprite::Alchemist,
    };
    ip = &args[3];
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

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let mut status_font = ttf_context.load_font("fonts/PixelOperator.ttf", STATUS_FONT_SIZE)?;

    let texture_creator = canvas.texture_creator();
    let bg_color = Color::RGB(255, 255, 255);
    let tile_color = Color::RGB(128, 64, 55);
    let floor_color = Color::RGB(64, 32, 30);
    let player_color = Color::RGB(128, 128, 0);
    let mut hit_change = 0.0;
    let sprites = HashMap::from([

        (
            Sprite::Commodore,
            texture_creator.load_texture("res/commodore.png")?,
        ),
        (
            Sprite::Alchemist,
            texture_creator.load_texture("res/alchemist.png")?,
        ),
        (
            Sprite::Basement,
            texture_creator.load_texture("res/basement_bg.png")?,
        ),
        (
            Sprite::Ground,
            texture_creator.load_texture("res/ground.png")?,
        ),
    ]);
    let mut tilt_change = 0;
    let mut tilt_time = 132;
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
            next_step: (0.0, 0.0),
            collide_directions: (false, false, false, false),
            current_sprite: player_sprite.clone(),
            hitboxes: Vec::new(),
            move_lock: false,
            current_action: Action::action(player_class.clone(), ActionType::Jab, 1),
            name: player_name.to_string(),
            inv_change: 0.0,
            inv_time: 1000.0,
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
    let mut environment: HashMap<u64, Entity> = HashMap::from([(
        rng.gen(),
        Entity {
            x: 24.0,
            y: 80.0,
            h: 0.0,
            w: 0.0,
            dx: 0.0,
            dy: 0.0,
            dir: true,
            hp: 0,
            next_step: (0.0, 0.0),
            collide_directions: (false, false, false, false),
            current_sprite: Sprite::Ground,
            hitboxes: Vec::new(),
            move_lock: false,
            current_action: Action::action(player_class.clone(), ActionType::Jab, 1),
            name: "obstacle".to_string(),
            inv_change: 0.0,
            inv_time: 1000.0,
        },
    )]);
    let mut entities_client: HashMap<u64, Entity> = HashMap::new();
    let mut w = false;
    let mut a = false;
    let mut s = false;
    let mut d = false;
    let mut w_released = false;
    let mut a_released = false;
    let mut s_released = false;
    let mut d_released = false;
    let mut j_released = false;
    let mut j = false;
    let mut k = false;
    let mut do_not_move = false;
    let mut smash_change = 0;

    let mut running = true;
    let mut event_pump = sdl_context.event_pump()?;
    let mut compare_time = SystemTime::now();
    // socket stuff

    let mut client = TcpStream::connect(ip).expect("Connection failed...");
    client.set_nonblocking(true);
    let (tx, rx) = mpsc::channel::<SendState>();
    let (tx_state, rx_state) = mpsc::channel::<SendState>();
    thread::spawn(move || loop {
        *time_from_last_packet.lock().unwrap() = SystemTime::now()
            .duration_since(time_from_last_packet_compare)
            .unwrap()
            .as_millis();

        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                if is_zero(&buff) {
                    println!("Received empty packet");
                    continue;
                }
                let state: Option<SendState> = match bincode::deserialize(&buff) {
                    Ok(s) => Some(s),
                    Err(_) => None,
                };
                if state.is_none() {
                    continue;
                }
                let state_ref = state.as_ref().unwrap();
                if state_ref.player.name.is_empty() {
                    continue;
                }

                if !network_entities_thread
                    .lock()
                    .unwrap()
                    .contains_key(&state_ref.id)
                    && state_ref.id != player_id
                {
                    network_entities_thread
                        .lock()
                        .unwrap()
                        .insert(state_ref.id, state_ref.player.clone());
                } else if state_ref.id != player_id {
                    *network_entities_thread
                        .lock()
                        .unwrap()
                        .get_mut(&state_ref.id)
                        .unwrap() = state_ref.player.clone();
                } else {
                }

                time_from_last_packet_compare = SystemTime::now();
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection failed with server...");
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let mut encoded: Vec<u8> = bincode::serialize(&msg).unwrap();
                // let mut buff = msg.clone().into_bytes();
                encoded.resize(MSG_SIZE, 0);
                client
                    .write_all(&encoded)
                    .expect("writing to socket failed");
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,
        }
        //thread::sleep(::std::time::Duration::from_millis(10));
    });
    thread::spawn(move || loop {
        let msg = SendState {
            id: player_id,
            player: entities_send
                .lock()
                .unwrap()
                .get(&player_id)
                .unwrap()
                .clone()
                .get_as_network_entity(),
        };
        if tx.send(msg).is_err() {
            break;
        }
        thread::sleep(time::Duration::from_millis(32));
    });
    while running {
        let delta = SystemTime::now().duration_since(compare_time).unwrap();
        if delta.as_millis() / 10 != 0 {
            //   println!("FPS: {}", 100 / (delta.as_millis()/10));
        }
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
                    if !w {
                        tilting = true;

                        do_not_move = false;
                    }
                    w = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    if !a {
                        tilting = true;
                        do_not_move = false;
                    }
                    a = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    s = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::J),
                    ..
                } => {
                    j = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::K),
                    ..
                } => {
                    k = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    if !d {
                        tilting = true;
                        do_not_move = false;
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
                    keycode: Some(Keycode::K),
                    ..
                } => {
                    k = false;
                }
                _ => {}
            }
        }

        if tilting && j {
            smashing = true;
        }
        if smashing {
            smash_change += delta.as_millis();
        }
        else {
            smash_change = 1;
        }
        canvas.set_draw_color(bg_color);
        canvas.clear();

        if !entities
            .lock()
            .unwrap()
            .get_mut(&player_id)
            .unwrap()
            .move_lock
        {
            if a && !smashing && !do_not_move {
                let dx = entities.lock().unwrap().get_mut(&player_id).unwrap().dx;

                entities.lock().unwrap().get_mut(&player_id).unwrap().dx -= dx.lerp(60.0, 0.5);
                entities.lock().unwrap().get_mut(&player_id).unwrap().dir = false;
            }
            if d && !smashing && !do_not_move {
                let dx = entities.lock().unwrap().get_mut(&player_id).unwrap().dx;
                entities.lock().unwrap().get_mut(&player_id).unwrap().dx -= dx.lerp(-60.0, 0.5);
                entities.lock().unwrap().get_mut(&player_id).unwrap().dir = true;
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
        if !a && !d && next_step_y == 0.0 || smashing{
            let dx = entities.lock().unwrap().get_mut(&player_id).unwrap().dx;
            entities.lock().unwrap().get_mut(&player_id).unwrap().dx -= dx.lerp(0.0, 0.87);
        }
        if j_released {
            if (a || d) && hit_change > Action::action(player_class.clone(), ActionType::Slide, smash_change).hit_time && smashing{

                    let hit_type = ActionType::SideSmash;
                    smashing = false;
                    do_not_move = true;

                entities
                    .lock()
                    .unwrap()
                    .get_mut(&player_id)
                    .unwrap()
                    .execute_action(delta.as_millis(), Action::action(player_class.clone(), hit_type, smash_change));
                hit_change = 0.0;
                j_released = false;
            }
            if w && hit_change > Action::action(player_class.clone(), ActionType::Uair, smash_change).hit_time && smashing {
                let mut hit_type = ActionType::Uair;
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
                        Action::action(player_class.clone(), ActionType::Uair, smash_change),
                    );
                hit_change = 0.0;
                w_released = false;
                j_released = false;
            }
                j_released = false;
        }
        if j && !smashing{
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
                && hit_change > Action::action(player_class.clone(), ActionType::Jab, 1).hit_time
            {
                let mut hit_type = ActionType::Jab;
                entities
                    .lock()
                    .unwrap()
                    .get_mut(&player_id)
                    .unwrap()
                    .execute_action(delta.as_millis(), Action::action(player_class.clone(), hit_type, 1));
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
                && hit_change > Action::action(player_class.clone(), ActionType::Nair, 1).hit_time
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
            if (a || d) && hit_change > Action::action(player_class.clone(), ActionType::Slide, 1).hit_time {
                let mut hit_type = ActionType::Slide;

                entities
                    .lock()
                    .unwrap()
                    .get_mut(&player_id)
                    .unwrap()
                    .execute_action(delta.as_millis(), Action::action(player_class.clone(), hit_type, 1));
                hit_change = 0.0;
            }
            if w && hit_change > Action::action(player_class.clone(), ActionType::Uair, 1).hit_time {
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
                && hit_change > Action::action(player_class.clone(), ActionType::Dair, 1).hit_time
            {
                let mut hit_type = ActionType::Dair;
                entities
                    .lock()
                    .unwrap()
                    .get_mut(&player_id)
                    .unwrap()
                    .execute_action(delta.as_millis(), Action::action(player_class.clone(), hit_type, 1));
                hit_change = 0.0;
            }

        }
        if !a && !d && !w {
            smashing = false;
            do_not_move = false;
        }
        if w && !smashing && !do_not_move {
            entities.lock().unwrap().get_mut(&player_id).unwrap().jump();
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
                e.collide_with(delta.as_millis(), env);
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
                    (e.x * SCALE as f32) as i32,
                    (e.y * SCALE as f32) as i32 - (2.0 * SCALE) as i32,
                    texture.query().width * SCALE as u32,
                    texture.query().height * SCALE as u32,
                ),
            )?;
        }
        for (id, e) in entities.lock().unwrap().iter_mut() {
            let texture = &sprites.get(&e.current_sprite).unwrap();
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
            if SHOW_HITBOXES {
                for hitbox in &e.hitboxes {
                    canvas.set_draw_color(Color::RGB(255, 130, 210));
                    canvas.draw_rect(Rect::new(
                        (hitbox.x * SCALE) as i32,
                        (hitbox.y * SCALE) as i32,
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
            if SHOW_HITBOXES {
                for hitbox in &e.hitboxes {
                    canvas.set_draw_color(Color::RGB(255, 130, 210));
                    canvas.draw_rect(Rect::new(
                        (hitbox.x as f32 * SCALE) as i32,
                        (hitbox.y as f32 * SCALE) as i32,
                        (hitbox.w as f32 * SCALE) as u32,
                        (hitbox.h as f32 * SCALE) as u32,
                    ));
                }
            }
        }
        for (i, e) in network_entities.lock().unwrap().iter().enumerate() {
            if e.1.name.is_empty() {
                continue;
            }
            let name_text = get_text(
                e.1.name.clone(),
                Color::RGBA(0, 0, 0, 255),
                STATUS_FONT_SIZE,
                &status_font,
                &texture_creator,
            )
            .unwrap();
            let position = (
                (180.0 * SCALE + i as f32 * 308.0 * SCALE) as i32,
                (SCALE * SCREEN_HEIGHT as f32 - 108.0 * SCALE) as i32,
            );
            render_text(
                &mut canvas,
                &name_text.text_texture,
                position,
                name_text.text_sprite,
                SCALE,
                SCALE,
            );
            let hp_text = get_text(
                format!("{}%", e.1.hp),
                Color::RGBA(0, 0, 0, 255),
                STATUS_FONT_SIZE,
                &status_font,
                &texture_creator,
            )
            .unwrap();
            let position = (
                (180.0 * SCALE + i as f32 * 308.0 * SCALE) as i32,
                (SCALE * SCREEN_HEIGHT as f32 - 60.0 * SCALE) as i32,
            );
            render_text(
                &mut canvas,
                &hp_text.text_texture,
                position,
                hp_text.text_sprite,
                SCALE,
                SCALE,
            );
        }
        for (i, e) in entities.lock().unwrap().iter().enumerate() {
            let name_text = get_text(
                e.1.name.clone(),
                Color::RGBA(0, 0, 0, 255),
                STATUS_FONT_SIZE,
                &status_font,
                &texture_creator,
            )
            .unwrap();
            let position = (
                (40.0 + i as f32 * 308.0 * SCALE) as i32,
                (SCALE * SCREEN_HEIGHT as f32 - 108.0 * SCALE) as i32,
            );
            render_text(
                &mut canvas,
                &name_text.text_texture,
                position,
                name_text.text_sprite,
                SCALE,
                SCALE,
            );
            let hp_text = get_text(
                format!("{}%", e.1.hp),
                Color::RGBA(0, 0, 0, 255),
                STATUS_FONT_SIZE,
                &status_font,
                &texture_creator,
            )
            .unwrap();
            let position = (
                (40.0 + i as f32 * 308.0 * SCALE) as i32,
                (SCALE * SCREEN_HEIGHT as f32 - 60.0 * SCALE) as i32,
            );
            render_text(
                &mut canvas,
                &hp_text.text_texture,
                position,
                hp_text.text_sprite,
                SCALE,
                SCALE,
            );
        }
        canvas.present();
        compare_time = SystemTime::now();

        thread::sleep(time::Duration::from_millis(10));
    }

    Ok(())
}
pub fn main() {
    main_loop();
}
