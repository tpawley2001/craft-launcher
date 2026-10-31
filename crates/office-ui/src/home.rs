//! The start page: greeting, a "New" tile per app, Open, and recent files across the suite.

use std::path::{Path, PathBuf};

use egui::{Align, Align2, Color32, CornerRadius, FontId, Layout, Margin, RichText, Sense, Stroke, StrokeKind, Vec2, vec2};
use jolly_roger_office_core::apps::{APPS, AppInfo, AppKind, all_extensions, route};
use jolly_roger_office_core::recent::ago;
use jolly_roger_office_core::{Config, RecentFile, ThemePref, brand};

use crate::icons::{Icon, IconCache};
use crate::theme::{self, Palette, semibold, serif};

/// What only the desktop shell can do. Tests supply a fake.
pub trait Host {
    /// Start `app`, opening `file` if given.
    fn launch(&mut self, app: AppKind, file: Option<&Path>, config: &Config) -> Result<(), String>;
    /// Whether `app`'s binary can be found with `config`.
    fn app_available(&mut self, app: AppKind, config: &Config) -> bool;
    /// Ask for a file to open; `None` when cancelled or when there is no dialog.
    fn pick_file(&mut self) -> Option<PathBuf> {
        None
    }
    /// Ask for an app's program file.
    fn pick_program(&mut self, _app: AppKind) -> Option<PathBuf> {
        None
    }
    /// The recent files across the suite, newest first, at most `limit`.
    fn recent(&mut self, limit: usize) -> Vec<RecentFile>;
    /// Remember that `file` was opened in `app`.
    fn record(&mut self, _app: AppKind, _file: &Path) {}
    /// Keep the settings.
    fn save_config(&mut self, _config: &Config) {}
    /// Now, in seconds since the Unix epoch.
    fn now(&self) -> i64;
}

/// How long the start page waits before looking at the recent files and apps again.
const REFRESH_SECS: f64 = 5.0;
/// How long a status message stays.
const STATUS_SECS: f64 = 6.0;
/// The content column's widest.
const MAX_W: f32 = 1000.0;

struct Status {
    text: String,
    error: bool,
    until: f64,
}

/// The start page's state.
pub struct HomeUi {
    pub config: Config,
    icons: IconCache,
    recent: Vec<RecentFile>,
    available: [bool; 3],
    next_refresh: f64,
    status: Option<Status>,
    settings_open: bool,
    about_open: bool,
    path_edits: [String; 3],
    applied_theme: Option<ThemePref>,
    fonts_ready: bool,
}

impl HomeUi {
    pub fn new(config: Config) -> Self {
        HomeUi {
            config,
            icons: IconCache::default(),
            recent: Vec::new(),
            available: [true; 3],
            next_refresh: f64::NEG_INFINITY,
            status: None,
            settings_open: false,
            about_open: false,
            path_edits: Default::default(),
            applied_theme: None,
            fonts_ready: false,
        }
    }

    /// Route `path` to its app and start it, recording it in the recent list.
    pub fn open_path(&mut self, path: &Path, host: &mut dyn Host, now: f64) {
        let Some(app) = route(path) else {
            self.say(format!("None of the apps opens {}.", file_label(path)), true, now);
            return;
        };
        self.launch(app, Some(path), host, now);
    }

    fn launch(&mut self, app: AppKind, file: Option<&Path>, host: &mut dyn Host, now: f64) {
        let info = AppInfo::of(app);
        match host.launch(app, file, &self.config) {
            Ok(()) => {
                if let Some(f) = file {
                    host.record(app, f);
                    self.say(format!("Opening {} in {}…", file_label(f), info.name), false, now);
                } else {
                    self.say(format!("Starting {}…", info.name), false, now);
                }
                self.next_refresh = now + 1.0;
            }
            Err(e) => self.say(e, true, now),
        }
    }

    fn say(&mut self, text: String, error: bool, now: f64) {
        if error {
            log::warn!("{text}");
        }
        self.status = Some(Status { text, error, until: now + STATUS_SECS * if error { 2.0 } else { 1.0 } });
    }

    fn refresh(&mut self, host: &mut dyn Host, now: f64) {
        if now < self.next_refresh {
            return;
        }
        self.next_refresh = now + REFRESH_SECS;
        self.recent = host.recent(self.config.recent_count);
        for (i, a) in APPS.iter().enumerate() {
            if let Some(slot) = self.available.get_mut(i) {
                *slot = host.app_available(a.kind, &self.config);
            }
        }
    }

