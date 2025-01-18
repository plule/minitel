mod app;

#[cfg(feature = "esp")]
#[path = "main_esp.rs"]
mod main;

#[cfg(feature = "axum")]
#[path = "main_axum.rs"]
mod main;

#[cfg(feature = "tcp")]
#[path = "main_tcp.rs"]
mod main;

fn main() {
    crate::main::main();
}
