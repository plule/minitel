pub mod stum {
    pub use minitel_stum::*;
}

#[cfg(feature = "ws")]
pub mod ws {
    pub use minitel_ws::*;
}

pub mod prelude {
    pub use minitel_stum::{SerialMinitel, SerialPlugMinitel};
    #[cfg(feature = "ws")]
    pub use minitel_ws::WebSocketMinitel;
}
