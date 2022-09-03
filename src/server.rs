use bincode;
use std::{
    io::{ErrorKind, Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};
use crate::network::is_zero;

const IP: &str = "0.0.0.0:8888";
const MSG_SIZE: usize = 96;

pub fn main() {
    let server = TcpListener::bind(IP).expect("Failed to bind...");
    server
        .set_nonblocking(true)
        .expect("Initialization failed...");

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);

            let tx = tx.clone();
            clients.push(socket.try_clone().expect("Failed cloning client..."));

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];

                match socket.read(&mut buff) {
                    Ok(_) => {
                        // let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        // println!("{:?}", buff);
                        //
                        //
                        if is_zero(&buff) {
                            println!("Closing connection with: {}", addr);
                            break;
                        }
                        tx.send(buff).expect("Failed to send message to channel...");
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                        break;
                    }
                    Err(_) => {
                        println!("Closing connection with: {}", addr);
                        break;
                    }
                }
                thread::sleep(::std::time::Duration::from_millis(10));
            });
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                    let mut buff = msg.clone();
                    buff.resize(MSG_SIZE, 0);
                    client.write_all(&buff).map(|_| client).ok()
                })
                .collect::<Vec<_>>();
        }

    }
}
