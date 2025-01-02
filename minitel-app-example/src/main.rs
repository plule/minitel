mod app;

#[cfg(feature = "esp")]
#[path = "main_esp.rs"]
mod main;

#[cfg(feature = "ws")]
#[path = "main_ws.rs"]
mod main;

fn main() {
    crate::main::main();
}