    fn set_config(&mut self, config: Config, host: &mut dyn Host) {
        if config != self.config {
            self.config = config;
            host.save_config(&self.config);
            self.next_refresh = f64::NEG_INFINITY;
        }
    }

    /// Draw the start page.
    pub fn ui(&mut self, ui: &mut egui::Ui, host: &mut dyn Host) {
        let ctx = ui.ctx().clone();
        if !self.fonts_ready {
            // A face installed with `set_fonts` is usable from the next pass.
            theme::install_fonts(&ctx);
            self.fonts_ready = true;
            ctx.request_repaint();
            return;
        }
        if self.applied_theme != Some(self.config.theme) {
            theme::apply(&ctx, self.config.theme);
            self.applied_theme = Some(self.config.theme);
        }
        let now = ctx.input(|i| i.time);
        self.refresh(host, now);
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(REFRESH_SECS));
        if self.status.as_ref().is_some_and(|s| now > s.until) {
            self.status = None;
        }
        // Files dropped on the window, and Ctrl+O.
        let dropped: Vec<PathBuf> =
            ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).filter(|p| !p.as_os_str().is_empty()).collect());
        for p in dropped {
            self.open_path(&p, host, now);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::O)) {
            self.pick_and_open(host, now);
        }

        let p = Palette::of(&ctx);
        egui::Panel::top("ribbon")
            .frame(egui::Frame::new().fill(p.ribbon).inner_margin(Margin::symmetric(16, 0)))
            .exact_size(56.0)
            .resizable(false)
            .show(ui, |ui| self.ribbon(ui, host, p));
        egui::Panel::bottom("footer")
            .frame(egui::Frame::new().fill(p.panel).stroke(Stroke::new(1.0, p.border)).inner_margin(Margin::symmetric(16, 0)))
            .exact_size(32.0)
            .resizable(false)
            .show(ui, |ui| footer(ui, p));
        egui::CentralPanel::default().frame(egui::Frame::new().fill(p.bg).inner_margin(Margin { left: 24, right: 24, top: 24, bottom: 8 })).show(
            ui,
            |ui| {
                egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    let w = ui.available_width().min(MAX_W);
                    let pad = ((ui.available_width() - w) / 2.0).max(0.0);
                    ui.horizontal(|ui| {
                        ui.add_space(pad);
                        ui.vertical(|ui| {
                            ui.set_width(w);
                            self.page(ui, host, p, now);
                        });
                    });
                });
            },
        );
        self.settings_dialog(&ctx, host, p);
        self.about_dialog(&ctx, p);
    }

    fn pick_and_open(&mut self, host: &mut dyn Host, now: f64) {
        if let Some(path) = host.pick_file() {
            self.open_path(&path, host, now);
        }
    }

    fn ribbon(&mut self, ui: &mut egui::Ui, host: &mut dyn Host, p: &Palette) {
        ui.horizontal_centered(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::hover());
            self.icons.paint(ui, Icon::Suite, rect);
            ui.add_space(6.0);
            ui.label(RichText::new(brand::SUITE_NAME).family(serif()).size(21.0).color(p.ribbon_text));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ribbon_button(ui, "About", p).clicked() {
                    self.about_open = true;
                }
                if ribbon_button(ui, "Settings", p).clicked() {
                    self.path_edits = APPS.map(|a| self.config.app_paths.get(&a.kind).map(|p| p.display().to_string()).unwrap_or_default());
                    self.settings_open = true;
                }
                let (label, next) = match self.config.theme {
                    ThemePref::System => ("Theme: System", ThemePref::Light),
                    ThemePref::Light => ("Theme: Light", ThemePref::Dark),
                    ThemePref::Dark => ("Theme: Dark", ThemePref::System),
                };
                if ribbon_button(ui, label, p).clicked() {
                    let c = Config { theme: next, ..self.config.clone() };
                    self.set_config(c, host);
                }
                let mut pirate = self.config.pirate_speak;
                let toggle = ui.add(egui::Checkbox::new(&mut pirate, RichText::new("Pirate speak").color(p.ribbon_text)));
                if toggle.changed() {
                    let c = Config { pirate_speak: pirate, ..self.config.clone() };
                    self.set_config(c, host);
                }
            });
        });
    }

    fn page(&mut self, ui: &mut egui::Ui, host: &mut dyn Host, p: &Palette, now: f64) {
        let f = brand::flavour(self.config.pirate_speak);
        // Greeting.
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(92.0, 92.0), Sense::hover());
            self.icons.paint(ui, Icon::Suite, rect);
            ui.add_space(14.0);
            ui.vertical(|ui| {
                ui.add_space(12.0);
                ui.label(RichText::new(f.greeting).family(serif()).size(38.0).color(p.text));
                ui.label(RichText::new(f.subtitle).size(15.0).color(p.muted));
            });
        });
        ui.add_space(26.0);
        section_heading(ui, f.new_heading, p);
        ui.add_space(8.0);
        // The three New tiles.
        let gap = 16.0;
        let tile_w = ((ui.available_width() - 2.0 * gap) / 3.0).max(160.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (i, app) in APPS.iter().enumerate() {
                let available = self.available.get(i).copied().unwrap_or(true);
                if self.new_tile(ui, app, available, tile_w, p).clicked() {
                    self.launch(app.kind, None, host, now);
                }
            }
        });
        ui.add_space(18.0);
        ui.horizontal(|ui| {
            if primary_button(ui, "Open a file…", p).clicked() {
                self.pick_and_open(host, now);
            }
            ui.add_space(8.0);
            ui.label(
                RichText::new("Word, Excel, PowerPoint and OpenDocument files, CSV, Markdown and more open in the right app. Or drop files here.")
                    .size(13.0)
                    .color(p.muted),
            );
        });
        if let Some(s) = &self.status {
            ui.add_space(10.0);
            let color = if s.error { p.danger } else { p.accent_text };
            ui.label(RichText::new(&s.text).size(14.0).family(semibold()).color(color));
        }
        ui.add_space(26.0);
        section_heading(ui, f.recent_heading, p);
        ui.add_space(8.0);
        let mut open: Option<RecentFile> = None;
        egui::Frame::new().fill(p.panel).stroke(Stroke::new(1.0, p.border)).corner_radius(CornerRadius::same(8)).inner_margin(Margin::same(6)).show(
            ui,
            |ui| {
                ui.set_width(ui.available_width());
                if self.recent.is_empty() {
                    ui.add_space(18.0);
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new(f.recent_empty).family(serif()).size(18.0).color(p.text));
                        ui.label(RichText::new(f.recent_empty_hint).size(13.0).color(p.muted));
                    });
                    ui.add_space(18.0);
                } else {
                    let unix_now = host.now();
                    let rows = self.recent.clone();
                    for r in &rows {
                        if self.recent_row(ui, r, unix_now, p).clicked() {
                            open = Some(r.clone());
                        }
                    }
                }
            },
        );
        if let Some(r) = open {
            self.launch(r.app, Some(&r.path), host, now);
        }
        ui.add_space(16.0);
    }

    fn new_tile(&mut self, ui: &mut egui::Ui, app: &AppInfo, available: bool, w: f32, p: &Palette) -> egui::Response {
        let (rect, resp) = ui.allocate_exact_size(vec2(w, 132.0), Sense::click());
        let label = format!("New {}", app.name);
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &label));
        let hovered = resp.hovered();
        let painter = ui.painter_at(rect);
        let fill = if hovered { p.panel_hover() } else { p.panel };
        let stroke = Stroke::new(if hovered { 2.0 } else { 1.0 }, if hovered { p.accent } else { p.border });
        painter.rect(rect, CornerRadius::same(8), fill, stroke, StrokeKind::Inside);
        let icon = egui::Rect::from_min_size(rect.min + vec2(18.0, 18.0), Vec2::splat(72.0));
        self.icons.paint(ui, Icon::App(app.kind), icon);
        let x = icon.right() + 16.0;
        let text_w = (rect.right() - x - 12.0).max(40.0);
        painter.text(egui::pos2(x, rect.top() + 22.0), Align2::LEFT_TOP, app.name, FontId::new(23.0, serif()), p.text);
        painter.text(egui::pos2(x, rect.top() + 54.0), Align2::LEFT_TOP, app.generic, FontId::proportional(13.5), p.muted);
        let (line, color) = if available {
            (format!("{}  →", app.new_label(self.config.pirate_speak)), p.accent_text)
        } else {
            (brand::flavour(self.config.pirate_speak).app_missing.to_string(), p.danger)
        };
        let galley = painter.layout(line, FontId::new(14.0, semibold()), color, text_w);
        painter.galley(egui::pos2(x, rect.top() + 86.0), galley, color);
        if !available {
            painter.text(egui::pos2(x, rect.top() + 106.0), Align2::LEFT_TOP, "Set its path in Settings", FontId::proportional(12.0), p.muted);
        }
        resp.on_hover_cursor(egui::CursorIcon::PointingHand)
    }

    fn recent_row(&mut self, ui: &mut egui::Ui, r: &RecentFile, now: i64, p: &Palette) -> egui::Response {
        let w = ui.available_width();
        let (rect, resp) = ui.allocate_exact_size(vec2(w, 46.0), Sense::click());
        let name = r.file_name();
        let app = AppInfo::of(r.app);
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, format!("Open {} in {}", r.file_name(), app.name)));
        let painter = ui.painter_at(rect);
        if resp.hovered() {
            painter.rect_filled(rect, CornerRadius::same(5), p.panel_hover());
        }
        let icon = egui::Rect::from_min_size(rect.min + vec2(8.0, 7.0), Vec2::splat(32.0));
        self.icons.paint(ui, Icon::App(r.app), icon);
        let x = icon.right() + 12.0;
        let right_w = 260.0_f32.min(w * 0.4);
        let text_w = (rect.right() - right_w - x - 12.0).max(60.0);
        let name_g = one_line(&painter, name, FontId::new(14.5, semibold()), p.text, text_w);
        painter.galley(egui::pos2(x, rect.top() + 6.0), name_g, p.text);
        let folder_g = one_line(&painter, r.folder(), FontId::proportional(12.0), p.muted, text_w);
        painter.galley(egui::pos2(x, rect.top() + 26.0), folder_g, p.muted);
        painter.text(egui::pos2(rect.right() - right_w, rect.center().y), Align2::LEFT_CENTER, app.name, FontId::proportional(13.0), p.muted);
        painter.text(egui::pos2(rect.right() - 12.0, rect.center().y), Align2::RIGHT_CENTER, ago(r.opened, now), FontId::proportional(13.0), p.muted);
        resp.on_hover_text(r.path.display().to_string()).on_hover_cursor(egui::CursorIcon::PointingHand)
    }

    fn settings_dialog(&mut self, ctx: &egui::Context, host: &mut dyn Host, p: &Palette) {
        if !self.settings_open {
            return;
        }
        let mut close = false;
        let mut save = false;
        let modal = egui::Modal::new(egui::Id::unique("jro-settings")).frame(
            egui::Frame::new().fill(p.bg).stroke(Stroke::new(1.0, p.border)).corner_radius(CornerRadius::same(8)).inner_margin(Margin::same(20)),
        );
        let resp = modal.show(ctx, |ui| {
            ui.set_width(560.0_f32.min(ctx.content_rect().width() - 80.0));
            ui.label(RichText::new("Settings").family(serif()).size(24.0).color(p.text));
            ui.add_space(8.0);
            ui.label(
                RichText::new("Where each app's program is. Leave a box empty to use the copy that came with the suite.").size(13.0).color(p.muted),
            );
            ui.add_space(8.0);
            for (i, app) in APPS.iter().enumerate() {
                let Some(edit) = self.path_edits.get_mut(i) else { continue };
                ui.label(RichText::new(app.name).family(semibold()).color(p.text));
                ui.horizontal(|ui| {
                    let field = egui::TextEdit::singleline(edit).hint_text("Bundled copy").desired_width(ui.available_width() - 90.0);
                    ui.add(field).widget_info(|| egui::WidgetInfo::labeled(egui::Role::TextInput, true, format!("{} program", app.name)));
                    if ui.button("Browse…").clicked()
                        && let Some(path) = host.pick_program(app.kind)
                    {
                        *edit = path.display().to_string();
                    }
                });
                ui.add_space(4.0);
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if primary_button(ui, "Save", p).clicked() {
                    save = true;
                    close = true;
                }
                if ui.button("Cancel").clicked() {
                    close = true;
                }
            });
        });
        if resp.should_close() {
            close = true;
        }
        if save {
            let mut c = self.config.clone();
            c.app_paths.clear();
            for (i, app) in APPS.iter().enumerate() {
                if let Some(e) = self.path_edits.get(i).map(|e| e.trim()).filter(|e| !e.is_empty()) {
                    c.app_paths.insert(app.kind, PathBuf::from(e));
                }
            }
            self.set_config(c, host);
        }
        if close {
            self.settings_open = false;
        }
    }

    fn about_dialog(&mut self, ctx: &egui::Context, p: &Palette) {
        if !self.about_open {
            return;
        }
        let mut close = false;
        let modal = egui::Modal::new(egui::Id::unique("jro-about")).frame(
            egui::Frame::new().fill(p.bg).stroke(Stroke::new(1.0, p.border)).corner_radius(CornerRadius::same(8)).inner_margin(Margin::same(22)),
        );
        let resp = modal.show(ctx, |ui| {
            ui.set_width(460.0_f32.min(ctx.content_rect().width() - 80.0));
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(64.0, 64.0), Sense::hover());
                self.icons.paint(ui, Icon::Suite, rect);
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(brand::SUITE_NAME).family(serif()).size(26.0).color(p.text));
                    ui.label(RichText::new(format!("Version {}", brand::VERSION)).size(13.0).color(p.muted));
                });
            });
            ui.add_space(10.0);
            let apps: Vec<&str> = APPS.iter().map(|a| a.name).collect();
            ui.label(RichText::new(format!("{}: a word processor, a spreadsheet and a presentation app under one flag.", apps.join(", "))).color(p.text));
            ui.add_space(6.0);
            ui.label(RichText::new(brand::CREDIT).color(p.text));
            ui.label(RichText::new("Captain's Log, Plunder Ledger and Broadside are based on WordCraft, GridCraft and DeckCraft by the ArtCraft team (MIT OR Apache-2.0).").size(13.0).color(p.muted));
            ui.label(RichText::new(format!("This build: {}. Licensed MIT OR Apache-2.0; see LICENSE-MIT, LICENSE-APACHE, NOTICE and ATTRIBUTION.md.", brand::PUBLISHER)).size(13.0).color(p.muted));
            ui.add_space(12.0);
            if primary_button(ui, "Close", p).clicked() {
                close = true;
            }
        });
        if close || resp.should_close() {
            self.about_open = false;
        }
    }

    /// Show `text` under the Open button when the page first appears (for example, a file the
    /// command line couldn't open).
    pub fn notice(&mut self, text: String, error: bool) {
        self.say(text, error, 0.0);
    }
}

