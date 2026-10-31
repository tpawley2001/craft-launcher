//! Recent files across the three apps.
//!
//! Two sources, merged: the launcher's own list (`recent.json` in its settings folder: every file
//! it opened), and each app's own Recent list (the `recent` array in the app's `ui.json`, which
//! WordCraft, GridCraft and DeckCraft all keep, most recent first). An app's list carries no
//! times, so its entries are dated by when that `ui.json` was last written, one second apart in
//! list order.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::apps::{APPS, AppInfo, AppKind};
use crate::config::{read_small, write_atomic};

/// Most entries the launcher's own list keeps.
pub const OWN_MAX: usize = 50;
/// Most entries read from one app's list.
pub const APP_MAX: usize = 50;

/// One recent file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentFile {
    pub path: PathBuf,
    /// The app it was opened in.
    pub app: AppKind,
    /// When, in seconds since the Unix epoch.
    pub opened: i64,
}

impl RecentFile {
    /// The file name, for the list's first column.
    pub fn file_name(&self) -> String {
        self.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| self.path.display().to_string())
    }

    /// The folder it is in.
    pub fn folder(&self) -> String {
        self.path.parent().map(|p| p.display().to_string()).unwrap_or_default()
    }
}

/// Now, in seconds since the Unix epoch (0 if the clock is before it).
pub fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX)).unwrap_or(0)
}

/// The launcher's own list from `dir`; empty when there is none or it can't be read.
pub fn load_own(dir: &Path) -> Vec<RecentFile> {
    match read_small(&dir.join("recent.json")) {
        Ok(Some(bytes)) => serde_json::from_slice::<Vec<RecentFile>>(&bytes).unwrap_or_default().into_iter().take(OWN_MAX).collect(),
        _ => Vec::new(),
    }
}

/// Put `path` (opened in `app` at `when`) at the top of the launcher's own list in `dir`.
pub fn record(dir: &Path, path: &Path, app: AppKind, when: i64) -> Result<(), String> {
    let mut list = load_own(dir);
    list.retain(|r| r.path != path);
    list.insert(0, RecentFile { path: path.to_path_buf(), app, opened: when });
    list.truncate(OWN_MAX);
    let json = serde_json::to_vec_pretty(&list).map_err(|e| format!("couldn't encode the recent files ({e})"))?;
    write_atomic(&dir.join("recent.json"), &json)
}

/// Remove `path` from the launcher's own list in `dir` (it is still in the app's own list, if
/// the app has it).
pub fn forget(dir: &Path, path: &Path) -> Result<(), String> {
    let mut list = load_own(dir);
    let before = list.len();
    list.retain(|r| r.path != path);
    if list.len() == before {
        return Ok(());
    }
    let json = serde_json::to_vec_pretty(&list).map_err(|e| format!("couldn't encode the recent files ({e})"))?;
    write_atomic(&dir.join("recent.json"), &json)
}

