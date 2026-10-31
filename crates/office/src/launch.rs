//! Finding an app's binary on this computer and starting it. Never downloads anything.
//!
//! Where an app is looked for, first match wins:
//! 1. the path set for it in the settings (`appPaths`);
//! 2. the bundle: `apps/<id>` beside the launcher, then `<id>` beside the launcher;
//! 3. `<id>` (or another of its fork names) on `PATH`.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::apps::{AppInfo, AppKind};
use crate::config::Config;

/// The executable's file name for `stem` on this OS.
fn exe_name(stem: &str) -> String {
    if cfg!(windows) { format!("{stem}.exe") } else { stem.to_string() }
}

/// Is `p` a file that can be run?
pub fn is_runnable(p: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(p) else { return false };
    if !meta.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Where the bundle keeps `app` relative to the folder `exe_dir` the launcher runs from.
pub fn bundle_candidates(app: &AppInfo, exe_dir: &Path) -> Vec<PathBuf> {
    vec![exe_dir.join("apps").join(exe_name(app.id)), exe_dir.join(exe_name(app.id))]
}

/// The binary to run for `kind`, looking in the places listed in the module docs. `exe_dir` is
/// the launcher's own folder and `path_var` the `PATH` to search.
pub fn resolve(kind: AppKind, config: &Config, exe_dir: Option<&Path>, path_var: Option<OsString>) -> Result<PathBuf, String> {
    let app = AppInfo::of(kind);
    if let Some(p) = config.app_paths.get(&kind) {
        return if is_runnable(p) {
            Ok(p.clone())
        } else {
            Err(format!("{} is set to {}, which isn't a program that can be run. Change it in Settings.", app.name, p.display()))
        };
    }
    if let Some(dir) = exe_dir {
        for c in bundle_candidates(app, dir) {
            if is_runnable(&c) {
                return Ok(c);
            }
        }
    }
    if let Some(path_var) = path_var {
        for dir in std::env::split_paths(&path_var) {
            for name in app.binary_names {
                let c = dir.join(exe_name(name));
                if is_runnable(&c) {
                    return Ok(c);
                }
            }
        }
    }
    Err(format!("{} isn't installed next to the launcher. Set where it is in Settings.", app.name))
}

/// [`resolve`] for this process: the launcher's own folder and the real `PATH`.
pub fn resolve_here(kind: AppKind, config: &Config) -> Result<PathBuf, String> {
    let exe_dir = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf));
    resolve(kind, config, exe_dir.as_deref(), std::env::var_os("PATH"))
}

/// The command that starts `exe`, opening `file` if given. It runs in the file's folder (or the
/// home folder), with AppImage runtime variables removed so they can't leak into the app.
pub fn command(exe: &Path, file: Option<&Path>) -> Command {
    let mut cmd = Command::new(exe);
    if let Some(f) = file {
        cmd.arg(f);
    }
    let dir = file.and_then(Path::parent).filter(|d| d.is_dir()).map(Path::to_path_buf).or_else(|| std::env::var_os("HOME").map(PathBuf::from));
    if let Some(d) = dir {
        cmd.current_dir(d);
    }
    for var in ["APPDIR", "APPIMAGE", "ARGV0", "OWD", "LD_PRELOAD"] {
        cmd.env_remove(var);
    }
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    cmd
}

/// Start `kind` (opening `file`, if any) without waiting for it; the child is reaped on a
/// background thread when it exits.
pub fn launch(kind: AppKind, file: Option<&Path>, config: &Config) -> Result<(), String> {
    let app = AppInfo::of(kind);
    let exe = resolve_here(kind, config)?;
    let mut child = command(&exe, file).spawn().map_err(|e| format!("Couldn't start {} ({e}).", app.name))?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

#[cfg(test)]
#[cfg(unix)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn make_exe(p: &Path) {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, b"#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[test]
    fn bundle_layout_is_found() {
        let d = tempfile::tempdir().unwrap();
        make_exe(&d.path().join("apps/broadside"));
        let got = resolve(AppKind::Broadside, &Config::default(), Some(d.path()), None).unwrap();
        assert_eq!(got, d.path().join("apps/broadside"));
    }

    #[test]
    fn settings_path_wins_and_a_bad_one_is_an_error() {
        let d = tempfile::tempdir().unwrap();
        make_exe(&d.path().join("apps/captains-log"));
        make_exe(&d.path().join("custom/log"));
        let mut c = Config::default();
        c.app_paths.insert(AppKind::CaptainsLog, d.path().join("custom/log"));
        assert_eq!(resolve(AppKind::CaptainsLog, &c, Some(d.path()), None).unwrap(), d.path().join("custom/log"));
        c.app_paths.insert(AppKind::CaptainsLog, d.path().join("missing"));
        let e = resolve(AppKind::CaptainsLog, &c, Some(d.path()), None).unwrap_err();
        assert!(e.contains("Settings"), "{e}");
    }

    #[test]
    fn non_executable_files_are_skipped() {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(d.path().join("apps")).unwrap();
        std::fs::write(d.path().join("apps/plunder-ledger"), b"not a program").unwrap();
        assert!(resolve(AppKind::PlunderLedger, &Config::default(), Some(d.path()), None).is_err());
    }

    #[test]
    fn path_is_searched_for_fork_names_only() {
        let d = tempfile::tempdir().unwrap();
        make_exe(&d.path().join("bin/gridcraft"));
        let path = std::env::join_paths([d.path().join("bin")]).unwrap();
        assert!(resolve(AppKind::PlunderLedger, &Config::default(), None, Some(path.clone())).is_err(), "upstream names are never run");
        make_exe(&d.path().join("bin/plunder-ledger"));
        assert_eq!(resolve(AppKind::PlunderLedger, &Config::default(), None, Some(path)).unwrap(), d.path().join("bin/plunder-ledger"));
    }

    #[test]
    fn command_passes_the_file_and_runs_in_its_folder() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("chart.xlsx");
        std::fs::write(&f, b"").unwrap();
        let cmd = command(Path::new("/bin/true"), Some(&f));
        assert_eq!(cmd.get_args().collect::<Vec<_>>(), [f.as_os_str()]);
        assert_eq!(cmd.get_current_dir(), Some(d.path()));
    }

    #[test]
    fn launch_starts_a_real_process() {
        let d = tempfile::tempdir().unwrap();
        let exe = d.path().join("fake-broadside");
        make_exe(&exe);
        let mut c = Config::default();
        c.app_paths.insert(AppKind::Broadside, exe);
        // A program written a moment ago can still be busy ("text file busy") while another test
        // thread forks; try a few times.
        let mut last = Err(String::new());
        for _ in 0..20 {
            last = launch(AppKind::Broadside, None, &c);
            if last.is_ok() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        last.unwrap();
    }
}
