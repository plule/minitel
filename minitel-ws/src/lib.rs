pub use minitel_stum::SerialMinitel;

use std::collections::VecDeque;

use log::info;
use minitel_stum::MinitelError;
use tungstenite::Utf8Bytes;

pub struct WebSocketMinitel<Stream: std::io::Read + std::io::Write> {
    ws: tungstenite::WebSocket<Stream>,
    buffer: VecDeque<u8>,
}

impl<Stream: std::io::Read + std::io::Write> WebSocketMinitel<Stream> {
    pub fn new(ws: tungstenite::WebSocket<Stream>) -> Self {
        Self {
            ws,
            buffer: VecDeque::new(),
        }
    }
}

impl<Stream: std::io::Read + std::io::Write> SerialMinitel for WebSocketMinitel<Stream> {
    fn send(&mut self, data: &[u8]) -> Result<(), MinitelError> {
        self.ws
            .send(tungstenite::Message::text(
                Utf8Bytes::try_from(data.to_vec()).map_err(|_| MinitelError::FormattingError)?,
            ))
            .map_err(|e| MinitelError::IOError(e.to_string()))
    }

    fn read(&mut self, data: &mut [u8]) -> Result<(), MinitelError> {
        while self.buffer.len() < data.len() {
            let message = self
                .ws
                .read()
                .map_err(|e| MinitelError::IOError(e.to_string()))?;
            if let tungstenite::Message::Text(data) = message {
                info!("Received message: {:?}", data);
                self.buffer.extend(data.as_bytes());
            }
        }
        for byte in data.iter_mut() {
            *byte = self.buffer.pop_front().unwrap();
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), MinitelError> {
        self.ws
            .flush()
            .map_err(|e| MinitelError::IOError(e.to_string()))?;
        self.buffer.clear();
        Ok(())
    }
}
