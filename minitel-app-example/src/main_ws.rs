use log::*;

use std::{
    net::{TcpListener, TcpStream},
    thread::spawn,
};

use tungstenite::{accept, handshake::HandshakeRole, Error, HandshakeError, Result};

fn handle_client(stream: TcpStream) -> Result<()> {
    info!("Running test");
    let socket = accept(stream).map_err(must_not_block)?;
    let minitel = minitel::ws_minitel(socket);
    let mut terminal = minitel::ws_terminal(minitel)?;

    let mut app = crate::app::App::default();
    app.run(&mut terminal)?;
    Ok(())
}

pub fn main() {
    env_logger::init();

    let server = TcpListener::bind("0.0.0.0:3615").unwrap();
    log::info!("Listening");

    for stream in server.incoming() {
        log::info!("New client");
        spawn(move || match stream {
            Ok(stream) => {
                if let Err(err) = handle_client(stream) {
                    match err {
                        Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
                        e => error!("{}", e),
                    }
                }
            }
            Err(e) => error!("Error accepting stream: {}", e),
        });
    }
}

fn must_not_block<Role: HandshakeRole>(err: HandshakeError<Role>) -> Error {
    match err {
        HandshakeError::Interrupted(_) => panic!("Bug: blocking socket would block"),
        HandshakeError::Failure(f) => f,
    }
}
