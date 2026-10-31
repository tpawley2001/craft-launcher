//! Drives the start page against a fake host: what it shows, and what its buttons start.
//! `JOLLY_ROGER_SHOTS=<dir>` also saves screenshots of it (run with `-- --test-threads=1`, as
//! rendering on several threads at once can crash older Vulkan loaders).

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use jolly_roger_office_core::{AppKind, Config, RecentFile, ThemePref};
use jolly_roger_office_ui::{HomeUi, Host};

const NOW: i64 = 1_793_000_000;

#[derive(Default)]
struct Log {
    launched: Vec<(AppKind, Option<PathBuf>)>,
    recorded: Vec<(AppKind, PathBuf)>,
    saved: Vec<Config>,
}

struct Fake {
    log: Rc<RefCell<Log>>,
    recent: Vec<RecentFile>,
    missing: Option<AppKind>,
}

impl Host for Fake {
    fn launch(&mut self, app: AppKind, file: Option<&Path>, _config: &Config) -> Result<(), String> {
        if Some(app) == self.missing {
            return Err("not installed".into());
        }
        self.log.borrow_mut().launched.push((app, file.map(Path::to_path_buf)));
        Ok(())
    }
    fn app_available(&mut self, app: AppKind, _config: &Config) -> bool {
        Some(app) != self.missing
    }
    fn recent(&mut self, limit: usize) -> Vec<RecentFile> {
        self.recent.iter().take(limit).cloned().collect()
    }
    fn record(&mut self, app: AppKind, file: &Path) {
        self.log.borrow_mut().recorded.push((app, file.to_path_buf()));
    }
    fn save_config(&mut self, config: &Config) {
        self.log.borrow_mut().saved.push(config.clone());
    }
    fn now(&self) -> i64 {
        NOW
    }
}

fn sample_recent() -> Vec<RecentFile> {
    let r = |p: &str, app, ago: i64| RecentFile { path: PathBuf::from(p), app, opened: NOW - ago };
    vec![
        r("/home/captain/Voyages/Ship's log, autumn crossing.docx", AppKind::CaptainsLog, 240),
        r("/home/captain/Treasury/Plunder tally 1721.xlsx", AppKind::PlunderLedger, 3 * 3_600),
        r("/home/captain/Talks/Raising the crew's spirits.pptx", AppKind::Broadside, 26 * 3_600),
        r("/home/captain/Charts/Tortuga provisions.csv", AppKind::PlunderLedger, 4 * 86_400),
        r("/home/captain/Letters/To the Governor of Nassau.odt", AppKind::CaptainsLog, 16 * 86_400),
    ]
}

fn harness(config: Config, recent: Vec<RecentFile>, missing: Option<AppKind>) -> (Harness<'static, HomeUi>, Rc<RefCell<Log>>) {
    let log = Rc::new(RefCell::new(Log::default()));
    let mut fake = Fake { log: log.clone(), recent, missing };
    let theme = if config.theme == ThemePref::Dark { egui::Theme::Dark } else { egui::Theme::Light };
    let mut h = Harness::builder()
        .with_size(egui::vec2(1180.0, 860.0))
        .with_theme(theme)
        .with_render_cursor(false)
        .build_ui_state(move |ui, home: &mut HomeUi| home.ui(ui, &mut fake), HomeUi::new(config));
    h.run();
    (h, log)
}

fn shot(h: &mut Harness<'_, HomeUi>, name: &str) {
    let Some(dir) = std::env::var_os("JOLLY_ROGER_SHOTS") else { return };
    assert!(!name.contains(['/', '\\']));
    let img = h.render().unwrap();
    // The harness draws the UI inside an 8-point margin of its own; the window has none.
    let m = (8.0 * h.ctx.pixels_per_point()).round() as u32;
    let img = image::imageops::crop_imm(&img, m, m, img.width() - 2 * m, img.height() - 2 * m).to_image();
    img.save(Path::new(&dir).join(format!("{name}.png"))).unwrap();
}

