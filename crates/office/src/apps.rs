//! The suite's three apps and which of them opens which file.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// One of the suite's apps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AppKind {
    /// The word processor (a WordCraft fork).
    CaptainsLog,
    /// The spreadsheet (a GridCraft fork).
    PlunderLedger,
    /// The presentation app (a DeckCraft fork).
    Broadside,
}

/// What the launcher knows about an app.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppInfo {
    pub kind: AppKind,
    /// The app's name, as its window and menus show it.
    pub name: &'static str,
    /// Machine name: the binary in the bundle (`apps/<id>`), the desktop file and icon stem.
    pub id: &'static str,
    /// What kind of app it is, in plain words.
    pub generic: &'static str,
    /// The pirate label of its "New" tile.
    pub new_pirate: &'static str,
    /// The plain label of its "New" tile.
    pub new_plain: &'static str,
    /// Other names its binary may have on `PATH` (the fork's own names come first).
    pub binary_names: &'static [&'static str],
    /// Settings folder names (under the config dir) whose `ui.json` holds the app's recent
    /// files: the fork's names first, then upstream's, which the fork may still use.
    pub settings_dirs: &'static [&'static str],
    /// File extensions the launcher routes to this app (lower case, no dot). The first is the
    /// app's main format.
    pub extensions: &'static [&'static str],
    /// MIME types the app's desktop entry claims.
    pub mime_types: &'static [&'static str],
    /// Freedesktop menu categories.
    pub categories: &'static str,
}

impl AppInfo {
    /// The app's entry in [`APPS`].
    pub fn of(kind: AppKind) -> &'static AppInfo {
        match kind {
            AppKind::CaptainsLog => &APPS[0],
            AppKind::PlunderLedger => &APPS[1],
            AppKind::Broadside => &APPS[2],
        }
    }

    /// The "New" tile's label for the `pirate_speak` setting.
    pub fn new_label(&self, pirate_speak: bool) -> &'static str {
        if pirate_speak { self.new_pirate } else { self.new_plain }
    }
}

/// The suite, in tile order.
pub const APPS: [AppInfo; 3] = [
    AppInfo {
        kind: AppKind::CaptainsLog,
        name: "Captain's Log",
        id: "captains-log",
        generic: "Word processor",
        new_pirate: "Open a fresh log",
        new_plain: "New document",
        binary_names: &["captains-log", "captainslog"],
        settings_dirs: &["captains-log", "captainslog", "CaptainsLog", "wordcraft", "WordCraft"],
        extensions: &["docx", "docm", "dotx", "dotm", "doc", "dot", "odt", "rtf", "txt", "md", "markdown", "html", "htm", "tex"],
        mime_types: &[
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "application/vnd.ms-word.document.macroEnabled.12",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.template",
            "application/vnd.ms-word.template.macroEnabled.12",
            "application/msword",
            "application/msword-template",
            "application/vnd.oasis.opendocument.text",
            "application/rtf",
            "text/rtf",
            "text/markdown",
            "text/x-tex",
        ],
        categories: "Office;WordProcessor;",
    },
    AppInfo {
        kind: AppKind::PlunderLedger,
        name: "Plunder Ledger",
        id: "plunder-ledger",
        generic: "Spreadsheet",
        new_pirate: "Tally the treasure",
        new_plain: "New workbook",
        binary_names: &["plunder-ledger", "plunderledger"],
        settings_dirs: &["plunder-ledger", "plunderledger", "PlunderLedger", "gridcraft", "GridCraft"],
        extensions: &["xlsx", "xlsm", "xlsb", "xls", "ods", "csv", "tsv"],
        mime_types: &[
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "application/vnd.ms-excel.sheet.macroEnabled.12",
            "application/vnd.ms-excel.sheet.binary.macroEnabled.12",
            "application/vnd.ms-excel",
            "application/vnd.oasis.opendocument.spreadsheet",
            "text/csv",
            "text/tab-separated-values",
        ],
        categories: "Office;Spreadsheet;",
    },
    AppInfo {
        kind: AppKind::Broadside,
        name: "Broadside",
        id: "broadside",
        generic: "Presentations",
        new_pirate: "Load the cannons",
        new_plain: "New presentation",
        binary_names: &["broadside"],
        settings_dirs: &["broadside", "Broadside", "deckcraft", "DeckCraft"],
        extensions: &["pptx", "potx", "ppsx", "deckcraft"],
        mime_types: &[
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "application/vnd.openxmlformats-officedocument.presentationml.template",
            "application/vnd.openxmlformats-officedocument.presentationml.slideshow",
            "application/x-deckcraft",
        ],
        categories: "Office;Presentation;",
    },
];

