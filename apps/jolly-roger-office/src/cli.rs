//! The command line.

use std::ffi::OsString;
use std::path::PathBuf;

use jolly_roger_office_core::AppKind;
use jolly_roger_office_core::apps::by_id;

pub const HELP: &str = "\
Usage: jolly-roger-office [FILE...]
       jolly-roger-office --new <app>
       jolly-roger-office --with <app> [FILE...]

With no arguments, open the start page. Given files, open each in the app that handles it
(.docx and other documents in Captain's Log, .xlsx and .csv in Plunder Ledger, .pptx in
Broadside) and exit.

  --new <app>             start <app> with a new file: captains-log, plunder-ledger or broadside
  --with <app> [FILE...]  open the files in <app>, whatever their type
  --list                  show where each app's program was found
  --export-icons <dir>    write the suite's icons (PNG and SVG) under <dir>/hicolor
  --desktop-files <dir> [--launcher <path>]
                          write menu entries and file associations under <dir>
                          (applications/, mime/packages/) that run <path>
  --version               print the version
  --help                  show this help";

/// What the command line asks for.
#[derive(Debug, PartialEq, Eq)]
pub enum Cli {
    Window,
    Open(Vec<PathBuf>),
    New(AppKind),
    With(AppKind, Vec<PathBuf>),
    List,
    ExportIcons(PathBuf),
    DesktopFiles { out: PathBuf, launcher: Option<String> },
    Version,
    Help,
    Error(String),
}

fn app_arg(v: Option<&OsString>, flag: &str) -> Result<AppKind, String> {
    let v = v.ok_or_else(|| format!("{flag} needs an app: captains-log, plunder-ledger or broadside"))?;
    let s = v.to_string_lossy();
    by_id(&s).ok_or_else(|| format!("{s:?} isn't one of the apps: captains-log, plunder-ledger or broadside"))
}

/// Read `args` (without the program name). Paths stay `OsString`s: a file name that isn't UTF-8
/// still opens.
pub fn parse(args: Vec<OsString>) -> Cli {
    let first = args.first().map(|a| a.to_string_lossy().into_owned());
    let rest = || args.iter().skip(1);
    match first.as_deref() {
        None => Cli::Window,
        Some("--help" | "-h") => Cli::Help,
        Some("--version" | "-V") => Cli::Version,
        Some("--list") => Cli::List,
        Some("--new") => match app_arg(args.get(1), "--new") {
            Ok(a) if args.len() == 2 => Cli::New(a),
            Ok(_) => Cli::Error("--new takes only the app".into()),
            Err(e) => Cli::Error(e),
        },
        Some("--with") => match app_arg(args.get(1), "--with") {
            Ok(a) => Cli::With(a, args.iter().skip(2).map(PathBuf::from).collect()),
            Err(e) => Cli::Error(e),
        },
        Some("--export-icons") => match args.get(1) {
            Some(d) if args.len() == 2 => Cli::ExportIcons(PathBuf::from(d)),
            _ => Cli::Error("--export-icons takes one folder".into()),
        },
        Some("--desktop-files") => {
            let Some(out) = args.get(1) else { return Cli::Error("--desktop-files needs a folder".into()) };
            match (args.get(2).map(|a| a.to_string_lossy().into_owned()).as_deref(), args.get(3), args.len()) {
                (None, _, 2) => Cli::DesktopFiles { out: PathBuf::from(out), launcher: None },
                (Some("--launcher"), Some(l), 4) => Cli::DesktopFiles { out: PathBuf::from(out), launcher: Some(l.to_string_lossy().into_owned()) },
                _ => Cli::Error("usage: --desktop-files <dir> [--launcher <path>]".into()),
            }
        }
        // `--` ends the options: everything after it is a file.
        Some("--") => Cli::Open(rest().map(PathBuf::from).collect()),
        Some(flag) if flag.starts_with("--") => Cli::Error(format!("unknown option {flag}")),
        Some(_) => Cli::Open(args.iter().map(PathBuf::from).collect()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(args: &[&str]) -> Cli {
        parse(args.iter().map(OsString::from).collect())
    }

    #[test]
    fn parses() {
        assert_eq!(p(&[]), Cli::Window);
        assert_eq!(p(&["a.docx", "b.xlsx"]), Cli::Open(vec!["a.docx".into(), "b.xlsx".into()]));
        assert_eq!(p(&["--", "--weird.docx"]), Cli::Open(vec!["--weird.docx".into()]));
        assert_eq!(p(&["--new", "broadside"]), Cli::New(AppKind::Broadside));
        assert_eq!(p(&["--with", "plunder-ledger", "x.txt"]), Cli::With(AppKind::PlunderLedger, vec!["x.txt".into()]));
        assert_eq!(p(&["--with", "captains-log"]), Cli::With(AppKind::CaptainsLog, vec![]));
        assert_eq!(p(&["--desktop-files", "/o"]), Cli::DesktopFiles { out: "/o".into(), launcher: None });
        assert_eq!(p(&["--desktop-files", "/o", "--launcher", "/l"]), Cli::DesktopFiles { out: "/o".into(), launcher: Some("/l".into()) });
        assert_eq!(p(&["--export-icons", "/i"]), Cli::ExportIcons("/i".into()));
        assert_eq!(p(&["-V"]), Cli::Version);
    }

    #[test]
    fn errors() {
        assert!(matches!(p(&["--new"]), Cli::Error(_)));
        assert!(matches!(p(&["--new", "photocraft"]), Cli::Error(_)));
        assert!(matches!(p(&["--new", "broadside", "x"]), Cli::Error(_)));
        assert!(matches!(p(&["--bogus"]), Cli::Error(_)));
        assert!(matches!(p(&["--export-icons"]), Cli::Error(_)));
        assert!(matches!(p(&["--desktop-files", "/o", "--x", "y"]), Cli::Error(_)));
    }
}
