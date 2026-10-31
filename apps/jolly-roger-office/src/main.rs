//! Jolly Roger Office: the suite's front door.
//!
//! With no arguments it opens the start page. Given files, it starts the right app for each and
//! exits (that is how the desktop's file associations reach it); see `--help` for the rest.
//! It never downloads anything: it only starts app binaries already on this computer.

#![cfg_attr(windows, windows_subsystem = "windows")]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod cli;

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use cli::Cli;
use jolly_roger_office_core::apps::{APPS, AppInfo, AppKind, route};
use jolly_roger_office_core::{Config, RecentFile, brand, config, desktop, launch, recent};
use jolly_roger_office_ui::{HomeUi, Host, Icon, icons};

fn main() -> ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    let dir = config::launcher_dir();
    let config = match &dir {
        Some(d) => {
            let (c, warning) = Config::load(d);
            if let Some(w) = warning {
                log::warn!("{w}");
            }
            c
        }
        None => Config::default(),
    };
    let mut host = NativeHost { dir, config_root: config::config_root() };
    match cli::parse(args) {
        Cli::Help => {
            println!("{}", cli::HELP);
            ExitCode::SUCCESS
        }
        Cli::Version => {
            println!("{} {}", brand::SUITE_ID, brand::VERSION);
            ExitCode::SUCCESS
        }
        Cli::Error(e) => {
            eprintln!("{}: {e}\n\n{}", brand::SUITE_ID, cli::HELP);
            ExitCode::from(2)
        }
        Cli::List => {
            for a in &APPS {
                match launch::resolve_here(a.kind, &config) {
                    Ok(p) => println!("{:<16} {}", a.name, p.display()),
                    Err(e) => println!("{:<16} (missing) {e}", a.name),
                }
            }
            ExitCode::SUCCESS
        }
        Cli::New(app) => report(host.launch(app, None, &config)),
        Cli::With(app, files) => {
            if files.is_empty() {
                return report(host.launch(app, None, &config));
            }
            let errors: Vec<String> = files.iter().filter_map(|f| open_one(&mut host, app, f, &config).err()).collect();
            finish(errors, config, host)
        }
        Cli::Open(files) => {
            let errors: Vec<String> = files
                .iter()
                .filter_map(|f| match route(f) {
                    Some(app) => open_one(&mut host, app, f, &config).err(),
                    None => Some(format!("None of the apps opens {}.", f.display())),
                })
                .collect();
            finish(errors, config, host)
        }
        Cli::ExportIcons(out) => report(export_icons(&out)),
        Cli::DesktopFiles { out, launcher } => {
            let launcher = launcher.or_else(|| std::env::current_exe().ok().map(|p| p.display().to_string()));
            match launcher {
                Some(l) => report(write_desktop_files(&out, &l)),
                None => report(Err("couldn't tell where the launcher is; pass --launcher <path>".into())),
            }
        }
        Cli::Window => run_window(config, host, Vec::new()),
    }
}

/// Exit quietly when every file opened; otherwise show the start page with what went wrong.
fn finish(errors: Vec<String>, config: Config, host: NativeHost) -> ExitCode {
    if errors.is_empty() {
        return ExitCode::SUCCESS;
    }
    for e in &errors {
        eprintln!("{e}");
    }
    run_window(config, host, errors)
}

fn report(result: Result<(), String>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// `file` as an absolute path, for the recent list and the app (which runs in another folder).
fn absolute(file: &Path) -> PathBuf {
    std::fs::canonicalize(file).unwrap_or_else(|_| std::env::current_dir().map(|d| d.join(file)).unwrap_or_else(|_| file.to_path_buf()))
}

fn open_one(host: &mut NativeHost, app: AppKind, file: &Path, config: &Config) -> Result<(), String> {
    let file = absolute(file);
    host.launch(app, Some(&file), config)?;
    host.record(app, &file);
    Ok(())
}

/// The desktop's side of the start page.
struct NativeHost {
    /// The launcher's settings folder.
    dir: Option<PathBuf>,
    /// Where the apps keep their settings (for their recent lists).
    config_root: Option<PathBuf>,
}

impl Host for NativeHost {
    fn launch(&mut self, app: AppKind, file: Option<&Path>, config: &Config) -> Result<(), String> {
        launch::launch(app, file, config)
    }

    fn app_available(&mut self, app: AppKind, config: &Config) -> bool {
        launch::resolve_here(app, config).is_ok()
    }

    fn pick_file(&mut self) -> Option<PathBuf> {
        let exts = jolly_roger_office_ui::home::open_filter();
        let mut d = rfd::FileDialog::new().set_title("Open").add_filter("Documents, workbooks and presentations", &exts);
        for a in &APPS {
            d = d.add_filter(a.name, a.extensions);
        }
        d.add_filter("All files", &["*"]).pick_file()
    }

    fn pick_program(&mut self, app: AppKind) -> Option<PathBuf> {
        rfd::FileDialog::new().set_title(format!("Where is {}?", AppInfo::of(app).name)).pick_file()
    }

    fn recent(&mut self, limit: usize) -> Vec<RecentFile> {
        recent::gather(self.dir.as_deref(), self.config_root.as_deref(), limit)
    }

    fn record(&mut self, app: AppKind, file: &Path) {
        if let Some(d) = &self.dir
            && let Err(e) = recent::record(d, file, app, recent::now())
        {
            log::warn!("{e}");
        }
    }

    fn save_config(&mut self, config: &Config) {
        match &self.dir {
            Some(d) => {
                if let Err(e) = config.save(d) {
                    log::warn!("{e}");
                }
            }
            None => log::warn!("no settings folder; the settings won't be kept"),
        }
    }

    fn now(&self) -> i64 {
        recent::now()
    }
}

struct Shell {
    home: HomeUi,
    host: NativeHost,
}

impl eframe::App for Shell {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.home.ui(ui, &mut self.host);
    }
}

