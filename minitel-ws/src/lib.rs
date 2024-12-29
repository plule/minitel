pub use minitel_stum::Minitel;

use std::{collections::VecDeque, io::Error, net::TcpStream};

use minitel_stum::SerialPort;
use std::io::{ErrorKind, Result};
use tungstenite::{Utf8Bytes, WebSocket};

pub type WSMinitel = Minitel<WSPort<TcpStream>>;

fn map_err(e: tungstenite::Error) -> std::io::Error {
    match e {
        tungstenite::Error::ConnectionClosed => ErrorKind::ConnectionReset.into(),
        tungstenite::Error::AlreadyClosed => ErrorKind::NotConnected.into(),
        tungstenite::Error::Io(error) => error,
        tungstenite::Error::Tls(tls_error) => Error::new(ErrorKind::Other, tls_error),
        tungstenite::Error::Capacity(capacity_error) => {
            Error::new(ErrorKind::InvalidData, capacity_error)
        }
        tungstenite::Error::Protocol(protocol_error) => {
            Error::new(ErrorKind::InvalidData, protocol_error)
        }
        tungstenite::Error::WriteBufferFull(_) => Error::new(ErrorKind::Other, "Write buffer full"),
        tungstenite::Error::Utf8 => Error::new(ErrorKind::InvalidData, "Invalid UTF-8 data"),
        tungstenite::Error::AttackAttempt => Error::new(ErrorKind::Other, "Attack attempt"),
        tungstenite::Error::Url(url_error) => Error::new(ErrorKind::InvalidData, url_error),
        tungstenite::Error::Http(_) => Error::new(ErrorKind::InvalidData, "HTTP error"),
        tungstenite::Error::HttpFormat(error) => Error::new(ErrorKind::InvalidData, error),
    }
}

pub fn ws_minitel(socket: WebSocket<TcpStream>) -> WSMinitel {
    WSMinitel::new(WSPort::new(socket))
}

pub struct WSPort<Stream: std::io::Read + std::io::Write> {
    ws: WebSocket<Stream>,
    buffer: VecDeque<u8>,
}

impl<Stream: std::io::Read + std::io::Write> WSPort<Stream> {
    pub fn new(ws: WebSocket<Stream>) -> Self {
        Self {
            ws,
            buffer: VecDeque::new(),
        }
    }
}

impl<Stream: std::io::Read + std::io::Write> From<WebSocket<Stream>> for WSPort<Stream> {
    fn from(ws: WebSocket<Stream>) -> Self {
        Self::new(ws)
    }
}

impl<Stream: std::io::Read + std::io::Write> SerialPort for WSPort<Stream> {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        // the minitel websocket only accepts text messages? can be invalid utf8?
        let text = Utf8Bytes::try_from(data.to_vec())
            .map_err(|_| std::io::Error::new(ErrorKind::InvalidData, "Invalid UTF-8 data"))?;
        self.ws
            .send(tungstenite::Message::text(text))
            .map_err(map_err)
    }

    fn read(&mut self, data: &mut [u8]) -> Result<()> {
        // The websocket provides data without control of the size
        // store them in a buffer, and deliver as much as requested
        while self.buffer.len() < data.len() {
            let message = self.ws.read().map_err(map_err)?;
            if let tungstenite::Message::Text(data) = message {
                self.buffer.extend(data.as_bytes());
            }
        }
        for byte in data.iter_mut() {
            *byte = self.buffer.pop_front().unwrap();
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.ws.flush().map_err(map_err)?;
        self.buffer.clear();
        Ok(())
    }
}
