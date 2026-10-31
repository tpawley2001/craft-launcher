//! Jolly Roger Office's interface: the suite's start page (New, Open, recent files), its pirate
//! palette and its icons, drawn in code. The desktop shell supplies a [`Host`] for launching apps,
//! file dialogs and storage.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod home;
pub mod icons;
pub mod theme;

pub use home::{HomeUi, Host};
pub use icons::Icon;
