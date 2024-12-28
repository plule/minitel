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

pub mod prelude {
    #[cfg(feature = "ratatui")]
    pub use minitel_ratatui::MinitelBackend;
    pub use minitel_stum::{SerialMinitel, SerialPlugMinitel};
    #[cfg(feature = "ws")]
    pub use minitel_ws::WebSocketMinitel;

    #[cfg(all(feature = "ratatui", feature = "ws"))]
    pub type WebSocketMinitelTerminal =
        ratatui::Terminal<MinitelBackend<WebSocketMinitel<std::net::TcpStream>>>;
}
