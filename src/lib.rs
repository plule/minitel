#![doc = include_str!("../README.md")]

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
#[cfg(any(feature = "esp", doc))]
pub mod esp {
    #[cfg(feature = "esp")]
    pub use minitel_esp::*;

    #[cfg(not(feature = "esp"))]
    pub use crate::minitel_esp::*;
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

#[cfg(any(feature = "esp", doc))]
#[doc(inline)]
pub use esp::ESPMinitel;

#[cfg(any(feature = "esp", doc))]
#[doc(inline)]
pub use esp::esp_minitel;

#[cfg(any(feature = "esp", doc))]
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

// Below: doc shenanigans when ESP toolchain is not available

/// Minitel terminal running on a ESP32 for ratatui applications
///
/// Generate the documentation locally with the ESP toolchain to see the actual content
#[cfg(all(not(feature = "esp"), feature = "ratatui", doc))]
pub type ESPTerminal = esp::ESPTerminal;

/// Build a Minitel terminal on an ESP32 for ratatui applications
///
/// Generate the documentation locally with the ESP toolchain to see the actual content
#[cfg(all(not(feature = "esp"), feature = "ratatui", doc))]
pub fn esp_terminal() -> std::io::Result<ESPTerminal> {
    unimplemented!()
}

/// ESP32 integration
///
/// Generate the documentation locally with the ESP toolchain to see the actual content
#[cfg(all(not(feature = "esp"), doc))]
pub mod minitel_esp {
    /// Minitel terminal running on a ESP32
    ///
    /// Generate the documentation locally with the ESP toolchain to see the actual content
    pub struct ESPMinitel;

    /// Minitel terminal running on a ESP32 for ratatui applications
    ///
    /// Generate the documentation locally with the ESP toolchain to see the actual content
    pub struct ESPTerminal;

    /// Build a Minitel terminal on an ESP32
    ///
    /// Generate the documentation locally with the ESP toolchain to see the actual content
    pub fn esp_minitel() -> ESPMinitel {
        unimplemented!()
    }

    /// Minitel terminal running on a ESP32
    ///
    /// Generate the documentation locally with the ESP toolchain to see the actual content
    pub fn esp_minitel_uart2() -> ESPMinitel {
        unimplemented!()
    }
}
