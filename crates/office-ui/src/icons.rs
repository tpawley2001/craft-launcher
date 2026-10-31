//! The suite's icons, drawn in code: each is an SVG document built here (no image files), drawn
//! by resvg for the window icon, the start page and the bundle's PNGs.
//!
//! - Suite: skull & crossbones over a brass compass rose, on deep-sea navy.
//! - Captain's Log: an open logbook with a quill, on sea blue.
//! - Plunder Ledger: a treasure chest in front of a ledger grid, on bottle green.
//! - Broadside: a ship's cannon firing at a slide, on oxblood.

use std::collections::HashMap;
use std::fmt::Write as _;

use jolly_roger_office_core::AppKind;

/// One of the icons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    Suite,
    App(AppKind),
}

impl Icon {
    /// Every icon, suite first.
    pub const ALL: [Icon; 4] = [Icon::Suite, Icon::App(AppKind::CaptainsLog), Icon::App(AppKind::PlunderLedger), Icon::App(AppKind::Broadside)];

    /// The icon's SVG document (256 × 256 user units).
    pub fn svg(self) -> String {
        match self {
            Icon::Suite => suite_svg(),
            Icon::App(AppKind::CaptainsLog) => captains_log_svg(),
            Icon::App(AppKind::PlunderLedger) => plunder_ledger_svg(),
            Icon::App(AppKind::Broadside) => broadside_svg(),
        }
    }
}

const INK: &str = "#2B1D0E";
const PARCHMENT: &str = "#F4E9D0";
const PARCHMENT_SHADE: &str = "#E2CFA4";
const BRASS: &str = "#D4A017";
const BRASS_DARK: &str = "#8C6A0A";
const WOOD: &str = "#7A4A1E";
const WOOD_LIGHT: &str = "#9A6430";

/// The rounded-square tile every icon sits on, with a radial gradient from `light` to `dark` and
/// a brass inner rim.
fn tile(light: &str, dark: &str) -> String {
    format!(
        r##"<defs><radialGradient id="bg" cx="0.38" cy="0.3" r="0.9"><stop offset="0" stop-color="{light}"/><stop offset="1" stop-color="{dark}"/></radialGradient></defs><rect x="8" y="8" width="240" height="240" rx="52" fill="url(#bg)"/><rect x="15" y="15" width="226" height="226" rx="45" fill="none" stroke="{BRASS}" stroke-opacity="0.55" stroke-width="3"/>"##
    )
}

fn doc(body: &str) -> String {
    format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256">{body}</svg>"#)
}

