//! The pirate palette (PIRATE_SPEC.md, shared by every app of the suite) and the fonts.

use std::sync::Arc;

use egui::epaint::text::VariationCoords;
use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontTweak, Stroke, Visuals};
use jolly_roger_office_core::ThemePref;

/// The colour tokens. Build UI from these; never hard-code a colour elsewhere.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub dark: bool,
    /// Window background.
    pub bg: Color32,
    /// Cards, fields and dialogs.
    pub panel: Color32,
    /// The top bar.
    pub ribbon: Color32,
    pub ribbon_text: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    /// Accent-coloured text on `bg` or `panel`: the brass, darkened in the light theme so small
    /// text stays readable on parchment.
    pub accent_text: Color32,
    /// Text drawn on an accent fill.
    pub on_accent: Color32,
    pub danger: Color32,
    pub text: Color32,
    /// Secondary text (folders, times, hints).
    pub muted: Color32,
    pub selection: Color32,
    pub border: Color32,
}

const fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

/// `c` at `alpha` (0–255), unmultiplied.
const fn with_alpha(hex: u32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied_const((hex >> 16) as u8, (hex >> 8) as u8, hex as u8, alpha)
}

/// Light: "Parchment".
pub const PARCHMENT: Palette = Palette {
    dark: false,
    bg: rgb(0xF4E9D0),
    panel: rgb(0xEADBB8),
    ribbon: rgb(0x1F3A4D),
    ribbon_text: rgb(0xF4E9D0),
    accent: rgb(0xB8860B),
    accent_hover: rgb(0xD4A017),
    accent_text: rgb(0x7F5C05),
    on_accent: rgb(0x2B1D0E),
    danger: rgb(0x8B1E1E),
    text: rgb(0x2B1D0E),
    muted: rgb(0x6B5536),
    selection: with_alpha(0xC9A227, 89),
    border: rgb(0x8C6D46),
};

/// Dark: "Midnight Sea". The spec leaves ribbon text, accent hover and selection open; they are
/// the dark text, a lighter brass and the accent at the light theme's 35 %.
pub const MIDNIGHT_SEA: Palette = Palette {
    dark: true,
    bg: rgb(0x0F1B24),
    panel: rgb(0x16262F),
    ribbon: rgb(0x0A141B),
    ribbon_text: rgb(0xE8DCC0),
    accent: rgb(0xD4A017),
    accent_hover: rgb(0xE6B83A),
    accent_text: rgb(0xD4A017),
    on_accent: rgb(0x0F1B24),
    danger: rgb(0xC0392B),
    text: rgb(0xE8DCC0),
    muted: rgb(0x9FA9A8),
    selection: with_alpha(0xD4A017, 89),
    border: rgb(0x3A4F5C),
};

impl Palette {
    pub fn for_dark(dark: bool) -> &'static Palette {
        if dark { &MIDNIGHT_SEA } else { &PARCHMENT }
    }

    /// The palette egui is drawing with now.
    pub fn of(ctx: &egui::Context) -> &'static Palette {
        Palette::for_dark(ctx.theme() == egui::Theme::Dark)
    }

    /// A card's fill when the pointer is over it.
    pub fn panel_hover(&self) -> Color32 {
        if self.dark { rgb(0x1D3240) } else { rgb(0xE3D0A6) }
    }
}

/// The serif face for headings and app names.
pub fn serif() -> FontFamily {
    FontFamily::Name("serif".into())
}

/// The semibold body face, for labels and buttons.
pub fn semibold() -> FontFamily {
    FontFamily::Name("semibold".into())
}

/// Register the suite's faces: Inter for body text, Source Serif 4 for headings.
pub fn install_fonts(ctx: &egui::Context) {
    const INTER: &[u8] = include_bytes!("../../../assets/fonts/Inter-Variable.ttf");
    const SERIF: &[u8] = include_bytes!("../../../assets/fonts/SourceSerif4-Semibold.ttf");
    let mut fonts = FontDefinitions::default();
    let fallback: Vec<String> = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let inter_semibold = FontTweak { coords: VariationCoords::new([(*b"wght", 600.0)]), ..Default::default() };
    fonts.font_data.insert("Inter".into(), Arc::new(FontData::from_static(INTER)));
    fonts.font_data.insert("Inter-SemiBold".into(), Arc::new(FontData::from_static(INTER).tweak(inter_semibold)));
    fonts.font_data.insert("SourceSerif4".into(), Arc::new(FontData::from_static(SERIF)));
    for (family, face) in [(FontFamily::Proportional, "Inter"), (semibold(), "Inter-SemiBold"), (serif(), "SourceSerif4")] {
        let mut stack = vec![face.to_string()];
        stack.extend(fallback.iter().cloned());
        fonts.families.insert(family, stack);
    }
    ctx.set_fonts(fonts);
}

