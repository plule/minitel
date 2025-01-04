#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

/// Core Minitel types and traits
///
/// The stum crate (Spécifications Techniques d'Utilisation du Minitel) expooses parts of the STUM1B specification.
pub mod stum {
    pub use minitel_stum::*;
}

#[doc(inline)]
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
    #[doc(inline)]
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

#[cfg(feature = "ratatui")]
#[doc(inline)]
pub use minitel_ratatui::MinitelBackend;

#[cfg(feature = "ws")]
#[doc(inline)]
pub use minitel_ws::WSMinitel;

#[cfg(feature = "ws")]
#[doc(inline)]
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

#[cfg(feature = "esp")]
#[doc(inline)]
pub use esp::ESPMinitel;

#[cfg(feature = "esp")]
#[doc(inline)]
pub use esp::esp_minitel;

#[cfg(feature = "esp")]
#[doc(inline)]
pub use esp::esp_minitel_uart2;

/// Minitel terminal running on a ESP32 for ratatui applications
#[cfg(all(feature = "esp", feature = "ratatui"))]
pub type ESPTerminal<'a> = ::ratatui::Terminal<ratatui::MinitelBackend<esp::ESPPort<'a>>>;

/// Build a Minitel terminal on an ESP32 for ratatui applications
#[cfg(all(feature = "esp", feature = "ratatui"))]
pub fn esp_terminal<'a>(minitel: esp::ESPMinitel<'a>) -> std::io::Result<ESPTerminal<'a>> {
    ESPTerminal::new(ratatui::MinitelBackend::new(minitel))
}