#[test]
fn greets_the_captain_in_light_and_dark() {
    for (theme, name) in [(ThemePref::Light, "jolly-roger-start-light"), (ThemePref::Dark, "jolly-roger-start-dark")] {
        let (mut h, _) = harness(Config { theme, ..Config::default() }, sample_recent(), None);
        h.get_by_label("Ahoy, Captain");
        h.get_by_label("New Captain's Log");
        h.get_by_label("New Plunder Ledger");
        h.get_by_label("New Broadside");
        h.get_by_label("Based on the ArtCraft Crafting Apps (MIT OR Apache-2.0)");
        shot(&mut h, name);
    }
}

#[test]
fn empty_and_missing_states() {
    for (theme, name) in [(ThemePref::Light, "jolly-roger-empty-light"), (ThemePref::Dark, "jolly-roger-empty-dark")] {
        let (mut h, _) = harness(Config { theme, ..Config::default() }, Vec::new(), Some(AppKind::Broadside));
        h.get_by_label("The log be empty, Captain.");
        h.get_by_label("New Broadside");
        shot(&mut h, name);
    }
}

#[test]
fn plain_english_when_pirate_speak_is_off() {
    let (mut h, _) = harness(Config { pirate_speak: false, theme: ThemePref::Light, ..Config::default() }, sample_recent(), None);
    h.get_by_label("Welcome");
    assert!(h.query_by_label("Ahoy, Captain").is_none());
    shot(&mut h, "jolly-roger-plain-light");
}

#[test]
fn new_tiles_start_their_app() {
    let (mut h, log) = harness(Config { theme: ThemePref::Light, ..Config::default() }, Vec::new(), None);
    h.get_by_label("New Plunder Ledger").click();
    h.run();
    h.get_by_label("New Broadside").click();
    h.run();
    assert_eq!(log.borrow().launched, vec![(AppKind::PlunderLedger, None), (AppKind::Broadside, None)]);
    assert!(log.borrow().recorded.is_empty(), "nothing to record without a file");
}

#[test]
fn recent_rows_open_in_their_app() {
    let (mut h, log) = harness(Config { theme: ThemePref::Dark, ..Config::default() }, sample_recent(), None);
    h.get_by_label("Open Plunder tally 1721.xlsx in Plunder Ledger").click();
    h.run();
    let want = PathBuf::from("/home/captain/Treasury/Plunder tally 1721.xlsx");
    assert_eq!(log.borrow().launched, vec![(AppKind::PlunderLedger, Some(want.clone()))]);
    assert_eq!(log.borrow().recorded, vec![(AppKind::PlunderLedger, want)]);
}

#[test]
fn a_missing_app_says_so() {
    let (mut h, log) = harness(Config { theme: ThemePref::Light, ..Config::default() }, Vec::new(), Some(AppKind::CaptainsLog));
    h.get_by_label("New Captain's Log").click();
    h.run();
    h.get_by_label("not installed");
    assert!(log.borrow().launched.is_empty());
}

#[test]
fn pirate_speak_toggle_is_saved() {
    let (mut h, log) = harness(Config { theme: ThemePref::Light, ..Config::default() }, Vec::new(), None);
    h.get_by_label("Pirate speak").click();
    h.run();
    h.get_by_label("Welcome");
    let saved = log.borrow().saved.last().cloned().unwrap();
    assert!(!saved.pirate_speak);
}

#[test]
fn about_box_credits_the_upstream_apps() {
    let (mut h, _) = harness(Config { theme: ThemePref::Light, ..Config::default() }, sample_recent(), None);
    h.get_by_label("About").click();
    h.run();
    h.get_by_label(&format!("Version {}", env!("CARGO_PKG_VERSION")));
    shot(&mut h, "jolly-roger-about-light");
}

#[test]
fn settings_lists_each_apps_program() {
    let mut config = Config { theme: ThemePref::Dark, ..Config::default() };
    config.app_paths.insert(AppKind::PlunderLedger, PathBuf::from("/opt/ledger/plunder-ledger"));
    let (mut h, log) = harness(config, sample_recent(), None);
    h.get_by_label("Settings").click();
    h.run();
    h.get_by_label("Captain's Log program");
    shot(&mut h, "jolly-roger-settings-dark");
    h.get_by_label("Save").click();
    h.run();
    // Nothing changed, so nothing is written.
    assert!(log.borrow().saved.is_empty());
}