/// Install both palettes and follow `pref` (egui switches between them with the desktop's
/// appearance when `pref` is [`ThemePref::System`]).
pub fn apply(ctx: &egui::Context, pref: ThemePref) {
    ctx.set_visuals_of(egui::Theme::Light, visuals(&PARCHMENT));
    ctx.set_visuals_of(egui::Theme::Dark, visuals(&MIDNIGHT_SEA));
    ctx.set_theme(match pref {
        ThemePref::System => egui::ThemePreference::System,
        ThemePref::Light => egui::ThemePreference::Light,
        ThemePref::Dark => egui::ThemePreference::Dark,
    });
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
    });
}

/// egui's visuals for `palette`.
pub fn visuals(palette: &Palette) -> Visuals {
    let p = palette;
    let mut v = if p.dark { Visuals::dark() } else { Visuals::light() };
    v.dark_mode = p.dark;
    v.override_text_color = Some(p.text);
    v.panel_fill = p.bg;
    v.window_fill = p.panel;
    v.window_stroke = Stroke::new(1.0, p.border);
    v.extreme_bg_color = if p.dark { rgb(0x0B151C) } else { rgb(0xFBF5E6) };
    v.faint_bg_color = p.panel;
    v.hyperlink_color = p.accent;
    v.warn_fg_color = p.accent;
    v.error_fg_color = p.danger;
    v.selection.bg_fill = p.selection;
    v.selection.stroke = Stroke::new(1.0, p.text);
    v.window_corner_radius = CornerRadius::same(6);
    v.menu_corner_radius = CornerRadius::same(4);
    let radius = CornerRadius::same(4);
    let w = &mut v.widgets;
    w.noninteractive.bg_fill = p.bg;
    w.noninteractive.weak_bg_fill = p.bg;
    w.noninteractive.bg_stroke = Stroke::new(1.0, p.border);
    w.noninteractive.fg_stroke = Stroke::new(1.0, p.text);
    w.inactive.bg_fill = p.panel;
    w.inactive.weak_bg_fill = p.panel;
    w.inactive.bg_stroke = Stroke::new(1.0, p.border);
    w.inactive.fg_stroke = Stroke::new(1.0, p.text);
    w.hovered.bg_fill = p.panel_hover();
    w.hovered.weak_bg_fill = p.panel_hover();
    w.hovered.bg_stroke = Stroke::new(1.0, p.accent);
    w.hovered.fg_stroke = Stroke::new(1.5, p.text);
    w.active.bg_fill = p.accent;
    w.active.weak_bg_fill = p.accent;
    w.active.bg_stroke = Stroke::new(1.0, p.accent_hover);
    w.active.fg_stroke = Stroke::new(1.5, p.on_accent);
    w.open = w.hovered;
    for s in [&mut w.noninteractive, &mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
        s.corner_radius = radius;
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_colours() {
        assert_eq!(PARCHMENT.bg, Color32::from_rgb(0xF4, 0xE9, 0xD0));
        assert_eq!(PARCHMENT.ribbon, Color32::from_rgb(0x1F, 0x3A, 0x4D));
        assert_eq!(PARCHMENT.accent, Color32::from_rgb(0xB8, 0x86, 0x0B));
        assert_eq!(PARCHMENT.danger, Color32::from_rgb(0x8B, 0x1E, 0x1E));
        assert_eq!(MIDNIGHT_SEA.bg, Color32::from_rgb(0x0F, 0x1B, 0x24));
        assert_eq!(MIDNIGHT_SEA.text, Color32::from_rgb(0xE8, 0xDC, 0xC0));
        assert_eq!(MIDNIGHT_SEA.border, Color32::from_rgb(0x3A, 0x4F, 0x5C));
    }
}
