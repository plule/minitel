mod app;

#[cfg(feature = "esp")]
#[path = "main_esp.rs"]
mod main;

#[cfg(feature = "axum")]
#[path = "main_axum.rs"]
mod main;

fn main() {
    crate::main::main();
}