/// A star point from the centre (128, 128): tip at `r` in direction `deg` (0 = north, clockwise),
/// shoulders at `w` from the centre, 45° either side; the two halves in `left` and `right`.
fn rose_point(out: &mut String, deg: f32, r: f32, w: f32, left: &str, right: &str) {
    let at = |a: f32, len: f32| {
        let rad = a.to_radians();
        (128.0 + len * rad.sin(), 128.0 - len * rad.cos())
    };
    let (tx, ty) = at(deg, r);
    let (lx, ly) = at(deg - 45.0, w);
    let (rx, ry) = at(deg + 45.0, w);
    let _ = write!(out, r#"<polygon points="{tx:.1},{ty:.1} {lx:.1},{ly:.1} 128,128" fill="{left}"/>"#);
    let _ = write!(out, r#"<polygon points="{tx:.1},{ty:.1} {rx:.1},{ry:.1} 128,128" fill="{right}"/>"#);
}

/// A bone from (x1, y1) to (x2, y2): a thick shaft with two knuckles at each end. Drawn twice by
/// the caller: once as an ink outline, then filled.
fn bone(out: &mut String, (x1, y1): (f32, f32), (x2, y2): (f32, f32), fill: &str, grow: f32) {
    let (dx, dy) = (x2 - x1, y2 - y1);
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    let (nx, ny) = (-dy / len * 7.5, dx / len * 7.5);
    let _ = write!(
        out,
        r#"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="{fill}" stroke-width="{:.1}" stroke-linecap="round"/>"#,
        13.0 + 2.0 * grow
    );
    for (x, y) in [(x1, y1), (x2, y2)] {
        for s in [-1.0, 1.0] {
            let _ = write!(out, r#"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="{fill}"/>"#, x + s * nx, y + s * ny, 9.0 + grow);
        }
    }
}

/// Skull & crossbones over a compass rose.
pub fn suite_svg() -> String {
    let mut s = tile("#2E5670", "#10222E");
    // The compass ring and its ticks.
    let _ = write!(
        s,
        r#"<circle cx="128" cy="128" r="98" fill="none" stroke="{BRASS}" stroke-width="3"/><circle cx="128" cy="128" r="90" fill="none" stroke="{BRASS}" stroke-opacity="0.7" stroke-width="7" stroke-dasharray="1.6 6.25"/>"#
    );
    // The rose: four short points between four long ones.
    for deg in [45.0, 135.0, 225.0, 315.0] {
        rose_point(&mut s, deg, 74.0, 16.0, PARCHMENT_SHADE, "#B59C6B");
    }
    for deg in [0.0, 90.0, 180.0, 270.0] {
        rose_point(&mut s, deg, 114.0, 22.0, BRASS, BRASS_DARK);
    }
    // Crossbones, crossing under the jaw: ink outlines first, then the bone fill.
    let a = ((84.0, 116.0), (172.0, 204.0));
    let b = ((172.0, 116.0), (84.0, 204.0));
    for (fill, grow) in [(INK, 3.0), (PARCHMENT, 0.0)] {
        bone(&mut s, a.0, a.1, fill, grow);
        bone(&mut s, b.0, b.1, fill, grow);
    }
    // The skull: cranium and jaw, outlined as one shape by drawing them in ink a little larger first.
    for (fill, grow) in [(INK, 3.5_f32), (PARCHMENT, 0.0)] {
        let _ = write!(
            s,
            r#"<circle cx="128" cy="104" r="{:.1}" fill="{fill}"/><rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="{:.1}" fill="{fill}"/>"#,
            40.0 + grow,
            103.0 - grow,
            118.0 - grow,
            50.0 + 2.0 * grow,
            44.0 + 2.0 * grow,
            12.0 + grow
        );
    }
    // Eyes, nose and teeth.
    let _ = write!(
        s,
        r##"<ellipse cx="112" cy="106" rx="11.5" ry="12.5" fill="#132633"/><ellipse cx="144" cy="106" rx="11.5" ry="12.5" fill="#132633"/><path d="M128 120 L121.5 134 Q128 137 134.5 134 Z" fill="#132633"/><g stroke="#132633" stroke-width="2.6" stroke-linecap="round"><line x1="116" y1="146" x2="116" y2="157"/><line x1="124" y1="147" x2="124" y2="158"/><line x1="132" y1="147" x2="132" y2="158"/><line x1="140" y1="146" x2="140" y2="157"/></g>"##
    );
    doc(&s)
}

/// An open logbook with a quill.
pub fn captains_log_svg() -> String {
    let mut s = tile("#3A7596", "#163A4F");
    let _ = write!(
        s,
        // Cover, then the two pages, then the ruled lines.
        r##"<polygon points="30,82 128,96 226,82 226,196 128,212 30,196" fill="#8B1E1E" stroke="{INK}" stroke-width="3" stroke-linejoin="round"/>
<polygon points="40,78 124,92 124,198 40,184" fill="{PARCHMENT}" stroke="{INK}" stroke-width="2.5" stroke-linejoin="round"/>
<polygon points="132,92 216,78 216,184 132,198" fill="{PARCHMENT_SHADE}" stroke="{INK}" stroke-width="2.5" stroke-linejoin="round"/>
<g stroke="{INK}" stroke-opacity="0.45" stroke-width="2.2" stroke-linecap="round">
<line x1="52" y1="104" x2="112" y2="113"/><line x1="52" y1="120" x2="112" y2="129"/><line x1="52" y1="136" x2="112" y2="145"/><line x1="52" y1="152" x2="100" y2="159"/>
<line x1="144" y1="113" x2="176" y2="108"/><line x1="144" y1="129" x2="168" y2="125"/><line x1="144" y1="145" x2="160" y2="142"/></g>
<path d="M214 30 C 186 40 150 84 136 160 C 160 120 196 84 214 30 Z" fill="#FFFBF0" stroke="{INK}" stroke-width="2.5" stroke-linejoin="round"/>
<path d="M214 30 C 186 64 158 108 136 160" fill="none" stroke="{INK}" stroke-width="2"/>
<line x1="136" y1="160" x2="128" y2="184" stroke="{INK}" stroke-width="4" stroke-linecap="round"/>
<circle cx="126" cy="190" r="3.5" fill="{INK}"/>"##
    );
    doc(&s)
}

/// A treasure chest in front of a ledger grid.
pub fn plunder_ledger_svg() -> String {
    let mut s = tile("#3C8A5E", "#173826");
    // The ledger sheet and its grid.
    let _ = write!(s, r#"<rect x="50" y="34" width="156" height="120" rx="5" fill="{PARCHMENT}" stroke="{INK}" stroke-width="2.5"/>"#);
    let _ = write!(s, r#"<rect x="50" y="34" width="156" height="20" rx="5" fill="{PARCHMENT_SHADE}"/>"#);
    let _ = write!(s, r#"<g stroke="{INK}" stroke-opacity="0.5" stroke-width="2">"#);
    for x in [89.0, 128.0, 167.0] {
        let _ = write!(s, r#"<line x1="{x}" y1="34" x2="{x}" y2="154"/>"#);
    }
    for y in [54.0, 74.0, 94.0, 114.0] {
        let _ = write!(s, r#"<line x1="50" y1="{y}" x2="206" y2="{y}"/>"#);
    }
    s.push_str("</g>");
    // Gold coins spilling over the sheet.
    for (cx, cy) in [(70.0, 128.0), (186.0, 124.0), (172.0, 132.0)] {
        let _ = write!(s, r#"<circle cx="{cx}" cy="{cy}" r="11" fill="{BRASS}" stroke="{BRASS_DARK}" stroke-width="2.5"/>"#);
    }
    let _ = write!(
        s,
        // Lid, body, brass bands and rim, lock plate with keyhole.
        r##"<path d="M44 158 L44 146 Q128 106 212 146 L212 158 Z" fill="{WOOD_LIGHT}" stroke="{INK}" stroke-width="3" stroke-linejoin="round"/>
<rect x="44" y="156" width="168" height="62" rx="6" fill="{WOOD}" stroke="{INK}" stroke-width="3"/>
<rect x="44" y="153" width="168" height="9" fill="{BRASS}" stroke="{INK}" stroke-width="2"/>
<rect x="68" y="124" width="11" height="94" fill="{BRASS}" stroke="{INK}" stroke-width="2"/>
<rect x="177" y="124" width="11" height="94" fill="{BRASS}" stroke="{INK}" stroke-width="2"/>
<rect x="114" y="150" width="28" height="34" rx="4" fill="{BRASS}" stroke="{INK}" stroke-width="2.5"/>
<circle cx="128" cy="164" r="4.5" fill="{INK}"/><rect x="126" y="166" width="4" height="10" fill="{INK}"/>"##
    );
    doc(&s)
}

/// A ship's cannon firing at a slide.
pub fn broadside_svg() -> String {
    let mut s = tile("#A63A32", "#4E1010");
    let _ = write!(
        s,
        // The slide: title rule and a little bar chart.
        r##"<rect x="40" y="34" width="176" height="112" rx="6" fill="{PARCHMENT}" stroke="{INK}" stroke-width="2.5"/>
<line x1="58" y1="56" x2="150" y2="56" stroke="{INK}" stroke-width="7" stroke-linecap="round"/>
<line x1="58" y1="74" x2="118" y2="74" stroke="{INK}" stroke-opacity="0.45" stroke-width="4" stroke-linecap="round"/>
<rect x="138" y="100" width="16" height="32" fill="#1F3A4D"/><rect x="160" y="84" width="16" height="48" fill="{BRASS}"/><rect x="182" y="70" width="16" height="62" fill="#8B1E1E"/>
<line x1="132" y1="132" x2="204" y2="132" stroke="{INK}" stroke-width="2.5"/>"##
    );
    let _ = write!(
        s,
        // Smoke at the muzzle, then the barrel (tilted up toward the slide), the carriage and the wheel.
        r##"<g fill="#FFFDF7" fill-opacity="0.92" stroke="{INK}" stroke-opacity="0.35" stroke-width="2"><circle cx="190" cy="152" r="15"/><circle cx="210" cy="140" r="11"/><circle cx="206" cy="166" r="9"/></g>
<g transform="rotate(-20 110 186)">
<rect x="56" y="172" width="122" height="28" rx="10" fill="#2E2E33" stroke="{INK}" stroke-width="3"/>
<circle cx="56" cy="186" r="15" fill="#2E2E33" stroke="{INK}" stroke-width="3"/>
<rect x="170" y="167" width="14" height="38" rx="4" fill="#3E3E45" stroke="{INK}" stroke-width="3"/>
<rect x="96" y="168" width="9" height="36" fill="{BRASS}" stroke="{INK}" stroke-width="2"/>
<line x1="66" y1="178" x2="166" y2="178" stroke="#7A7A85" stroke-width="3" stroke-linecap="round"/>
</g>
<path d="M58 222 L76 196 L150 196 L162 222 Z" fill="{WOOD_LIGHT}" stroke="{INK}" stroke-width="3" stroke-linejoin="round"/>
<circle cx="96" cy="214" r="20" fill="{WOOD}" stroke="{INK}" stroke-width="3"/>
<g stroke="{INK}" stroke-width="2.5"><line x1="96" y1="196" x2="96" y2="232"/><line x1="78" y1="214" x2="114" y2="214"/><line x1="83" y1="201" x2="109" y2="227"/><line x1="109" y1="201" x2="83" y2="227"/></g>
<circle cx="96" cy="214" r="5" fill="{BRASS}" stroke="{INK}" stroke-width="2"/>"##
    );
    doc(&s)
}

/// `icon` drawn at `px` × `px`, as unmultiplied RGBA bytes.
pub fn rgba(icon: Icon, px: u32) -> Result<Vec<u8>, String> {
    let svg = icon.svg();
    let tree = resvg::usvg::Tree::from_str(&svg, &resvg::usvg::Options::default()).map_err(|e| format!("icon SVG is invalid ({e})"))?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(px, px).ok_or_else(|| format!("can't draw an icon {px} px wide"))?;
    let scale = px as f32 / tree.size().width().max(1.0);
    resvg::render(&tree, resvg::tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    Ok(pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect())
}

/// `icon` as a PNG file, `px` × `px`.
pub fn png(icon: Icon, px: u32) -> Result<Vec<u8>, String> {
    let bytes = rgba(icon, px)?;
    let img = image::RgbaImage::from_raw(px, px, bytes).ok_or("icon buffer has the wrong size")?;
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).map_err(|e| format!("couldn't encode the icon ({e})"))?;
    Ok(out.into_inner())
}

/// Textures of the icons, drawn once per size.
#[derive(Default)]
pub struct IconCache {
    textures: HashMap<(Icon, u32), egui::TextureHandle>,
}

impl IconCache {
    /// `icon` as a texture for `points` × `points` on screen (drawn at the screen's pixel density).
    pub fn texture(&mut self, ctx: &egui::Context, icon: Icon, points: f32) -> Option<egui::TextureHandle> {
        let px = ((points * ctx.pixels_per_point()).round() as u32).clamp(8, 1024);
        if let Some(t) = self.textures.get(&(icon, px)) {
            return Some(t.clone());
        }
        let bytes = match rgba(icon, px) {
            Ok(b) => b,
            Err(e) => {
                log::warn!("{e}");
                return None;
            }
        };
        let image = egui::ColorImage::from_rgba_unmultiplied([px as usize, px as usize], &bytes);
        let t = ctx.load_texture(format!("jro-icon-{icon:?}-{px}"), image, egui::TextureOptions::LINEAR);
        self.textures.insert((icon, px), t.clone());
        Some(t)
    }

    /// Draw `icon` filling `rect`.
    pub fn paint(&mut self, ui: &egui::Ui, icon: Icon, rect: egui::Rect) {
        if let Some(t) = self.texture(ui.ctx(), icon, rect.width()) {
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            ui.painter().image(t.id(), rect, uv, egui::Color32::WHITE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_draws() {
        for icon in Icon::ALL {
            let px = rgba(icon, 64).unwrap();
            assert_eq!(px.len(), 64 * 64 * 4);
            // Opaque in the middle, transparent in the corner (the tile is rounded).
            let mid = (32 * 64 + 32) * 4 + 3;
            assert_eq!(px[mid], 255, "{icon:?}");
            assert_eq!(px[3], 0, "{icon:?}");
        }
    }

    #[test]
    fn png_has_a_png_header() {
        let p = png(Icon::Suite, 32).unwrap();
        assert_eq!(&p[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn icons_are_distinct() {
        let all: Vec<Vec<u8>> = Icon::ALL.iter().map(|i| rgba(*i, 32).unwrap()).collect();
        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                assert_ne!(all[i], all[j]);
            }
        }
    }
}
