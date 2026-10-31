# AGENTS.md: guide for AI agents and contributors

This repository is the launcher and start page of **Jolly Roger Office**, a pirate-themed office
suite of three apps: **Captain's Log** (word processor, a WordCraft fork), **Plunder Ledger**
(spreadsheet, a GridCraft fork) and **Broadside** (presentations, a DeckCraft fork). It is a
local fork of ArtCraft Launcher (`storytold/craft-launcher`); the shared spec for the whole suite
is `../PIRATE_SPEC.md`. Rust only: native egui/eframe, no webview shells.

Names are placeholders the owner may change: the suite's live in `crates/office/src/brand.rs`,
the apps' in `crates/office/src/apps.rs`. Never spell them out anywhere else in code.

## 1. Workspace map

```text
crates/
  office     jolly-roger-office-core   brand, the three apps and their file types, routing,
                                       settings, recent files, finding and starting app binaries,
                                       desktop entries
  office-ui  jolly-roger-office-ui     the egui start page, the pirate palette (theme.rs), the
                                       icons drawn in code (icons.rs)
apps/
  jolly-roger-office                   desktop shell (eframe) and command line
xtask/                                 cargo xtask ci | wasm | version | ico | icns
assets/fonts/                          Inter, Source Serif 4 (OFL-1.1)
```

## 2. Golden rules

- **Local only.** Never push, never add remotes, never open upstream PRs or issues, never publish.
- **Nothing is downloaded.** The launcher starts binaries already on this computer (settings
  path, the bundle's `apps/<id>`, or the fork's name on `PATH`) and never fetches apps, catalogs,
  updates or pictures. Don't add an HTTP client.
- **Never crash.** Non-test code never panics: no `unwrap()`, `expect()`, `panic!`,
  `unreachable!`, `todo!` or `unimplemented!`. Every production crate root carries
  `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]`.
  No `unsafe` (`unsafe_code = "forbid"`). Settings and the apps' `ui.json` files are untrusted:
  cap their size and fall back to defaults when they don't parse.
- **Trademarks.** No ArtCraft or Craft names, logos or links in the UI, icons, window titles or
  menu entries. The one exception is the credit line, `brand::CREDIT`: "Based on the ArtCraft
  Crafting Apps (MIT OR Apache-2.0)", shown in the footer and About. Keep `LICENSE-MIT`,
  `LICENSE-APACHE`, `NOTICE` (with upstream's copyright line) and `ATTRIBUTION.md`.
- **Assets.** Every bundled asset gets an `ATTRIBUTION.md` row in the same change. Art is drawn in
  code (SVG built in Rust, `icons.rs`) or is CC0/OSI-licensed with its source recorded. Never
  Microsoft or Adobe artwork.
- **Look.** Colours come from `theme::Palette` (the spec's Parchment and Midnight Sea); never
  hard-code a colour in UI code. Headings and app names use `theme::serif()`, labels
  `theme::semibold()`.
- **Pirate flavour in chrome only.** Strings that change with `pirate_speak` live in
  `brand::Flavour`; commands, dialogs and errors stay plain English.
- **Accessible names.** Painted clickable areas set `widget_info` with a label; the kittest
  harness fails on unlabelled widgets.

## 3. Before you commit

```sh
cargo xtask ci     # fmt --check, clippy -D warnings, tests, wasm check
```

Screenshots of the start page: `JOLLY_ROGER_SHOTS=<dir> cargo test -p jolly-roger-office-ui
--test start_page -- --test-threads=1`; look at the PNGs.

## 4. Parallel agents

Use your own target dir (`CARGO_TARGET_DIR=target/agent-<name>`), at most 8 build jobs, and keep
every `Cargo.toml` valid at all times (write to a temp file and `mv`).
