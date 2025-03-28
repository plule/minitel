# Minitel rust stack

![Crates.io License](https://img.shields.io/crates/l/minitel) ![Crates.io Version](https://img.shields.io/crates/v/minitel) ![docs.rs](https://img.shields.io/docsrs/minitel) ![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/plule/minitel/ci.yml) 

This is an experimental rust development stack dedicated to the [Minitel](https://en.wikipedia.org/wiki/Minitel).

In its current state, it can be used to:

- Run on an ESP board such as [this one](https://www.tindie.com/products/iodeo/minitel-esp32-dongle/) to control the minitel
- Run on a websocket server to control an emulator such as [Miedit on Minipavi](http://www.minipavi.fr/emulminitel/indexws.php)
- Read and Write `.vdt` files

It includes an integration of [ratatui](https://ratatui.rs), a library to write console applications.

## Modules

The `minitel` crate contains everything needed for development, though most its feature are behind feature gates. Its module are:

- [stum]: Contains the core functionality, exposing the specificitation described in STUM1B (Spécifications Techniques d’Utilisation du Minitel).
- [futures]: Generic integration for the `futures` crate. Gated behind the `futures` feature.
- [axum]: Axum websocket integration. Gated behind the `ws` feature.
- [esp]: ESP32 integration. Gated behind the `esp` feature.
- [ratatui]: Ratatui backend module, compatible with both the previous integration. Gated behind the `ratatui` feature.

Lastly, [minitel-app-example] is a demonstration application that can be built both as a server serving a websocket, or as an embedded ESP32 firmware.

## Scope and limitations

This crate is focused on the Télétel Vidéotex standard, which is the one specific to the Minitel.

It does not include any management for the "mixte" or "téléinformatique" modes.

## Building `minitel-app-example`

This crate is a sample app that can be built into a websocket server or an embedded ESP32 plugged on the minitel serial port.

### TCP Socket

Creates a simple TCP listener.

`cargo run -p minitel-app-example --features tcp -- 0.0.0.0:3000`

### Axum WebSocket

Creates a web server serving a minitel websocket on `/ws`.

`cargo run -p minitel-app-example --features axum -- 0.0.0.0:3000`.

### ESP32

Creates a firmware running on ESP32, assuming the same pinout as the [iodeo dongle](https://www.tindie.com/products/iodeo/minitel-esp32-dongle/).

`cargo +esp --config minitel-app-example/cargo-config-esp.toml run -p minitel-app-example --features=esp -- --monitor`.

Targetting an ESP32 requires to have [setup an environment ready for ESP32 using the standard library](https://docs.esp-rs.org/book/introduction.html).

## Ratatui integration

`minitel::ratatui` includes a backend to run ratatui programs on Minitel. This backend supports most of the basic ratatui features.

It carries limitations from the Minitel standard. The most impactful one is the requirement of using space characters for changing certain attributes (background color, invert).

As a result, to control the background color, you must ensure to start the area with a space. Block's padding are useful to ensure this is the case, by adding a left padding: `.padding(Padding::left(1))`.

The current implementation does not take advantage of the Repeat control code, making page drawing slower than it should.

## ESP32 toolchain

Using the `esp` feature requires to have [setup an environment ready for ESP32 using the standard library](https://docs.esp-rs.org/book/introduction.html).

By default, on `docs.rs` a fake documentation stub will be built for the main esp members. The actual doc can be built with `cargo +esp doc --config minitel-app-example/cargo-config-esp.toml --features ratatui,esp --open`