fn run_window(config: Config, host: NativeHost, notices: Vec<String>) -> ExitCode {
    let mut home = HomeUi::new(config);
    if let Some(first) = notices.into_iter().next() {
        home.notice(first, true);
    }
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(brand::SUITE_NAME)
        .with_app_id(brand::APP_ID)
        .with_inner_size([1120.0, 780.0])
        .with_min_inner_size([760.0, 520.0])
        .with_drag_and_drop(true);
    match icons::rgba(Icon::Suite, 256) {
        Ok(rgba) => viewport = viewport.with_icon(egui::IconData { rgba, width: 256, height: 256 }),
        Err(e) => log::warn!("{e}"),
    }
    let options = eframe::NativeOptions { viewport, wgpu_options: within_adapter_limits(eframe::WgpuConfiguration::default()), ..Default::default() };
    let result = eframe::run_native(brand::SUITE_NAME, options, Box::new(move |_cc| Ok(Box::new(Shell { home, host }))));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}: couldn't open the window ({e})", brand::SUITE_NAME);
            ExitCode::FAILURE
        }
    }
}

/// Ask wgpu for no more than the GPU offers, so a modest adapter can't fail device creation.
fn within_adapter_limits(mut config: eframe::WgpuConfiguration) -> eframe::WgpuConfiguration {
    if let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &mut config.wgpu_setup {
        let egui_descriptor = Arc::clone(&setup.device_descriptor);
        setup.device_descriptor = Arc::new(move |adapter| {
            let mut descriptor = egui_descriptor(adapter);
            descriptor.required_limits = descriptor.required_limits.or_worse_values_from(&adapter.limits());
            descriptor
        });
    }
    config
}

/// Icon sizes written for the desktop's icon theme.
const ICON_SIZES: [u32; 8] = [16, 24, 32, 48, 64, 128, 256, 512];

/// Write every icon as `hicolor/<n>x<n>/apps/<name>.png` and `hicolor/scalable/apps/<name>.svg`
/// under `out`.
fn export_icons(out: &Path) -> Result<(), String> {
    for icon in Icon::ALL {
        let name = match icon {
            Icon::Suite => brand::APP_ID.to_string(),
            Icon::App(kind) => desktop::app_icon_name(AppInfo::of(kind)),
        };
        for px in ICON_SIZES {
            let dir = out.join(format!("hicolor/{px}x{px}/apps"));
            std::fs::create_dir_all(&dir).map_err(|e| format!("couldn't create {} ({e})", dir.display()))?;
            let path = dir.join(format!("{name}.png"));
            std::fs::write(&path, icons::png(icon, px)?).map_err(|e| format!("couldn't write {} ({e})", path.display()))?;
        }
        let dir = out.join("hicolor/scalable/apps");
        std::fs::create_dir_all(&dir).map_err(|e| format!("couldn't create {} ({e})", dir.display()))?;
        let path = dir.join(format!("{name}.svg"));
        std::fs::write(&path, icon.svg()).map_err(|e| format!("couldn't write {} ({e})", path.display()))?;
    }
    Ok(())
}

/// Write the menu entries to `out/applications/` and the MIME package to `out/mime/packages/`,
/// all running `launcher`.
fn write_desktop_files(out: &Path, launcher: &str) -> Result<(), String> {
    let apps_dir = out.join("applications");
    let mime_dir = out.join("mime/packages");
    for d in [&apps_dir, &mime_dir] {
        std::fs::create_dir_all(d).map_err(|e| format!("couldn't create {} ({e})", d.display()))?;
    }
    let mut files = vec![(apps_dir.join(desktop::suite_file_name()), desktop::suite_entry(launcher))];
    for a in &APPS {
        files.push((apps_dir.join(desktop::app_file_name(a)), desktop::app_entry(a, launcher)));
    }
    files.push((mime_dir.join(format!("{}.xml", brand::APP_ID)), desktop::mime_package()));
    for (path, text) in files {
        std::fs::write(&path, text).map_err(|e| format!("couldn't write {} ({e})", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_every_icon_size() {
        let d = tempfile::tempdir().unwrap();
        export_icons(d.path()).unwrap();
        for px in ICON_SIZES {
            assert!(d.path().join(format!("hicolor/{px}x{px}/apps/{}.png", brand::APP_ID)).is_file());
        }
        assert!(d.path().join("hicolor/scalable/apps/tech.hillyard.JollyRogerOffice.broadside.svg").is_file());
    }

    #[test]
    fn writes_desktop_files() {
        let d = tempfile::tempdir().unwrap();
        write_desktop_files(d.path(), "/opt/Jolly Roger/jolly-roger-office").unwrap();
        let suite = std::fs::read_to_string(d.path().join("applications").join(desktop::suite_file_name())).unwrap();
        assert!(suite.contains("Exec=\"/opt/Jolly Roger/jolly-roger-office\" %F"));
        for a in &APPS {
            assert!(d.path().join("applications").join(desktop::app_file_name(a)).is_file());
        }
        assert!(d.path().join("mime/packages/tech.hillyard.JollyRogerOffice.xml").is_file());
    }
}
