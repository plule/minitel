//! A simple echo server.
//!
//! You can test this out by running:
//!
//!     cargo run --example echo-server 127.0.0.1:12345
//!
//! And then in another window run:
//!
//!     cargo run --example client ws://127.0.0.1:12345/
//!
//! Type a message into the client window, press enter to send it and
//! see it echoed back.

use log::info;
use minitel::SerialMinitel;

use std::{
    net::{TcpListener, TcpStream},
    thread::spawn,
};

use log::*;
use tungstenite::{accept, handshake::HandshakeRole, Error, HandshakeError, Message, Result};

fn must_not_block<Role: HandshakeRole>(err: HandshakeError<Role>) -> Error {
    match err {
        HandshakeError::Interrupted(_) => panic!("Bug: blocking socket would block"),
        HandshakeError::Failure(f) => f,
    }
}

fn handle_client(stream: TcpStream) -> Result<()> {
    info!("Running test");
    let socket = accept(stream).map_err(must_not_block)?;
    let mut minitel = minitel::WebSocketMinitel::new(socket);
    minitel.writeln("Bonjour, monde êîôûàèù").unwrap();
    loop {
        minitel.read_byte().unwrap();
        minitel.writeln("Hello, world êîôûàèù").unwrap();
    }
}

fn main() {
    env_logger::init();

    let server = TcpListener::bind("0.0.0.0:3615").unwrap();

    for stream in server.incoming() {
        spawn(move || match stream {
            Ok(stream) => {
                if let Err(err) = handle_client(stream) {
                    match err {
                        Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
                        e => error!("testéé: {}", e),
                    }
                }
            }
            Err(e) => error!("Error accepting stream: {}", e),
        });
    }
}
