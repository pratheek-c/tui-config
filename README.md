# DMS Config Creator

A terminal-based configuration editor for DMS (Diagnostic Management System) JSON files, built with [Ratatui](https://ratatui.rs/) and [Crossterm](https://github.com/crossterm-rs/crossterm).

```
  ╔══════════════════════════════════════════════════════════════╗
  ║  CONFIG   CREATOR   v1.0                            ║
  ╚══════════════════════════════════════════════════════════════╝
   ⚡ SELECT OPERATION ⚡

     ◆ ▓▓▓  Info Creator  ▓▓▓
     ◇    Edit Existing Config
```

## Features

- **Cyberpunk-themed TUI** — neon colors, ASCII art banner, glowing borders, deep purple backgrounds
- **Two modes** — Create new configs or edit existing ones via a file browser
- **7-step form wizard**:
  1. **Part Number** — validated `###-PCA######-X` format
  2. **Basic Info** —  programming details, card setup
  3. **Offline Stages** — with nested testcase editing (id, name, position)
  4. **Diagnostics Stages** — name, JSON file, steps, stage order
  5. **Cards** — card name, part number, type, parameters, XML, EEPROM read position
  6. **Interactive Queries** — key/value pairs with test toggle
  7. **Review & Save** — full JSON preview before saving
- **Dynamic JSON manipulation** — uses `serde_json::Value` for flexible schema handling
- **Files saved as** `./INFO_JSON/<part_number>.json`

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.70+

### Build & Run

```bash
# Debug build
cargo run

# Release build (~780KB)
cargo build --release
./target/release/tui-start
```

### Usage

1. Launch the app — you'll see the dashboard
2. Choose **Info Creator** to create a new config, or **Edit Existing Config** to modify one
3. Both options open a file browser:
   - **Create mode** — press `n` to enter a new part number, or select an existing file to overwrite
   - **Edit mode** — select an existing `.json` file from `INFO_JSON/`
4. Fill in each step of the form wizard
5. Review the generated JSON on the final step
6. Press `Enter` or `s` to save

### Keybindings

| Screen | Key | Action |
|--------|-----|--------|
| **Dashboard** | `↑`/`↓` | Navigate menu |
| | `Enter` | Select |
| | `q` | Quit |
| **File Browser** | `↑`/`↓` | Navigate files |
| | `Enter` | Select file |
| | `n` | New file (Create mode only) |
| | `Esc` | Back to dashboard |
| **Form Steps** | `Tab`/`↑`/`↓` | Cycle fields |
| | `Enter` | Next step / Confirm |
| | `Esc` | Previous step |
| | `Space` | Toggle boolean (Programming Reqd) |
| **List Steps** | `a` | Add item |
| | `e` | Edit selected item |
| | `d` | Delete selected item |
| | `t` | Add testcase (Offline Stages) |
| | `s` | Toggle SFP test (Interactive Queries) |

## Architecture

```
src/
├── main.rs                  # Entry point, terminal setup, event loop
├── app/
│   ├── app.rs               # Controller — all input handling
│   └── mode.rs              # Mode state machine (Dashboard/FileBrowser/Form/Popup/Quit)
├── domain/
│   ├── validators.rs        # Part number validation
│   └── transformers.rs      # Flat fields ↔ serde_json::Value conversion
├── services/
│   ├── fs.rs                # Filesystem I/O (INFO_JSON dir, file listing, save/load)
│   └── json.rs              # JSON load/save wrappers
├── state/
│   ├── app_state.rs         # AppState, FormStep enum, mode transitions
│   ├── form_state.rs        # All editable fields for every form section
│   └── file_state.rs        # File browser selection state
└── ui/                      # Pure rendering — never mutates state or performs I/O
    ├── theme.rs             # Shared neon color constants
    ├── dashboard.rs         # ASCII art banner, menu, status bar
    ├── file_browser.rs      # Purpose-aware file listing
    ├── form/
    │   └── stepper.rs       # 7-step form renderer
    └── components/
        └── popup.rs         # Reusable confirmation and form popups
```

### Design Principles

| Layer | Responsibility | Rule |
|-------|---------------|------|
| **domain** | Pure logic — validation, JSON transformation | No I/O, no state |
| **services** | Filesystem and JSON I/O | No UI, no state mutation |
| **state** | Holds all application data | No I/O, no rendering |
| **app** | Controller — handles input, orchestrates layers | May call services, mutate state |
| **ui** | Pure rendering from state | Never mutates state, never does I/O |

## JSON Schema

The generated configuration is wrapped in an `Info` object:


## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| [ratatui](https://crates.io/crates/ratatui) | 0.30 | Terminal UI framework |
| [crossterm](https://crates.io/crates/crossterm) | 0.28 | Cross-platform terminal control |
| [serde_json](https://crates.io/crates/serde_json) | 1.0 | Dynamic JSON manipulation |
| [chrono](https://crates.io/crates/chrono) | 0.4 | Timestamps |

## License

Internal tool — not publicly licensed.
