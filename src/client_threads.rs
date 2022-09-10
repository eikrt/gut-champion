use crate::network::*;
use crate::entity::*;
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
    env,
    io::{self, ErrorKind},
    process::exit,
    sync::mpsc,
    thread,
    time,
    fs,
};

const MSG_SIZE: usize = 96;
pub fn client_threads(
    player_id: u64,
    ip: String,
    time_from_last_packet: Arc<Mutex<u128>>,
    entities_send: Arc<Mutex<HashMap<u64, Entity>>>,
    network_entities_thread: Arc<Mutex<HashMap<u64, NetworkEntity>>>,
) {
    let mut time_from_last_packet_compare = SystemTime::now();
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
}
