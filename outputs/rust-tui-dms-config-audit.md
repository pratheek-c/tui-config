# Codebase Audit: rust-tui-dms-config

**Audit Date:** 2026-04-27  
**Codebase:** `D:/Pratheek/rust/tui-start` (local repository, no prior paper)  
**Tool version:** `tui-start v0.1.0`  
**Slug:** `rust-tui-dms-config`  
**Purpose:** Pre-paper structural audit — architecture analysis, bug inventory, design observations, and research-positioning notes for writing a tool/systems paper.

---

## Executive Summary

`tui-start` is a **Rust TUI application** built with `ratatui` + `crossterm` that guides users through a 7-step form to create and edit JSON configuration files for a hardware Device Management System (DMS / T-MAMS). The codebase is compact (~1,100 SLOC across 17 source files), well-commented, and follows a clean layered architecture. The code compiles cleanly, ships 2 passing unit tests, and demonstrates several strong design patterns worth describing in a paper.

However, **three critical silent-data-loss bugs** (BUG-02, BUG-03, BUG-04), **one controller–UI mismatch** (BUG-05), and an **unused external dependency** (ISSUE-01) must be addressed before the tool can be presented as production-ready or described as a correct implementation of its stated design goals.

---

## 1. What the System Does

| Property | Value |
|---|---|
| Language | Rust 2021 edition |
| Binary | `tui-start` |
| UI crate | `ratatui 0.30` |
| Input crate | `crossterm 0.28` |
| Data format | JSON (`serde_json 1.0`) |
| Declared but unused | `chrono 0.4` |
| Total source files | 17 |
| Lines of source code | ~1,100 (excluding `target/`) |
| Unit tests | 2 (both pass) |
| Compiler warnings | 2 (`dead_code`) |

The application stores configs in `./INFO_JSON/<part-number>.json` relative to the working directory. The JSON schema captures: hardware part number, UCD/CFPGA/DEDI/T-MAMS subsystem identifiers, offline test stages (with nested testcases), diagnostics stages, PCB cards, and interactive queries.

---

## 2. Architecture Audit

### 2.1 Module Map

```
main.rs                 ← terminal setup, event loop (100 ms poll)
app/
  app.rs                ← controller (App struct, handle_key, render)
  mode.rs               ← Mode enum (state machine)
state/
  app_state.rs          ← AppState + FormStep enum
  form_state.rs         ← FormState (flat field bag), ListItem
  file_state.rs         ← file browser state
domain/
  transformers.rs       ← JSON ↔ form field conversions (schema owner)
  validators.rs         ← pure validation functions
services/
  fs.rs                 ← all filesystem I/O
  json.rs               ← JSON load/save via services::fs
ui/
  dashboard.rs          ← dashboard renderer
  file_browser.rs       ← file list renderer
  form/
    stepper.rs          ← multi-step form renderer
    sections/mod.rs     ← placeholder (empty)
  components/
    popup.rs            ← reusable popup renderer
```

### 2.2 Layering Rules — Observed Compliance

| Rule | Stated | Verified |
|---|---|---|
| UI never mutates state | ✅ (doc comment) | ✅ Confirmed — all render functions take `&AppState` |
| Domain has no I/O | ✅ (doc comment) | ✅ Confirmed — `transformers.rs` and `validators.rs` are pure |
| All I/O in `services/` | ✅ (doc comment) | ✅ Confirmed — `fs::write` called only via `services::fs` |
| No JSON construction in UI | ✅ (doc comment) | ✅ Confirmed — UI reads `&AppState`, never calls `json!()` |

**Finding:** The layering contract is faithfully implemented. This is a genuine architectural contribution worth describing in the paper.

### 2.3 State Machine Design

```
Dashboard ──Enter──► FileBrowser(Create|Edit)
                          │
                          ├── 'n' (Create) ──► Form(PartNumber)
                          └── Enter (load) ──► Form(BasicInfo)
                                                    │
                               Form steps cycle     │
                               PartNumber→BasicInfo→OfflineStages→
                               DiagnosticsStages→Cards→
                               InteractiveQueries→Review
                                                    │
                                                    └──► ConfirmPopup ──► Dashboard
```

The `Mode` enum is an owned variant type — impossible to be in an undefined state. `FormStep` encodes progression via `index()` / `from_index()`. This is idiomatic Rust FSM design.

**Finding:** The state machine has no unreachable variants. `Mode::Quit` is reached but the `while !app.should_quit` guard exits before rendering it. Correct.

### 2.4 The Transformer Pattern

`domain/transformers.rs` is the **single source of truth for the JSON schema**. No other module constructs JSON objects directly. Build/extract function pairs exist for every schema section:

