# Attribution

Every non-code asset in this repository, with its author, source and license. Jolly Roger Office's
own code and original assets are MIT OR Apache-2.0 (see [`LICENSE-MIT`](LICENSE-MIT),
[`LICENSE-APACHE`](LICENSE-APACHE) and [`NOTICE`](NOTICE)). Based on the ArtCraft Crafting Apps
(MIT OR Apache-2.0): this launcher is a fork of ArtCraft Launcher by the ArtCraft Team and its
contributors. When you add an asset, add a row here in the same change; third-party assets must be
permissively licensed and keep their license file next to them.

## Bundled in the app

| Path | Title | Author | Source | License |
|---|---|---|---|---|
| `crates/office-ui/src/icons.rs` (generated: `--export-icons`) | Suite icon (skull & crossbones over a compass rose) and the Captain's Log (logbook and quill), Plunder Ledger (treasure chest and ledger grid) and Broadside (cannon and slide) icons | Hillyard Tech | Original work, drawn in code as SVG built in Rust and rendered by resvg; no image files | MIT OR Apache-2.0 |
| `assets/fonts/Inter-Variable.ttf` | Inter 4.001, variable (body text) | The Inter Project Authors (Rasmus Andersson) | Google Fonts, <https://github.com/google/fonts/tree/main/ofl/inter> (`Inter[opsz,wght].ttf`, renamed) | SIL OFL 1.1, [`assets/fonts/OFL-Inter.txt`](assets/fonts/OFL-Inter.txt) |
| `assets/fonts/SourceSerif4-Semibold.ttf` | Source Serif 4 Semibold (headings and app names) | Frank Grießhammer, Adobe | <https://github.com/adobe-fonts/source-serif>, copied unmodified from the WordCraft fork's `assets/fonts/` | SIL OFL 1.1, [`assets/fonts/OFL-SourceSerif4.txt`](assets/fonts/OFL-SourceSerif4.txt) |
| (dependency) `epaint_default_fonts/fonts/NotoEmoji-Regular.ttf` | Noto Emoji Regular, monochrome emoji fallback for arbitrary file names | The Noto Project Authors (Google) | Bundled transitively by `egui 0.37`'s `monochrome_emoji_fonts` feature from `epaint_default_fonts`; <https://github.com/storytold/egui/tree/6d518fc799681571963842f95b55a6b68431b15a/crates/epaint_default_fonts/fonts> | SIL OFL 1.1, `epaint_default_fonts/fonts/OFL.txt`; see also the dependency `README` and egui `README.md` |
| (dependency) `epaint_default_fonts/fonts/emoji-icon-font.ttf` | emoji-icon-font, monochrome symbol/emoji fallback | John Slegers | Bundled transitively by `egui 0.37`'s `monochrome_emoji_fonts` feature from `epaint_default_fonts`; <https://github.com/jslegers/emoji-icon-font> | MIT, Copyright (c) 2014 John Slegers, `epaint_default_fonts/fonts/emoji-icon-font-mit-license.txt` |
| (dependency) `epaint_default_fonts/fonts/egui-icons.ttf` | egui-icons, five-glyph subset of emoji-icon-font | John Slegers | [egui 6d518fc7](https://github.com/storytold/egui/tree/6d518fc799681571963842f95b55a6b68431b15a/crates/epaint_default_fonts/fonts); included by default_fonts independently of monochrome_emoji_fonts | MIT, Copyright (c) 2014 John Slegers; full upstream text in `NOTICE` |
| (dependency) `epaint_default_fonts/fonts/Ubuntu-Light.ttf` | Ubuntu Light, default font fallback | Canonical Ltd. | [Pinned egui source](https://github.com/storytold/egui/tree/6d518fc799681571963842f95b55a6b68431b15a/crates/epaint_default_fonts/fonts) | Ubuntu Font Licence 1.0; full upstream text and copyright in `NOTICE` |
| (dependency) `epaint_default_fonts/fonts/Hack-Regular.ttf` | Hack Regular, default monospace fallback | Source Foundry Authors; Bitstream Inc.; DejaVu contributors | [Pinned egui source](https://github.com/storytold/egui/tree/6d518fc799681571963842f95b55a6b68431b15a/crates/epaint_default_fonts/fonts) | MIT and Bitstream Vera License; DejaVu work public domain; full upstream notices in `NOTICE` |