/// The recent files `app` lists in its own `ui.json` under `config_root`, in any of its known
/// settings folders.
pub fn app_list(config_root: &Path, app: &AppInfo) -> Vec<RecentFile> {
    let mut out = Vec::new();
    for dir in app.settings_dirs {
        let path = config_root.join(dir).join("ui.json");
        let Ok(Some(bytes)) = read_small(&path) else { continue };
        let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else { continue };
        let Some(items) = value.get("recent").and_then(|r| r.as_array()) else { continue };
        let written = std::fs::metadata(&path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .and_then(|d| i64::try_from(d.as_secs()).ok())
            .unwrap_or(0);
        for (i, item) in items.iter().take(APP_MAX).enumerate() {
            let Some(p) = item.as_str().filter(|s| !s.is_empty()) else { continue };
            let offset = i64::try_from(i).unwrap_or(i64::MAX);
            out.push(RecentFile { path: PathBuf::from(p), app: app.kind, opened: written.saturating_sub(offset) });
        }
    }
    out
}

/// Merge `lists` into one: each path once (at its latest time), newest first, only the ones
/// `keep` accepts (normally: still on disk), at most `limit`.
pub fn merge(lists: impl IntoIterator<Item = Vec<RecentFile>>, limit: usize, keep: impl Fn(&Path) -> bool) -> Vec<RecentFile> {
    let mut best: HashMap<PathBuf, RecentFile> = HashMap::new();
    for entry in lists.into_iter().flatten() {
        match best.get(&entry.path) {
            Some(old) if old.opened >= entry.opened => {}
            _ => {
                best.insert(entry.path.clone(), entry);
            }
        }
    }
    let mut all: Vec<RecentFile> = best.into_values().collect();
    all.sort_by(|a, b| b.opened.cmp(&a.opened).then_with(|| a.path.cmp(&b.path)));
    all.into_iter().filter(|r| keep(&r.path)).take(limit).collect()
}

/// Everything: the launcher's list in `launcher_dir` and every app's list under `config_root`,
/// merged, files that are gone left out.
pub fn gather(launcher_dir: Option<&Path>, config_root: Option<&Path>, limit: usize) -> Vec<RecentFile> {
    let mut lists = Vec::new();
    if let Some(dir) = launcher_dir {
        lists.push(load_own(dir));
    }
    if let Some(root) = config_root {
        for app in &APPS {
            lists.push(app_list(root, app));
        }
    }
    merge(lists, limit, |p| p.is_file())
}

/// "just now", "5 min ago", "3 h ago", "2 days ago"… for a time `secs` before `now`.
pub fn ago(when: i64, now: i64) -> String {
    let d = now.saturating_sub(when).max(0);
    match d {
        0..60 => "just now".into(),
        60..3_600 => format!("{} min ago", d / 60),
        3_600..86_400 => format!("{} h ago", d / 3_600),
        86_400..172_800 => "yesterday".into(),
        172_800..1_209_600 => format!("{} days ago", d / 86_400),
        1_209_600..5_259_600 => format!("{} weeks ago", d / 604_800),
        5_259_600..31_557_600 => format!("{} months ago", d / 2_629_800),
        _ => "over a year ago".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rf(p: &str, app: AppKind, t: i64) -> RecentFile {
        RecentFile { path: PathBuf::from(p), app, opened: t }
    }

    #[test]
    fn record_puts_newest_first_and_dedupes() {
        let dir = tempfile::tempdir().unwrap();
        record(dir.path(), Path::new("/a.docx"), AppKind::CaptainsLog, 10).unwrap();
        record(dir.path(), Path::new("/b.xlsx"), AppKind::PlunderLedger, 20).unwrap();
        record(dir.path(), Path::new("/a.docx"), AppKind::CaptainsLog, 30).unwrap();
        let l = load_own(dir.path());
        assert_eq!(l, vec![rf("/a.docx", AppKind::CaptainsLog, 30), rf("/b.xlsx", AppKind::PlunderLedger, 20)]);
        forget(dir.path(), Path::new("/a.docx")).unwrap();
        assert_eq!(load_own(dir.path()).len(), 1);
    }

    #[test]
    fn own_list_is_capped() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..(OWN_MAX + 5) {
            record(dir.path(), Path::new(&format!("/f{i}.docx")), AppKind::CaptainsLog, i as i64).unwrap();
        }
        assert_eq!(load_own(dir.path()).len(), OWN_MAX);
    }

    #[test]
    fn corrupt_own_list_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("recent.json"), b"[{]").unwrap();
        assert!(load_own(dir.path()).is_empty());
        // And recording over it starts afresh instead of failing.
        record(dir.path(), Path::new("/x.pptx"), AppKind::Broadside, 1).unwrap();
        assert_eq!(load_own(dir.path()).len(), 1);
    }

    #[test]
    fn reads_each_apps_ui_json() {
        let root = tempfile::tempdir().unwrap();
        for (dir, files) in [("wordcraft", r#"["/w1.docx","/w2.docx"]"#), ("gridcraft", r#"["/g.xlsx", 5, ""]"#), ("broadside", r#"["/d.pptx"]"#)] {
            std::fs::create_dir_all(root.path().join(dir)).unwrap();
            std::fs::write(root.path().join(dir).join("ui.json"), format!(r#"{{"zoom":1,"recent":{files}}}"#)).unwrap();
        }
        let w = app_list(root.path(), AppInfo::of(AppKind::CaptainsLog));
        assert_eq!(w.iter().map(|r| r.path.to_str().unwrap()).collect::<Vec<_>>(), ["/w1.docx", "/w2.docx"]);
        assert!(w[0].opened > w[1].opened);
        let g = app_list(root.path(), AppInfo::of(AppKind::PlunderLedger));
        assert_eq!(g.len(), 1, "non-strings and blanks are skipped");
        let d = app_list(root.path(), AppInfo::of(AppKind::Broadside));
        assert_eq!(d[0].app, AppKind::Broadside);
    }

    #[test]
    fn unreadable_app_settings_are_skipped() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("deckcraft")).unwrap();
        std::fs::write(root.path().join("deckcraft/ui.json"), b"garbage").unwrap();
        std::fs::create_dir_all(root.path().join("gridcraft")).unwrap();
        std::fs::write(root.path().join("gridcraft/ui.json"), br#"{"recent":"not a list"}"#).unwrap();
        assert!(app_list(root.path(), AppInfo::of(AppKind::Broadside)).is_empty());
        assert!(app_list(root.path(), AppInfo::of(AppKind::PlunderLedger)).is_empty());
    }

    #[test]
    fn merge_keeps_latest_and_filters() {
        let a = vec![rf("/a", AppKind::CaptainsLog, 5), rf("/b", AppKind::PlunderLedger, 9)];
        let b = vec![rf("/a", AppKind::CaptainsLog, 7), rf("/gone", AppKind::Broadside, 100)];
        let m = merge([a, b], 10, |p| p != Path::new("/gone"));
        assert_eq!(m, vec![rf("/b", AppKind::PlunderLedger, 9), rf("/a", AppKind::CaptainsLog, 7)]);
        let m = merge([vec![rf("/a", AppKind::CaptainsLog, 1), rf("/b", AppKind::CaptainsLog, 2)]], 1, |_| true);
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn gather_merges_both_sources_and_drops_missing_files() {
        let root = tempfile::tempdir().unwrap();
        let docs = tempfile::tempdir().unwrap();
        let real = docs.path().join("map.xlsx");
        std::fs::write(&real, b"x").unwrap();
        let launcher = root.path().join("jolly-roger-office");
        record(&launcher, &real, AppKind::PlunderLedger, 1).unwrap();
        record(&launcher, Path::new("/nowhere/lost.docx"), AppKind::CaptainsLog, 2).unwrap();
        let got = gather(Some(&launcher), Some(root.path()), 10);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].path, real);
    }

    #[test]
    fn ago_reads_naturally() {
        assert_eq!(ago(100, 110), "just now");
        assert_eq!(ago(0, 300), "5 min ago");
        assert_eq!(ago(0, 7_200), "2 h ago");
        assert_eq!(ago(0, 90_000), "yesterday");
        assert_eq!(ago(0, 3 * 86_400), "3 days ago");
        assert_eq!(ago(0, 21 * 86_400), "3 weeks ago");
        assert_eq!(ago(0, 400 * 86_400), "over a year ago");
        assert_eq!(ago(500, 100), "just now", "a time in the future is now");
    }

    #[test]
    fn names_for_display() {
        let r = rf("/home/c/Logs/voyage.docx", AppKind::CaptainsLog, 0);
        assert_eq!(r.file_name(), "voyage.docx");
        assert_eq!(r.folder(), "/home/c/Logs");
    }
}