| Section | Build fn | Extract fn |
|---|---|---|
| Template | `default_template` | — |
| Offline Stage | `build_offline_stage` | `extract_offline_stage` |
| Testcase | `build_testcase` | `extract_testcase` |
| Diagnostics Stage | `build_diagnostics_stage` | `extract_diagnostics_stage` |
| Card | `build_card` | `extract_card` |
| Interactive Query | `build_interactive_query` | `extract_interactive_query` |
| UCD | `build_ucd` | `extract_ucd` |
| CFPGA | `build_cfpga` | `extract_cfpga` |
| DEDI | `build_dedi` | `extract_dedi` |
| T-MAMS | `build_tmams` | `extract_tmams` |
| Programming Details | `build_programming_details` | `extract_programming_details` |

**Finding (Positive):** Every section has a symmetric build/extract pair. Round-trip correctness is structurally guaranteed for all sections *except* those affected by bugs listed in §3.

---

## 3. Bug Inventory

### 🔴 BUG-01 — JSON Key Typo: `"intruction_message"` (missing 's')

| | |
|---|---|
| **Severity** | Medium |
| **Files** | `src/domain/transformers.rs:54`, `src/app/app.rs:272`, `src/state/form_state.rs:203` |

The JSON output key is `"intruction_message"` (missing 's') in the schema template, the write path (`sync_basic_info_to_config`), and the read path (`from_config`). Because all three use the same misspelled key, internal round-trips are consistent. However, any DMS consumer that reads `"instruction_message"` will receive `null`/missing.

**Evidence:**
```rust
// transformers.rs:54
"intruction_message": ""
// app.rs:272
obj.insert("intruction_message".into(), json!(f.instruction_message));
// form_state.rs:203
let instruction_message = str_field(&info, "intruction_message");
```

**Fix:** Rename key to `"instruction_message"` in all three locations (coordinate with DMS consumer schema).

---

### 🔴 BUG-02 — `Programming_Reqd` Cannot Be Toggled — Hardcoded `false` on New Configs

| | |
|---|---|
| **Severity** | High — silent data loss |
| **Files** | `src/app/app.rs:242-242`, `src/state/form_state.rs:46` |

`FormState` contains `programming_reqd: bool` and `sync_basic_info_to_config` writes it to `config["Info"]["Programming_Reqd"]`. However, **the Basic Info step's `focused_basic_field()` maps only 14 string fields (indices 0–13)**; `programming_reqd` is not among them. There is no `Tab`-accessible field or toggle key for this boolean. It initialises to `false` and can never be set to `true` when creating a new config.

When loading an existing config via `from_config`, the value is correctly deserialized — so edits to existing configs preserve the field — but any newly created config will always output `"Programming_Reqd": false`, silently.

**Evidence:**
```rust
// app.rs — no case for programming_reqd
fn focused_basic_field(&mut self) -> Option<&mut String> {
    match f.focused_field {
        0 => Some(&mut f.ucd_path),
        // ... 13 cases ...
        _ => None,
    }
}
// form_state.rs
programming_reqd: false,  // always
```

**Fix:** Add a toggle keybinding (e.g., `Space` on a dedicated line) or a 15th Tab-cycle field that shows the boolean state and flips it.

---

### 🔴 BUG-03 — `card_setup_json` Cannot Be Edited — Always Empty String

| | |
|---|---|
| **Severity** | High — silent data loss |
| **Files** | `src/app/app.rs:271`, `src/state/form_state.rs:51`, `src/ui/form/stepper.rs` |

Identical to BUG-02: `card_setup_json` is a form field that gets written to `config["Info"]["Card_Setup"]["json"]`, and is correctly loaded from existing configs, but is never exposed in the Basic Info UI. Any new config will always have `"Card_Setup": { "json": "" }`.

**Evidence:**
```rust
// app.rs:271 — written but field is never focused
obj.insert("Card_Setup".into(), json!({ "json": f.card_setup_json }));

// stepper.rs render_basic_info — only 14 fields rendered, no card_setup_json entry
let fields = [
    ("◈ UCD Path", &f.ucd_path, NEON_CYAN),
    // ... 13 more entries — no card_setup_json
];
```

**Fix:** Add a 15th entry to the `fields` array in `render_basic_info` and a corresponding case in `focused_basic_field()`.

---

### 🟡 BUG-04 — `Recovery_Stages` Exists in Schema but Has No UI

| | |
|---|---|
| **Severity** | Medium — missing functionality |
| **Files** | `src/domain/transformers.rs:25` |

The default template includes `"Recovery_Stages": []`. There is no `FormStep::RecoveryStages`, no handler, no renderer. The field will always be an empty array in every config created with this tool. If the DMS system requires recovery stages for any workflow, users cannot populate them.

**Fix:** Either add a `FormStep::RecoveryStages` step (mirroring the existing `OfflineStages` implementation), or explicitly document that recovery stages are out of scope and remove the key from the template.

