//! Jolly Roger Office's core: the three apps of the suite, which app opens which file, the
//! recent-files list across all of them, the launcher's settings, and starting an app.
//!
//! Nothing here touches the network. The launcher only ever starts binaries that are already on
//! this computer (the bundle's own, or paths set in the settings).

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod apps;
pub mod brand;
pub mod config;
pub mod desktop;
pub mod launch;
pub mod recent;

pub use apps::{APPS, AppInfo, AppKind, route};
pub use config::{Config, ThemePref};
pub use recent::RecentFile;
