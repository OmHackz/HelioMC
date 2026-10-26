//! Helio core: all launcher logic. No terminal output, no println.
//!
//! The core is a pure library. It returns data, emits `Event`s over a channel,
//! and raises typed `HelioError`s. Every front end (the `helio` CLI today,
//! a TUI or GUI later) renders these.

pub mod banner;
pub mod brand;
pub mod config;
pub mod error;
pub mod events;
pub mod http;
pub mod icons;
pub mod logging;
pub mod logredact;
pub mod paths;
pub mod theme;
pub mod ui;

pub mod accounts;

pub use error::{HelioError, HelioResult};

/// Create a channel of events for progress reporting to a front end.
pub fn event_channel() -> (
    tokio::sync::mpsc::Sender<events::Event>,
    tokio::sync::mpsc::Receiver<events::Event>,
) {
    tokio::sync::mpsc::channel(256)
}