---

### 🟡 BUG-05 — Offline Stage Popup: Focus Indicator Is Invisible

| | |
|---|---|
| **Severity** | Medium UX — controller/UI mismatch |
| **Files** | `src/app/app.rs:handle_offline_stage_popup_key`, `src/ui/form/stepper.rs:199–213` |

`handle_offline_stage_popup_key` cycles `focused_field` through 0–3, mapping to `os_name`, `os_stage_order`, `os_tool`, and `os_steps`. However, the UI for this popup renders all four values as a single concatenated `"Details"` string and always passes `0` as the focused index to `render_form_popup`:

```rust
// stepper.rs — NOT passing f.focused_field
crate::ui::components::popup::render_form_popup(
    frame, "⟨ Add/Edit Offline Stage ⟩",
    &[("Details", all_fields.as_str())],
    0,   // ← always 0, focus is invisible
);
```

Compare with the testcase popup (correct):
```rust
// stepper.rs — CORRECTLY passes f.focused_field
crate::ui::components::popup::render_form_popup(
    frame, "⟨ Add/Edit Testcase ⟩", &fields, f.focused_field,
);
```

The user is typing into os_name/os_stage_order/os_tool/os_steps while pressing Tab, but cannot see which field is active.

**Fix:** Refactor the offline stage popup to render 4 individual fields (matching the testcase popup pattern) and pass `f.focused_field`.

---

### 🟡 BUG-06 — Part Number Extracted from Nested Cards Array (Fragile)

| | |
|---|---|
| **Severity** | Low–Medium |
| **Files** | `src/state/form_state.rs:from_config()` |

When loading an existing config for editing, the part number is extracted from `config["Info"]["cards"][0]["partNumber"]`:

```rust
let part_number = config
    .get("Info")
    .and_then(|v| v.get("cards"))
    .and_then(|v| v.as_array())
    .and_then(|arr| arr.first())       // ← breaks if cards is empty
    .and_then(|c| c.get("partNumber"))
    .and_then(|v| v.as_str())
    .unwrap_or("")
    .to_string();
```

If `cards` is empty or `partNumber` is absent, `part_number = ""`, and `save_config` will write to `INFO_JSON/.json`.

**Fix:** Add a top-level `"partNumber"` key to the JSON schema (in `default_template`), or validate non-empty part number after `from_config`.

---

## 4. Code Quality Observations

### ISSUE-01 — Unused Dependency: `chrono`

```toml
# Cargo.toml
chrono = "0.4"
```

`chrono` is declared but never imported or used in any source file. Confirmed by `grep -rn "chrono" src/` returning no matches. This adds ~200 KB to compile time unnecessarily.

**Fix:** Remove `chrono = "0.4"` from `Cargo.toml`.

---

### ISSUE-02 — Dead Code: `validate_required` and `validate_u64`

```
warning: function `validate_required` is never used
warning: function `validate_u64` is never used
```

Two validators exist but are never called. Most list-step fields (stage order, tool, steps, query value, etc.) accept empty strings without validation. Only `part_number` and `card_name`/`os_name`/`iq_key` (checked at commit) are guarded.

**Fix:** Either call these validators in the appropriate commit functions, or delete them.

---

### ISSUE-03 — Zero Tests for `domain/transformers.rs`

The transformer module is the most critical correctness surface — all JSON field name knowledge lives here — yet it has no tests. `domain/validators.rs` correctly has 4 tests.

**Fix:** Add round-trip tests (e.g., `build_offline_stage` → serialize → `extract_offline_stage` → assert field equality).

---

### ISSUE-04 — JSON Schema Case Inconsistency

The schema mixes three naming conventions within the same top-level object:

| Style | Keys |
|---|---|
| PascalCase | `Offline_Stages`, `Diagnostics_Stages`, `Programming_Reqd`, `CFPGA`, `UCD`, `DEDI` |
| camelCase | `stageOrder`, `cardXml`, `isSfpTest`, `tejDMSxml`, `tclScript` |
| snake_case | `eeprom_read_pos`, `work_flow_version`, `schema_version` |

This is likely inherited from the existing DMS system. If so, it should be documented as a constraint, not a design choice.

---

### ISSUE-05 — No README or User Documentation

The repository has no `README.md`, no user manual, and no rustdoc examples. Module-level doc comments (`//!`) are thorough, but there is no quick-start guide or schema documentation.

---

## 5. Architecture Strengths (Paper-Worthy)

These properties are genuine contributions and should be highlighted in a paper:

