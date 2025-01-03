#![doc = include_str!("../README.md")]

/// Core Minitel types and traits
///
/// The stum crate (Spécifications Techniques d'Utilisation du Minitel) expooses parts of the STUM1B specification.
pub mod stum {
    pub use minitel_stum::*;
}

/// Minitel interface, the entry point to the library
///
/// This struct wraps a serial port (websocket, physical, file, ...) and provides
/// methods to interact with the device.
///
/// This struct can be initialized using the `ws_minitel`, `esp_minitel` or `esp_minitel_uart2`
/// functions, depending on the target platform and enabled features. It can also operate on any
/// std::io::Read and/or std::io::Write object.
pub use stum::Minitel;

/// Websocket integration
///
/// Implements the necessary traits to use a Minitel terminal over a websocket connection.
#[cfg(feature = "ws")]
pub mod ws {
    pub use minitel_ws::*;
}

/// ESP32 integration
///
/// Implements the necessary traits to use a Minitel terminal over an ESP32 microcontroller.
#[cfg(feature = "esp")]
pub mod esp {
    pub use minitel_esp::*;
}

/// Ratatui integration
///
/// Exposes a backend for ratatui, a terminal UI library. This helps writing interactive
/// applications for the Minitel.
#[cfg(feature = "ratatui")]
pub mod ratatui {
    pub use minitel_ratatui::*;
}

/// Ratatui minitel backend
#[cfg(feature = "ratatui")]
pub use minitel_ratatui::MinitelBackend;

/// Minitel terminal used over a websocket connection
#[cfg(feature = "ws")]
pub use minitel_ws::WSMinitel;

/// Build a Minitel terminal over a websocket connection
#[cfg(feature = "ws")]
pub use minitel_ws::ws_minitel;

/// Websocket Minitel terminal used in ratatui applications
#[cfg(all(feature = "ws", feature = "ratatui"))]
pub type WSTerminal =
    ::ratatui::Terminal<ratatui::MinitelBackend<minitel_ws::WSPort<std::net::TcpStream>>>;

/// Build a Minitel terminal over a websocket connection for ratatui applications
#[cfg(all(feature = "ws", feature = "ratatui"))]
pub fn ws_terminal(minitel: minitel_ws::WSMinitel) -> std::io::Result<WSTerminal> {
    WSTerminal::new(ratatui::MinitelBackend::new(minitel))
}

/// Minitel terminal running on a ESP32
#[cfg(feature = "esp")]
pub use minitel_esp::ESPMinitel;

/// Build a Minitel terminal on an ESP32
#[cfg(feature = "esp")]
pub use minitel_esp::esp_minitel;

/// Minitel terminal running on a ESP32, using UART2 (the default in the ESP32 minitel dev board from iodeo)
#[cfg(feature = "esp")]
pub use minitel_esp::esp_minitel_uart2;

/// Minitel terminal running on a ESP32 for ratatui applications
#[cfg(all(feature = "esp", feature = "ratatui"))]
pub type ESPTerminal<'a> = ::ratatui::Terminal<ratatui::MinitelBackend<minitel_esp::ESPPort<'a>>>;

/// Build a Minitel terminal on an ESP32 for ratatui applications
#[cfg(all(feature = "esp", feature = "ratatui"))]
pub fn esp_terminal<'a>(minitel: minitel_esp::ESPMinitel<'a>) -> std::io::Result<ESPTerminal<'a>> {
    ESPTerminal::new(ratatui::MinitelBackend::new(minitel))
}
