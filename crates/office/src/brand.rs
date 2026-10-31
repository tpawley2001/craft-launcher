//! Every name, id and flavour string of the suite, in one place: the names are placeholders the
//! owner may change, so nothing else in the workspace spells them out.

/// The suite's name, shown in the window title, About and the desktop menu.
pub const SUITE_NAME: &str = "Jolly Roger Office";
/// Machine name: the launcher binary, its settings folder and the icon file stem.
pub const SUITE_ID: &str = "jolly-roger-office";
/// Reverse-DNS application id (Wayland app id, desktop file name, icon name).
pub const APP_ID: &str = "tech.hillyard.JollyRogerOffice";
/// Who ships this build.
pub const PUBLISHER: &str = "Hillyard Tech";
/// The credit line every surface that names the suite also carries.
pub const CREDIT: &str = "Based on the ArtCraft Crafting Apps (MIT OR Apache-2.0)";
/// The launcher's version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The flavour strings of the start page. Only chrome is flavoured: commands, dialogs and errors
/// stay plain English whatever `pirate_speak` says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flavour {
    /// The big greeting on the start page.
    pub greeting: &'static str,
    /// The line under the greeting.
    pub subtitle: &'static str,
    /// Heading over the three "New" tiles.
    pub new_heading: &'static str,
    /// Heading over the recent-files list.
    pub recent_heading: &'static str,
    /// The recent-files list when it is empty.
    pub recent_empty: &'static str,
    /// The hint under an empty recent-files list.
    pub recent_empty_hint: &'static str,
    /// A tile whose app binary can't be found.
    pub app_missing: &'static str,
}

/// Pirate flavour (the default).
pub const PIRATE: Flavour = Flavour {
    greeting: "Ahoy, Captain",
    subtitle: "What be we charting today? Pick a vessel or haul up a file from the hold.",
    new_heading: "Set sail with something new",
    recent_heading: "Recent plunder",
    recent_empty: "The log be empty, Captain.",
    recent_empty_hint: "Files you open from here or in any of the three apps will wash up on this shore.",
    app_missing: "Lost at sea: not found",
};

/// Plain English, for `pirate_speak = false`.
pub const PLAIN: Flavour = Flavour {
    greeting: "Welcome",
    subtitle: "Start something new or pick up where you left off.",
    new_heading: "New",
    recent_heading: "Recent files",
    recent_empty: "No recent files yet.",
    recent_empty_hint: "Files you open from here or in any of the three apps will appear here.",
    app_missing: "Not found",
};

/// The flavour strings for the `pirate_speak` setting.
pub fn flavour(pirate_speak: bool) -> &'static Flavour {
    if pirate_speak { &PIRATE } else { &PLAIN }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credit_names_the_licence() {
        assert!(CREDIT.contains("MIT OR Apache-2.0"));
        assert!(CREDIT.starts_with("Based on the ArtCraft"));
    }

    #[test]
    fn pirate_speak_switches_flavour() {
        assert_eq!(flavour(true).greeting, "Ahoy, Captain");
        assert_eq!(flavour(false).greeting, "Welcome");
    }

    #[test]
    fn plain_flavour_has_no_pirate_words() {
        let p = PLAIN;
        for s in [p.greeting, p.subtitle, p.new_heading, p.recent_heading, p.recent_empty, p.recent_empty_hint, p.app_missing] {
            let l = s.to_lowercase();
            for word in ["ahoy", "captain", "plunder", "sea", " be "] {
                assert!(!l.contains(word), "{s:?} contains {word:?}");
            }
        }
    }
}