| Strength | Description |
|---|---|
| **Strict layer isolation** | UI never mutates state; domain is pure; all I/O in services. Confirmed empirically, not just claimed. |
| **Mode-enum FSM** | Navigation state expressed as owned Rust enum variants. No invalid state possible at type level. |
| **Transformer pattern** | All JSON schema knowledge centralized in `domain/transformers.rs`. Adding a new field requires changing exactly one file. |
| **No panic in production paths** | All fallible operations return `Result<_, String>`. No `unwrap()` in the event loop or save/load paths. |
| **KeyEventKind::Press guard** | `main.rs` filters to `KeyEventKind::Press` only, preventing double-entry on platforms that emit Press+Release. |
| **Event poll at 100 ms** | Low-CPU spinning without blocking input responsiveness. Explicit design choice. |
| **Cyberpunk themed** | Consistent neon RGB palette via named constants, reused across all screens. |

---

## 6. Design Tradeoffs to Discuss in the Paper

| Tradeoff | Decision Made | Alternative |
|---|---|---|
| FormState as a flat monolith | One struct with ~60 fields, avoids Rust borrow complexity | Per-step sub-structs with lifetimes |
| Steps string as comma-separated field | Easy to type in TUI; parsed at commit | Array-native editing (add/remove individual steps) |
| Part number as filename | Simple; no separate index needed | Metadata sidecar or SQLite |
| `INFO_JSON/` hardcoded in CWD | Simple to use; no config needed | `--output-dir` CLI arg or env var |
| No undo/redo | Lower complexity | Command pattern stack |
| No async I/O | Correct for small JSON files | Tokio for large file lists |

---

## 7. Research Paper Positioning

### Proposed title
**"CVOUS: A Rust TUI Application for Guided DMS Hardware Configuration Authoring"**  
*(or a domain-appropriate alternative — slug `tui-dms-config`)*

### Contribution categories

1. **Tool contribution:** A terminal-native, keyboard-driven form editor that outputs validated, schema-correct JSON for hardware DMS workflows.
2. **Architecture contribution:** Demonstration of strict layering (UI/Controller/Domain/Services) in a Rust TUI context using mode-enum FSMs and the transformer pattern as a JSON schema management strategy.
3. **Engineering contribution:** Analysis of design tradeoffs specific to TUI state management in Rust (flat vs. hierarchical state, borrow-checker pressures, FSM design).

### Comparable prior work to cite
- `ratatui` ecosystem examples and the ratatui book (TUI framework reference)
- `bubbletea` (Go TUI framework — The Elm Architecture) as contrast
- Papers on form-based data entry for structured data (e.g., PBE, guided wizards in CLI tools)
- Rust type-system FSM papers (e.g., "Session Types" or "Typestate" in Rust)

---

## 8. Fix Priority Checklist

| # | Issue | Severity | Fix Effort |
|---|---|---|---|
| BUG-02 | `Programming_Reqd` not toggleable | 🔴 High | Low (add toggle to Basic Info UI) |
| BUG-03 | `card_setup_json` not editable | 🔴 High | Low (add 15th field to Basic Info) |
| BUG-05 | Offline stage popup focus invisible | 🟡 Medium | Medium (refactor popup rendering) |
| BUG-04 | `Recovery_Stages` no UI | 🟡 Medium | High (add full step) or Low (remove key) |
| BUG-01 | `intruction_message` typo | 🟡 Medium | Trivial (3-file rename) |
| BUG-06 | Part number from cards[0] | 🟡 Low | Low (add fallback) |
| ISSUE-01 | Unused `chrono` dep | 🔵 Low | Trivial (remove 1 line) |
| ISSUE-02 | Dead validator functions | 🔵 Low | Low (use or delete) |
| ISSUE-03 | No transformer tests | 🔵 Low | Medium (write round-trip tests) |
| ISSUE-04 | Schema case inconsistency | 🔵 Cosmetic | Document (if inherited) |
| ISSUE-05 | No README | 🔵 Low | Low (write 1 page) |

---

## 9. Reproduction Notes

```bash
# Build
cargo build

# Run tests (2 pass)
cargo test

# Run application (requires terminal with RGB color support)
cargo run

# Compiler warnings
cargo check
# → warning: `validate_required` is never used
# → warning: `validate_u64` is never used
```

No external tooling, no database, no network access required. All state stored locally in `./INFO_JSON/`.

---

## Sources

- **Codebase (local):** `D:/Pratheek/rust/tui-start` — `tui-start v0.1.0` (Rust 2021 edition)
- **ratatui crate:** https://crates.io/crates/ratatui (v0.30) — https://ratatui.rs/
- **crossterm crate:** https://crates.io/crates/crossterm (v0.28)
- **serde_json crate:** https://crates.io/crates/serde_json (v1.0)
- **chrono crate (unused):** https://crates.io/crates/chrono (v0.4)
- **ratatui book (architecture reference):** https://ratatui.rs/concepts/
- **Rust dead_code lint:** https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#dead-code
- **ratatui GitHub:** https://github.com/ratatui/ratatui