/// A file's name for messages.
fn file_label(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| path.display().to_string())
}

/// `text` on one line, cut to `max_w` with an ellipsis when it is wider.
fn one_line(painter: &egui::Painter, text: String, font: FontId, color: Color32, max_w: f32) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple_singleline(text, font, color);
    job.wrap = egui::text::TextWrapping::truncate_at_width(max_w.max(0.0));
    painter.layout_job(job)
}

fn section_heading(ui: &mut egui::Ui, text: &str, p: &Palette) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(text).family(serif()).size(21.0).color(p.text));
        let rest = ui.available_rect_before_wrap();
        let y = rest.center().y + 2.0;
        ui.painter().hline(rest.left() + 6.0..=rest.right(), y, Stroke::new(1.0, p.border));
    });
}

fn ribbon_button(ui: &mut egui::Ui, text: &str, p: &Palette) -> egui::Response {
    let button = egui::Button::new(RichText::new(text).color(p.ribbon_text).size(13.5))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0, p.ribbon_text.gamma_multiply(0.45)))
        .corner_radius(CornerRadius::same(4));
    ui.add(button)
}

fn primary_button(ui: &mut egui::Ui, text: &str, p: &Palette) -> egui::Response {
    let button = egui::Button::new(RichText::new(text).family(semibold()).size(14.0).color(p.on_accent))
        .fill(p.accent)
        .stroke(Stroke::new(1.0, p.accent_hover))
        .corner_radius(CornerRadius::same(5))
        .min_size(vec2(0.0, 32.0));
    ui.add(button)
}

fn footer(ui: &mut egui::Ui, p: &Palette) {
    ui.horizontal_centered(|ui| {
        ui.label(RichText::new(brand::CREDIT).size(12.0).color(p.muted));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(format!("{} {}", brand::SUITE_NAME, brand::VERSION)).size(12.0).color(p.muted));
        });
    });
}

/// The Open dialog's filter: every extension the suite opens.
pub fn open_filter() -> Vec<&'static str> {
    all_extensions()
}
