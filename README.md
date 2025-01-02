# Minitel rust stack

## Building `minitel-app-example`

This crates is a sample app that can be built into a websocket server or an embedded ESP32 plugged on the minitel serial port.

Building for a websocket server: `cargo build -p minitel-app-example --features ws`.

Building for ESP: `cargo +esp --config minitel-app-example/cargo-config-esp.toml build -p minitel-app-example --features=esp`.
