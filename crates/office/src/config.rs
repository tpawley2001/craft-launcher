//! The launcher's settings (`config.json` in its settings folder) and where settings folders are.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::apps::AppKind;
use crate::brand;

/// Largest settings file read; anything bigger is not ours.
pub const MAX_SETTINGS_BYTES: u64 = 1 << 20;

/// Light, dark, or whatever the desktop uses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemePref {
    #[default]
    System,
    Light,
    Dark,
}

/// The launcher's settings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    /// Pirate flavour in the start page's chrome strings (on by default).
    pub pirate_speak: bool,
    pub theme: ThemePref,
    /// An app binary to use instead of the bundle's own.
    pub app_paths: BTreeMap<AppKind, PathBuf>,
    /// How many recent files the start page lists.
    pub recent_count: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config { pirate_speak: true, theme: ThemePref::System, app_paths: BTreeMap::new(), recent_count: 12 }
    }
}

impl Config {
    /// The settings in `dir`, or the defaults when there are none or they can't be read (the
    /// reason is returned beside them, for the log).
    pub fn load(dir: &Path) -> (Config, Option<String>) {
        let path = dir.join("config.json");
        match read_small(&path) {
            Ok(None) => (Config::default(), None),
            Ok(Some(bytes)) => match serde_json::from_slice::<Config>(&bytes) {
                Ok(c) => (c.sanitized(), None),
                Err(e) => (Config::default(), Some(format!("{} isn't valid ({e}); using the defaults", path.display()))),
            },
            Err(e) => (Config::default(), Some(e)),
        }
    }

    /// Write the settings to `dir` (created if needed), replacing the file atomically.
    pub fn save(&self, dir: &Path) -> Result<(), String> {
        let json = serde_json::to_vec_pretty(self).map_err(|e| format!("couldn't encode the settings ({e})"))?;
        write_atomic(&dir.join("config.json"), &json)
    }

    /// Keep values in range and drop empty paths.
    pub fn sanitized(mut self) -> Self {
        self.recent_count = self.recent_count.clamp(1, 50);
        self.app_paths.retain(|_, p| !p.as_os_str().is_empty());
        self
    }
}

/// The OS's per-user settings root: `$XDG_CONFIG_HOME` (or `~/.config`) on Linux and the BSDs,
/// `~/Library/Application Support` on macOS, `%APPDATA%` on Windows.
pub fn config_root() -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    } else if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    }
}

/// The launcher's settings folder: `JOLLY_ROGER_OFFICE_CONFIG_DIR` when set (tests, portable
/// use), else `<config root>/jolly-roger-office`.
pub fn launcher_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("JOLLY_ROGER_OFFICE_CONFIG_DIR").filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(dir));
    }
    config_root().map(|r| r.join(brand::SUITE_ID))
}

/// The contents of `path` when it exists and is at most [`MAX_SETTINGS_BYTES`]; `Ok(None)` when
/// it doesn't exist.
pub fn read_small(path: &Path) -> Result<Option<Vec<u8>>, String> {
    use std::io::Read;
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("couldn't read {} ({e})", path.display())),
    };
    let mut bytes = Vec::new();
    file.take(MAX_SETTINGS_BYTES + 1).read_to_end(&mut bytes).map_err(|e| format!("couldn't read {} ({e})", path.display()))?;
    if bytes.len() as u64 > MAX_SETTINGS_BYTES {
        return Err(format!("{} is too large to be a settings file", path.display()));
    }
    Ok(Some(bytes))
}

/// Write `bytes` to `path` through a temporary file beside it, so a crash never leaves half a file.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let dir = path.parent().ok_or_else(|| format!("{} has no folder", path.display()))?;
    std::fs::create_dir_all(dir).map_err(|e| format!("couldn't create {} ({e})", dir.display()))?;
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let tmp = dir.join(format!(".{name}.{}.tmp", std::process::id()));
    std::fs::write(&tmp, bytes).map_err(|e| format!("couldn't write {} ({e})", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("couldn't replace {} ({e})", path.display())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_have_pirate_speak_on() {
        let c = Config::default();
        assert!(c.pirate_speak);
        assert_eq!(c.theme, ThemePref::System);
    }

    #[test]
    fn missing_file_is_the_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let (c, warn) = Config::load(dir.path());
        assert_eq!(c, Config::default());
        assert!(warn.is_none());
    }

    #[test]
    fn corrupt_file_is_the_defaults_with_a_warning() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("config.json"), b"{not json").unwrap();
        let (c, warn) = Config::load(dir.path());
        assert_eq!(c, Config::default());
        assert!(warn.unwrap().contains("isn't valid"));
    }

    #[test]
    fn round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let mut c = Config { pirate_speak: false, theme: ThemePref::Dark, ..Config::default() };
        c.app_paths.insert(AppKind::Broadside, PathBuf::from("/opt/broadside"));
        c.save(&dir.path().join("sub")).unwrap();
        let (back, warn) = Config::load(&dir.path().join("sub"));
        assert!(warn.is_none());
        assert_eq!(back, c);
    }

    #[test]
    fn partial_file_keeps_other_defaults() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("config.json"), br#"{"theme":"light","recentCount":9999,"appPaths":{"captains-log":""}}"#).unwrap();
        let (c, _) = Config::load(dir.path());
        assert!(c.pirate_speak);
        assert_eq!(c.theme, ThemePref::Light);
        assert_eq!(c.recent_count, 50);
        assert!(c.app_paths.is_empty());
    }

    #[test]
    fn huge_file_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.json");
        std::fs::write(&p, vec![b' '; (MAX_SETTINGS_BYTES + 10) as usize]).unwrap();
        assert!(read_small(&p).is_err());
        let (c, warn) = Config::load(dir.path());
        assert_eq!(c, Config::default());
        assert!(warn.is_some());
    }
}
