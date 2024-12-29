pub mod stum {
    pub use minitel_stum::*;
}

#[cfg(feature = "ws")]
pub mod ws {
    pub use minitel_ws::*;
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
