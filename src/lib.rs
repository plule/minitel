pub mod stum {
    pub use minitel_stum::*;
}

#[cfg(feature = "ws")]
pub mod ws {
    pub use minitel_ws::*;
}

#[cfg(feature = "esp")]
pub mod esp {
    pub use minitel_esp::*;
}

#[cfg(feature = "ratatui")]
pub mod ratatui {
    pub use minitel_ratatui::*;
}

#[cfg(feature = "ratatui")]
pub use minitel_ratatui::MinitelBackend;

#[cfg(feature = "ws")]
pub use minitel_ws::WSMinitel;

#[cfg(feature = "ws")]
pub use minitel_ws::ws_minitel;

#[cfg(all(feature = "ws", feature = "ratatui"))]
pub type WSTerminal =
    ::ratatui::Terminal<ratatui::MinitelBackend<minitel_ws::WSPort<std::net::TcpStream>>>;

#[cfg(all(feature = "ws", feature = "ratatui"))]
pub fn ws_terminal(minitel: minitel_ws::WSMinitel) -> std::io::Result<WSTerminal> {
    WSTerminal::new(ratatui::MinitelBackend::new(minitel))
}

#[cfg(feature = "esp")]
pub use minitel_esp::ESPMinitel;

#[cfg(feature = "esp")]
pub use minitel_esp::esp_minitel;

#[cfg(feature = "esp")]
pub use minitel_esp::esp_minitel_uart2;

#[cfg(all(feature = "esp", feature = "ratatui"))]
pub type ESPTerminal<'a> = ::ratatui::Terminal<ratatui::MinitelBackend<minitel_esp::ESPPort<'a>>>;

#[cfg(all(feature = "esp", feature = "ratatui"))]
pub fn esp_terminal<'a>(minitel: minitel_esp::ESPMinitel<'a>) -> std::io::Result<ESPTerminal<'a>> {
    ESPTerminal::new(ratatui::MinitelBackend::new(minitel))
}
