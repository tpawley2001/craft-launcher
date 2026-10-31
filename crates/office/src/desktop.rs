//! Freedesktop menu entries and file associations for the suite.
//!
//! Every entry runs the launcher: the suite's own entry opens the start page (or routes the
//! files it is given), and each app's entry runs `--with <app>` so the launcher records the file
//! in the recent list and starts the right binary. File managers offer each app for its own
//! types and the suite for all of them.

use crate::apps::{APPS, AppInfo, all_mime_types};
use crate::brand;

/// The desktop file name of the suite's entry (its stem is also the Wayland app id).
pub fn suite_file_name() -> String {
    format!("{}.desktop", brand::APP_ID)
}

/// The desktop file name of `app`'s entry.
pub fn app_file_name(app: &AppInfo) -> String {
    format!("{}.{}.desktop", brand::APP_ID, app.id)
}

/// The icon name of `app` (`<APP_ID>.<id>`); the suite's icon is [`brand::APP_ID`].
pub fn app_icon_name(app: &AppInfo) -> String {
    format!("{}.{}", brand::APP_ID, app.id)
}

/// One argument of an `Exec` line, quoted as the Desktop Entry spec asks when it needs it.
pub fn exec_arg(arg: &str) -> String {
    let plain = !arg.is_empty() && arg.chars().all(|c| c.is_ascii_alphanumeric() || "-_./+,:@=".contains(c));
    let quoted = if plain {
        arg.to_string()
    } else {
        let mut s = String::from("\"");
        for c in arg.chars() {
            if matches!(c, '"' | '`' | '$' | '\\') {
                s.push('\\');
            }
            s.push(c);
        }
        s.push('"');
        s
    };
    // The Exec value is itself a string value, whose backslashes are escaped once more; a
    // literal % is written %%.
    quoted.replace('\\', "\\\\").replace('%', "%%")
}

/// The suite's own entry; `launcher` is the launcher binary's absolute path.
pub fn suite_entry(launcher: &str) -> String {
    let exec = exec_arg(launcher);
    let mimes: String = all_mime_types().iter().map(|m| format!("{m};")).collect();
    let actions: String = APPS.iter().map(|a| format!("new-{};", a.id)).collect();
    let mut s = format!(
        "[Desktop Entry]\nType=Application\nName={name}\nGenericName=Office Suite\nComment=Write, reckon and present: the Jolly Roger Office start page. {credit}\n\
         Exec={exec} %F\nTryExec={exec}\nIcon={icon}\nTerminal=false\nStartupNotify=true\nStartupWMClass={icon}\nCategories=Office;\n\
         Keywords=office;suite;document;spreadsheet;presentation;docx;xlsx;pptx;\nMimeType={mimes}\nActions={actions}\n",
        name = brand::SUITE_NAME,
        credit = brand::CREDIT,
        icon = brand::APP_ID,
    );
    for a in &APPS {
        s.push_str(&format!(
            "\n[Desktop Action new-{id}]\nName=New {name}\nExec={exec} --new {id}\nIcon={icon}\n",
            id = a.id,
            name = a.name,
            icon = app_icon_name(a)
        ));
    }
    s
}

/// `app`'s entry; `launcher` is the launcher binary's absolute path.
pub fn app_entry(app: &AppInfo, launcher: &str) -> String {
    let exec = exec_arg(launcher);
    let mimes: String = app.mime_types.iter().map(|m| format!("{m};")).collect();
    format!(
        "[Desktop Entry]\nType=Application\nName={name}\nGenericName={generic}\nComment={generic} in {suite}\n\
         Exec={exec} --with {id} %F\nTryExec={exec}\nIcon={icon}\nTerminal=false\nStartupNotify=true\nCategories={cats}\nMimeType={mimes}\n",
        name = app.name,
        generic = app.generic,
        suite = brand::SUITE_NAME,
        id = app.id,
        icon = app_icon_name(app),
        cats = app.categories,
    )
}

/// The shared-mime-info package for the one type the desktop doesn't know already: DeckCraft's
/// own `.deckcraft` presentations.
pub fn mime_package() -> String {
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
     <mime-info xmlns=\"http://www.freedesktop.org/standards/shared-mime-info\">\n\
     \x20 <mime-type type=\"application/x-deckcraft\">\n\
     \x20   <comment>Broadside presentation</comment>\n\
     \x20   <sub-class-of type=\"application/zip\"/>\n\
     \x20   <glob pattern=\"*.deckcraft\"/>\n\
     \x20 </mime-type>\n\
     </mime-info>\n"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_args_are_quoted_when_needed() {
        assert_eq!(exec_arg("/opt/jro/jolly-roger-office"), "/opt/jro/jolly-roger-office");
        assert_eq!(exec_arg("/home/c/My Apps/jro"), "\"/home/c/My Apps/jro\"");
        assert_eq!(exec_arg("/a$b"), "\"/a\\\\$b\"");
        assert_eq!(exec_arg("/100%"), "\"/100%%\"");
    }

    #[test]
    fn suite_entry_claims_every_type_and_has_new_actions() {
        let e = suite_entry("/opt/jro/jolly-roger-office");
        assert!(e.starts_with("[Desktop Entry]\n"));
        assert!(e.contains("Exec=/opt/jro/jolly-roger-office %F\n"));
        assert!(e.contains("application/vnd.openxmlformats-officedocument.wordprocessingml.document;"));
        assert!(e.contains("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet;"));
        assert!(e.contains("application/vnd.openxmlformats-officedocument.presentationml.presentation;"));
        assert!(e.contains("[Desktop Action new-broadside]\n"));
        assert!(e.contains("Exec=/opt/jro/jolly-roger-office --new captains-log\n"));
        assert!(!e.to_lowercase().contains("artcraft launcher"));
    }

    #[test]
    fn app_entries_route_through_the_launcher() {
        for a in &APPS {
            let e = app_entry(a, "/j/jro");
            assert!(e.contains(&format!("Exec=/j/jro --with {} %F\n", a.id)));
            assert!(e.contains(&format!("Name={}\n", a.name)));
            assert!(e.contains(&format!("Icon={}\n", app_icon_name(a))));
            for m in a.mime_types {
                assert!(e.contains(m));
            }
        }
    }

    #[test]
    fn file_names() {
        assert_eq!(suite_file_name(), "tech.hillyard.JollyRogerOffice.desktop");
        assert_eq!(app_file_name(&APPS[0]), "tech.hillyard.JollyRogerOffice.captains-log.desktop");
    }
}
