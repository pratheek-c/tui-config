# Audit Plan: rust-tui-dms-config

**Target codebase:** `D:/Pratheek/rust/tui-start` (local, no prior paper)
**Slug:** `rust-tui-dms-config`
**Goal:** Full structural + correctness audit to support writing a research/tool paper.

## What the system is
A terminal user interface (TUI) application written in Rust (`ratatui` + `crossterm`) that
creates and edits JSON configuration files for a hardware Device Management System (DMS /
T-MAMS). Users fill in a 7-step guided form; the result is a structured JSON config written
to `INFO_JSON/<part-number>.json`.

## Audit axes
1. **Architecture audit** — layer separation, state machine design, MVC adherence.
2. **Schema correctness** — do transformer round-trips (build → extract) preserve all fields?
3. **Validation coverage** — what's validated vs. silently accepted?
4. **UI–logic consistency** — does the UI correctly expose all editable fields?
5. **Bug hunting** — silent data loss, typos in JSON keys, unreachable code, unused deps.
6. **Reproducibility / developer experience** — build, test, run, documentation.
7. **Research positioning** — what novel or paper-worthy claims can be made?

## Files to check
- `src/main.rs` — event loop design
- `src/app/app.rs` — controller dispatch, sync functions
- `src/app/mode.rs` — state machine
- `src/state/{app_state,form_state,file_state}.rs`
- `src/domain/transformers.rs` — JSON schema
- `src/domain/validators.rs` — validation tests
- `src/services/{fs,json}.rs`
- `src/ui/**` — renderer/controller alignment
- `Cargo.toml` — dependency audit

## Output
`outputs/rust-tui-dms-config-audit.md`
