use std::collections::VecDeque;
use std::io::{ErrorKind, Result};

use crate::{AsyncMinitelRead, AsyncMinitelWrite};
use axum::extract::ws::WebSocket;

/// A minitel port backed by an axum websocket
pub struct Port {
    ws: WebSocket,
    buffer: VecDeque<u8>,
}

impl Port {
    pub fn new(ws: WebSocket) -> Self {
        Self {
            ws,
            buffer: VecDeque::new(),
        }
    }
}

impl AsyncMinitelWrite for Port {
    async fn write(&mut self, data: &[u8]) -> Result<()> {
        // the minitel websocket only accepts text messages? can be invalid utf8?
        let string = String::from_utf8(data.to_vec())
            .map_err(|_| std::io::Error::new(ErrorKind::InvalidData, "Invalid UTF-8 data"))?;
        self.ws.send(string.into()).await.map_err(axum_map_err)
    }

    async fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl AsyncMinitelRead for Port {
    async fn read(&mut self, data: &mut [u8]) -> Result<()> {
        // The websocket provides data without control of the size
        // store them in a buffer, and deliver as much as requested
        while self.buffer.len() < data.len() {
            let message = self.ws.recv().await.unwrap().unwrap();
            if let axum::extract::ws::Message::Text(data) = message {
                self.buffer.extend(data.as_bytes());
            }
        }
        for byte in data.iter_mut() {
            *byte = self.buffer.pop_front().unwrap();
        }
        Ok(())
    }
}

fn axum_map_err(e: axum::Error) -> std::io::Error {
    std::io::Error::new(ErrorKind::Other, e.to_string())
}
