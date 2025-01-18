#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

/// Core Minitel types and traits
///
/// The stum module (Spécifications Techniques d'Utilisation du Minitel) exposes parts of the STUM1B specification.
pub mod stum;

#[doc(inline)]
pub use stum::Minitel;

/// Axum integration
#[cfg(feature = "axum")]
pub mod axum;

/// ESP32 integration
///
/// Implements the necessary traits to use a Minitel terminal over an ESP32 microcontroller.
#[cfg(feature = "esp")]
pub mod esp;

/// Ratatui integration
///
/// Exposes a backend for ratatui, a terminal UI library. This helps writing interactive
/// applications for the Minitel.
#[cfg(feature = "ratatui")]
pub mod ratatui;

#[cfg(feature = "ratatui")]
#[doc(inline)]
pub use ratatui::MinitelBackend;

#[cfg(feature = "esp")]
#[doc(inline)]
pub use esp::esp_minitel_uart2;