/// The app that opens `path`, by its extension (any case), or `None` for a file none of them
/// opens. Plain text, Markdown and HTML go to Captain's Log; CSV and TSV to Plunder Ledger.
pub fn route(path: &Path) -> Option<AppKind> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    APPS.iter().find(|a| a.extensions.contains(&ext.as_str())).map(|a| a.kind)
}

/// Every extension the suite opens, for the Open dialog's filter.
pub fn all_extensions() -> Vec<&'static str> {
    APPS.iter().flat_map(|a| a.extensions.iter().copied()).collect()
}

/// Every MIME type the suite claims, for the launcher's own desktop entry.
pub fn all_mime_types() -> Vec<&'static str> {
    APPS.iter().flat_map(|a| a.mime_types.iter().copied()).collect()
}

/// The app whose machine name is `id` (`captains-log`, `plunder-ledger`, `broadside`), also
/// accepting the upstream names (`wordcraft`, `gridcraft`, `deckcraft`) and the generic words
/// `doc`, `sheet` and `deck`.
pub fn by_id(id: &str) -> Option<AppKind> {
    match id.to_ascii_lowercase().as_str() {
        "captains-log" | "captainslog" | "log" | "wordcraft" | "doc" | "document" | "word" => Some(AppKind::CaptainsLog),
        "plunder-ledger" | "plunderledger" | "ledger" | "gridcraft" | "sheet" | "spreadsheet" | "workbook" => Some(AppKind::PlunderLedger),
        "broadside" | "deckcraft" | "deck" | "slides" | "presentation" => Some(AppKind::Broadside),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn office_formats_route_to_their_app() {
        assert_eq!(route(Path::new("/x/report.docx")), Some(AppKind::CaptainsLog));
        assert_eq!(route(Path::new("Budget.XLSX")), Some(AppKind::PlunderLedger));
        assert_eq!(route(Path::new("pitch.pptx")), Some(AppKind::Broadside));
        assert_eq!(route(Path::new("notes.md")), Some(AppKind::CaptainsLog));
        assert_eq!(route(Path::new("data.csv")), Some(AppKind::PlunderLedger));
        assert_eq!(route(Path::new("old.doc")), Some(AppKind::CaptainsLog));
        assert_eq!(route(Path::new("old.xls")), Some(AppKind::PlunderLedger));
        assert_eq!(route(Path::new("a.odt")), Some(AppKind::CaptainsLog));
        assert_eq!(route(Path::new("a.ods")), Some(AppKind::PlunderLedger));
        assert_eq!(route(Path::new("show.deckcraft")), Some(AppKind::Broadside));
    }

    #[test]
    fn unknown_files_route_nowhere() {
        assert_eq!(route(Path::new("photo.png")), None);
        assert_eq!(route(Path::new("README")), None);
        assert_eq!(route(Path::new(".docx")), None);
        assert_eq!(route(Path::new("")), None);
    }

    #[test]
    fn no_extension_belongs_to_two_apps() {
        let mut seen = HashSet::new();
        for e in all_extensions() {
            assert!(seen.insert(e), "{e} is listed twice");
            assert_eq!(e, e.to_ascii_lowercase());
        }
    }

    #[test]
    fn of_matches_table_order() {
        for a in &APPS {
            assert_eq!(AppInfo::of(a.kind).kind, a.kind);
            assert_eq!(by_id(a.id), Some(a.kind));
        }
        assert_eq!(by_id("WordCraft"), Some(AppKind::CaptainsLog));
        assert_eq!(by_id("nope"), None);
    }

    #[test]
    fn serde_names_are_the_ids() {
        for a in &APPS {
            assert_eq!(serde_json::to_string(&a.kind).unwrap(), format!("\"{}\"", a.id));
        }
    }
}
