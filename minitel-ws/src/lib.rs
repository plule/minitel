pub use minitel_stum::Minitel;

use std::{collections::VecDeque, net::TcpStream};

use minitel_stum::SerialPort;
use tungstenite::{Utf8Bytes, WebSocket};

pub type WSMinitel = Minitel<WSPort<TcpStream>>;

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
    type Error = tungstenite::Error;

    fn send(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        // the minitel websocket only accepts text messages? can be invalid utf8?
        let text = Utf8Bytes::try_from(data.to_vec())?;
        self.ws.send(tungstenite::Message::text(text))
    }

    fn read(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        // The websocket provides data without control of the size
        // store them in a buffer, and deliver as much as requested
        while self.buffer.len() < data.len() {
            let message = self.ws.read()?;
            if let tungstenite::Message::Text(data) = message {
                self.buffer.extend(data.as_bytes());
            }
        }
        for byte in data.iter_mut() {
            *byte = self.buffer.pop_front().unwrap();
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.ws.flush()?;
        self.buffer.clear();
        Ok(())
    }
}
