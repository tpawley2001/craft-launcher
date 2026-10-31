# Jolly Roger Office

The front door of **Jolly Roger Office**, a pirate-themed all-in-one office suite: one start page
for the suite's three apps, with a "New" button for each, an Open button that routes any document
to the right app, and the recent files of all three in one list.

| App | What it is | Opens |
|---|---|---|
| **Captain's Log** | Word processor | `.docx` `.docm` `.dotx` `.dotm` `.doc` `.dot` `.odt` `.rtf` `.txt` `.md` `.html` `.tex` |
| **Plunder Ledger** | Spreadsheet | `.xlsx` `.xlsm` `.xlsb` `.xls` `.ods` `.csv` `.tsv` |
| **Broadside** | Presentations | `.pptx` `.potx` `.ppsx` `.deckcraft` |

Based on the ArtCraft Crafting Apps (MIT OR Apache-2.0). This launcher is a fork of the ArtCraft
launcher (`craft-launcher`); Captain's Log, Plunder Ledger and Broadside are based on WordCraft,
GridCraft and DeckCraft by the ArtCraft team (MIT OR Apache-2.0). It is not affiliated with or
endorsed by ArtCraft.

## What it does

- **Start page**: "Ahoy, Captain" (pirate flavour is on by default; switch it off with *Pirate
  speak* in the top bar and the page speaks plain English). Light ("Parchment") and dark
  ("Midnight Sea") themes, or follow the desktop.
- **New**: a tile per app starts it on a new file.
- **Open a file…** (or Ctrl+O, or drop files on the window): each file opens in the app that
  handles its type.
- **Recent files across the suite**: the launcher's own list (everything it opened) merged with
  each app's own Recent list, read from the app's `ui.json` (`~/.config/<app>/ui.json`; the forks'
  folder names first, then upstream's `wordcraft`, `gridcraft`, `deckcraft`).
- **File associations**: menu entries for the suite and each app, all running the launcher, so a
  double-clicked document lands in the right app and in the recent list.
- **Nothing is downloaded.** The upstream launcher installed and updated apps from GitHub; this
  one only starts binaries already on the computer, found in this order: the path set in
  Settings, `apps/<id>` beside the launcher (the bundle layout), `<id>` beside the launcher, then
  `captains-log` / `plunder-ledger` / `broadside` on `PATH`.

## Command line

```text
jolly-roger-office                       the start page
jolly-roger-office FILE...               open files in the right app, then exit
jolly-roger-office --new <app>           start captains-log, plunder-ledger or broadside
jolly-roger-office --with <app> FILE...  open files in a given app
jolly-roger-office --list                where each app's program was found
jolly-roger-office --export-icons <dir>  write the icons (PNG + SVG) under <dir>/hicolor
jolly-roger-office --desktop-files <dir> [--launcher <path>]
```

Settings live in `~/.config/jolly-roger-office/config.json` (`pirateSpeak`, `theme`,
`appPaths`, `recentCount`) and the launcher's recent list in `recent.json` beside it.
`JOLLY_ROGER_OFFICE_CONFIG_DIR` moves that folder.

## Workspace

```text
crates/office      jolly-roger-office-core   brand names, the three apps and their file types,
                                             routing, settings, recent files, launching,
                                             desktop entries
crates/office-ui   jolly-roger-office-ui     the egui start page, the pirate palette, icons drawn
                                             in code (SVG built in Rust, rendered by resvg)
apps/jolly-roger-office                      the desktop shell (eframe) and command line
xtask/                                       cargo xtask ci | wasm | version | ico | icns
```

All names live in `crates/office/src/brand.rs` (suite) and `crates/office/src/apps.rs` (apps).

## Build and test

```sh
cargo build --release -p jolly-roger-office
cargo xtask ci                     # fmt --check, clippy -D warnings, tests, wasm check
# screenshots of the start page (light and dark) into a folder:
JOLLY_ROGER_SHOTS=/tmp/shots cargo test -p jolly-roger-office-ui --test start_page -- --test-threads=1
```

The Linux bundle (launcher + the three apps + menu entries + icons, as a tarball) is built by
`../bundle/build.sh` from the sibling forks.

## Licence

MIT OR Apache-2.0, at your option (`LICENSE-MIT`, `LICENSE-APACHE`). See `NOTICE` and
`ATTRIBUTION.md` for bundled fonts and their licences.
