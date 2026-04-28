//! App: Main controller.
//!
//! Owns all state, handles input, delegates rendering to `ui` and I/O to `services`.
//!
//! Flow:  User Input → handle_key() → mutate State → UI reads State → render

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use serde_json::json;
use std::path::PathBuf;

use crate::app::mode::{BrowserPurpose, Mode};
use crate::domain::transformers;
use crate::domain::validators;
use crate::services::{fs as fs_svc, json as json_svc};
use crate::state::app_state::{AppState, FormStep};
use crate::state::file_state::FileState;
use crate::state::form_state::ListItem;
use crate::ui;

const DASHBOARD_ITEMS: &[&str] = &["Info Creator", "Edit Existing Config"];

/// Number of Basic Info fields (for Tab cycling).
const BASIC_INFO_FIELD_COUNT: usize = 16;

pub struct App {
    pub state: AppState,
    pub file_state: FileState,
    pub info_dir: PathBuf,
    pub dashboard_selected: usize,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Result<Self, String> {
        let cwd = std::env::current_dir()
            .map_err(|e| format!("Cannot get CWD: {}", e))?;
        let info_dir = fs_svc::ensure_info_json_dir(&cwd)?;
        Ok(Self {
            state: AppState::new(),
            file_state: FileState::new(),
            info_dir,
            dashboard_selected: 0,
            should_quit: false,
        })
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.state.mode {
            Mode::Dashboard => self.handle_dashboard_key(key),
            Mode::FileBrowser(purpose) => self.handle_file_browser_key(key, purpose),
            Mode::Form(step) => self.handle_form_key(key, step),
            Mode::ConfirmPopup => self.handle_confirm_popup_key(key),
            Mode::Quit => self.should_quit = true,
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        match self.state.mode {
            Mode::Dashboard => ui::dashboard::render(frame, self.dashboard_selected),
            Mode::FileBrowser(purpose) => {
                ui::file_browser::render(frame, &self.file_state, &self.state.status_message, purpose);
            }
            Mode::Form(step) => {
                ui::form::stepper::render(frame, &self.state, step);
            }
            Mode::ConfirmPopup => {
                ui::form::stepper::render(frame, &self.state, FormStep::Review);
                ui::components::popup::render_confirm(frame, "Save this configuration? (y/n)");
            }
            Mode::Quit => {}
        }
    }

    // ============================================================
    //  Dashboard
    // ============================================================

    fn handle_dashboard_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.dashboard_selected > 0 { self.dashboard_selected -= 1; }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.dashboard_selected < DASHBOARD_ITEMS.len() - 1 { self.dashboard_selected += 1; }
            }
            KeyCode::Enter => match self.dashboard_selected {
                0 => { self.refresh_file_list(); self.state.mode = Mode::FileBrowser(BrowserPurpose::Create); }
                1 => { self.refresh_file_list(); self.state.mode = Mode::FileBrowser(BrowserPurpose::Edit); }
                _ => {}
            },
            KeyCode::Char('q') | KeyCode::Esc => { self.state.mode = Mode::Quit; self.should_quit = true; }
            _ => {}
        }
    }

    // ============================================================
    //  File Browser
    // ============================================================

    fn refresh_file_list(&mut self) {
        match fs_svc::list_json_files(&self.info_dir) {
            Ok(files) => { self.file_state.files = files; self.file_state.selected = 0; self.state.clear_status(); }
            Err(e) => self.state.set_error(e),
        }
    }

    fn handle_file_browser_key(&mut self, key: KeyEvent, purpose: BrowserPurpose) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.file_state.select_prev(),
            KeyCode::Down | KeyCode::Char('j') => self.file_state.select_next(),
            KeyCode::Enter => {
                // Both modes: open selected file
                //   Create mode → load it to continue editing
                //   Edit mode → load it to modify
                if let Some((_name, path)) = self.file_state.selected_file() {
                    let ps = path.to_string_lossy().to_string();
                    match json_svc::load_config(path) {
                        Ok(config) => self.state.reset_for_edit(config, ps),
                        Err(e) => self.state.set_error(e),
                    }
                }
            }
            KeyCode::Char('n') => {
                // Only in Create mode: start a new config from scratch
                if purpose == BrowserPurpose::Create {
                    self.state.reset_for_new();
                }
            }
            KeyCode::Esc => self.state.mode = Mode::Dashboard,
            _ => {}
        }
    }

    // ============================================================
    //  Form dispatch
    // ============================================================

    fn handle_form_key(&mut self, key: KeyEvent, step: FormStep) {
        // Testcase popup (nested inside offline-stage editing)
        if self.state.form.os_testcase_editing {
            self.handle_testcase_popup_key(key);
            return;
        }
        // Offline-stage popup
        if self.state.form.offline_stage_editing {
            self.handle_offline_stage_popup_key(key);
            return;
        }
        // Diagnostics-stage popup
        if self.state.form.diagnostics_stage_editing {
            self.handle_diag_stage_popup_key(key);
            return;
        }
        // Card popup
        if self.state.form.card_editing {
            self.handle_card_popup_key(key);
            return;
        }
        // Query popup
        if self.state.form.query_editing {
            self.handle_query_popup_key(key);
            return;
        }

        match step {
            FormStep::PartNumber => self.handle_part_number_step(key),
            FormStep::BasicInfo => self.handle_basic_info_step(key),
            FormStep::OfflineStages => self.handle_list_step_offline(key),
            FormStep::DiagnosticsStages => self.handle_list_step_diag(key),
            FormStep::Cards => self.handle_list_step_cards(key),
            FormStep::InteractiveQueries => self.handle_list_step_queries(key),
            FormStep::Review => self.handle_review_step(key),
        }
    }

    // ============================================================
    //  Step 1: Part Number
    // ============================================================

    fn handle_part_number_step(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                match validators::validate_part_number(&self.state.form.part_number) {
                    Ok(()) => {
                        self.state.config = transformers::default_template(&self.state.form.part_number);
                        self.advance(FormStep::PartNumber);
                        self.state.clear_status();
                    }
                    Err(e) => self.state.set_error(e),
                }
            }
            KeyCode::Esc => self.state.mode = Mode::Dashboard,
            KeyCode::Backspace => { self.state.form.part_number.pop(); self.state.clear_status(); }
            KeyCode::Char(c) => { self.state.form.part_number.push(c); self.state.clear_status(); }
            _ => {}
        }
    }

    // ============================================================
    //  Step 2: Basic Info (UCD, CFPGA, DEDI, T-MAMS, etc.)
    // ============================================================

    fn handle_basic_info_step(&mut self, key: KeyEvent) {
        let f = &mut self.state.form;
        let max = BASIC_INFO_FIELD_COUNT;
        let focused = f.focused_field;

        match key.code {
            KeyCode::Tab | KeyCode::Down => f.focused_field = (focused + 1) % max,
            KeyCode::Up => f.focused_field = if focused == 0 { max - 1 } else { focused - 1 },
            KeyCode::Enter => {
                // Validate & advance
                self.sync_basic_info_to_config();
                self.advance(FormStep::BasicInfo);
                self.state.clear_status();
            }
            KeyCode::Esc => self.go_back(FormStep::BasicInfo),
            // Toggle Programming_Reqd when field 15 is focused
            KeyCode::Char(' ') if focused == 15 => {
                self.state.form.programming_reqd = !self.state.form.programming_reqd;
            }
            KeyCode::Backspace => self.edit_basic_info_field(|s| { s.pop(); }),
            KeyCode::Char(c) => self.edit_basic_info_field(|s| { s.push(c); }),
            _ => {}
        }
    }

    /// Get a mutable reference to the currently focused Basic Info field.
    fn focused_basic_field(&mut self) -> Option<&mut String> {
        let f = &mut self.state.form;
        match f.focused_field {
            0 => Some(&mut f.ucd_path),
            1 => Some(&mut f.ucd_address),
            2 => Some(&mut f.ucd_tc_series),
            3 => Some(&mut f.ucd_testcase_id),
            4 => Some(&mut f.cfpga_file_name),
            5 => Some(&mut f.cfpga_tcl_script),
            6 => Some(&mut f.dedi_location),
            7 => Some(&mut f.dedi_path),
            8 => Some(&mut f.tmams_workflow_version),
            9 => Some(&mut f.tmams_schema_version),
            10 => Some(&mut f.prog_name),
            11 => Some(&mut f.prog_json),
            12 => Some(&mut f.prog_stage_order),
            13 => Some(&mut f.card_setup_json),
            14 => Some(&mut f.instruction_message),
            // index 15 = Programming_Reqd toggle (handled specially)
            _ => None,
        }
    }

    fn edit_basic_info_field<F: FnOnce(&mut String)>(&mut self, op: F) {
        if let Some(field) = self.focused_basic_field() {
            op(field);
        }
        self.state.clear_status();
    }

    fn sync_basic_info_to_config(&mut self) {
        let f = &self.state.form;
        let ucd = transformers::build_ucd(&f.ucd_path, &f.ucd_address, &f.ucd_tc_series, &f.ucd_testcase_id);
        let cfpga = transformers::build_cfpga(&f.cfpga_file_name, &f.cfpga_tcl_script);
        let dedi = transformers::build_dedi(&f.dedi_location, &f.dedi_path);
        let tmams = transformers::build_tmams(&f.tmams_workflow_version, &f.tmams_schema_version);
        let prog = transformers::build_programming_details(&f.prog_name, &f.prog_json, &f.prog_stage_order);

        if let Some(info) = self.state.config.get_mut("Info") {
            if let Some(obj) = info.as_object_mut() {
                obj.insert("UCD".into(), ucd);
                obj.insert("CFPGA".into(), cfpga);
                obj.insert("DEDI".into(), dedi);
                obj.insert("T-MAMS".into(), tmams);
                obj.insert("programming_details".into(), prog);
                obj.insert("Programming_Reqd".into(), json!(f.programming_reqd));
                obj.insert("Card_Setup".into(), json!({ "json": f.card_setup_json }));
                obj.insert("intruction_message".into(), json!(f.instruction_message));
            }
        }
    }

    // ============================================================
    //  Step 3: Offline Stages (list with nested testcases)
    // ============================================================

    fn handle_list_step_offline(&mut self, key: KeyEvent) {
        let f = &self.state.form;
        let len = f.offline_stages.len();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => { if len > 0 { self.list_prev(FormStep::OfflineStages); } }
            KeyCode::Down | KeyCode::Char('j') => { if len > 0 { self.list_next(FormStep::OfflineStages, len); } }
            KeyCode::Char('a') => {
                self.state.form.clear_offline_stage_fields();
                self.state.form.offline_stages_selected = None; // ADD mode, not edit
                self.state.form.offline_stage_editing = true;
                self.state.form.focused_field = 0;
                self.state.set_info("Add Offline Stage — fill fields, Enter to commit, Esc to cancel");
            }
            KeyCode::Char('e') => {
                if let Some(idx) = self.state.form.offline_stages_selected {
                    let data = transformers::extract_offline_stage(&self.state.form.offline_stages[idx].value);
                    self.state.form.os_name = data.name;
                    self.state.form.os_stage_order = data.stage_order;
                    self.state.form.os_tool = data.tool;
                    self.state.form.os_steps = transformers::steps_to_string(&data.steps);
                    // Load testcases into sub-list
                    self.state.form.os_testcases = data.testcases.as_array()
                        .map(|arr| arr.iter().enumerate().map(|(_i, tc)| {
                            let (id, name, pos) = transformers::extract_testcase(tc);
                            ListItem::new(format!("{}: {} (id={})", pos, name, id), tc.clone())
                        }).collect()).unwrap_or_default();
                    self.state.form.os_testcases_selected = None;
                    self.state.form.offline_stage_editing = true;
                    self.state.form.focused_field = 0;
                    self.state.set_info("Edit Offline Stage — modify fields, Enter to commit, Esc to cancel");
                }
            }
            KeyCode::Delete | KeyCode::Char('d') => {
                if let Some(idx) = self.state.form.offline_stages_selected {
                    self.state.form.offline_stages.remove(idx);
                    self.state.form.offline_stages_selected = None;
                    self.sync_list_to_config(FormStep::OfflineStages);
                }
            }
            KeyCode::Enter => { self.sync_list_to_config(FormStep::OfflineStages); self.advance(FormStep::OfflineStages); }
            KeyCode::Esc => { self.sync_list_to_config(FormStep::OfflineStages); self.go_back(FormStep::OfflineStages); }
            _ => {}
        }
    }

    fn handle_offline_stage_popup_key(&mut self, key: KeyEvent) {
        // 4 fields: name, stage_order, tool, steps
        let max_fields = 4;
        let f = &mut self.state.form;
        match key.code {
            KeyCode::Tab => f.focused_field = (f.focused_field + 1) % max_fields,
            KeyCode::Esc => {
                f.offline_stage_editing = false;
                f.focused_field = 0;
                self.state.clear_status();
            }
            KeyCode::Char('t') => {
                // Open testcase sub-popup
                f.clear_testcase_fields();
                f.os_testcase_editing = true;
                f.focused_field = 0;
                return;
            }
            KeyCode::Enter => {
                self.commit_offline_stage();
            }
            KeyCode::Backspace => {
                let idx = f.focused_field.min(max_fields - 1);
                match idx {
                    0 => { f.os_name.pop(); }
                    1 => { f.os_stage_order.pop(); }
                    2 => { f.os_tool.pop(); }
                    3 => { f.os_steps.pop(); }
                    _ => {}
                }
            }
            KeyCode::Char(c) => {
                let idx = f.focused_field.min(max_fields - 1);
                match idx {
                    0 => { f.os_name.push(c); }
                    1 => { f.os_stage_order.push(c); }
                    2 => { f.os_tool.push(c); }
                    3 => { f.os_steps.push(c); }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn commit_offline_stage(&mut self) {
        let f = &self.state.form;
        if f.os_name.trim().is_empty() {
            self.state.set_error("Stage name is required");
            return;
        }
        let steps = transformers::string_to_steps(&f.os_steps);
        let tcs: Vec<serde_json::Value> = f.os_testcases.iter().map(|t| t.value.clone()).collect();
        let value = transformers::build_offline_stage(
            &f.os_name, &f.os_stage_order, &f.os_tool, &steps, &json!(tcs),
        );
        let item = ListItem::new(&f.os_name, value);

        if let Some(idx) = self.state.form.offline_stages_selected {
            self.state.form.offline_stages[idx] = item;
        } else {
            self.state.form.offline_stages.push(item);
        }
        self.state.form.offline_stage_editing = false;
        self.state.form.focused_field = 0;
        self.sync_list_to_config(FormStep::OfflineStages);
        self.state.clear_status();
    }

    // ---- Testcase popup (nested inside offline stage) ----

    fn handle_testcase_popup_key(&mut self, key: KeyEvent) {
        let max_fields = 3;
        let f = &mut self.state.form;
        match key.code {
            KeyCode::Tab => f.focused_field = (f.focused_field + 1) % max_fields,
            KeyCode::Esc => { f.os_testcase_editing = false; f.focused_field = 0; }
            KeyCode::Enter => self.commit_testcase(),
            KeyCode::Backspace => {
                match f.focused_field.min(max_fields - 1) {
                    0 => { f.tc_id.pop(); }
                    1 => { f.tc_name.pop(); }
                    2 => { f.tc_position.pop(); }
                    _ => {}
                }
            }
            KeyCode::Char(c) => {
                match f.focused_field.min(max_fields - 1) {
                    0 => { f.tc_id.push(c); }
                    1 => { f.tc_name.push(c); }
                    2 => { f.tc_position.push(c); }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn commit_testcase(&mut self) {
        let f = &self.state.form;
        let id: u64 = f.tc_id.parse().unwrap_or(0);
        let pos: u64 = f.tc_position.parse().unwrap_or(0);
        let tc = transformers::build_testcase(id, &f.tc_name, pos);
        let item = ListItem::new(
            format!("{}: {} (id={})", pos, f.tc_name, id),
            tc,
        );
        if let Some(idx) = self.state.form.os_testcases_selected {
            self.state.form.os_testcases[idx] = item;
        } else {
            self.state.form.os_testcases.push(item);
        }
        self.state.form.os_testcase_editing = false;
        self.state.form.clear_testcase_fields();
        self.state.form.focused_field = 0;
    }

    // ============================================================
    //  Step 4: Diagnostics Stages
    // ============================================================

    fn handle_list_step_diag(&mut self, key: KeyEvent) {
        let len = self.state.form.diagnostics_stages.len();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => { if len > 0 { self.list_prev(FormStep::DiagnosticsStages); } }
            KeyCode::Down | KeyCode::Char('j') => { if len > 0 { self.list_next(FormStep::DiagnosticsStages, len); } }
            KeyCode::Char('a') => {
                self.state.form.clear_diagnostics_stage_fields();
                self.state.form.diagnostics_stage_editing = true;
                self.state.form.focused_field = 0;
                self.state.set_info("Add Diagnostics Stage — fill fields, Enter to commit");
            }
            KeyCode::Char('e') => {
                if let Some(idx) = self.state.form.diagnostics_stages_selected {
                    let data = transformers::extract_diagnostics_stage(
                        &self.state.form.diagnostics_stages[idx].value,
                    );
                    self.state.form.ds_name = data.name;
                    self.state.form.ds_json = data.json_file;
                    self.state.form.ds_steps = transformers::steps_to_string(&data.steps);
                    self.state.form.ds_stage_order = data.stage_order;
                    self.state.form.diagnostics_stage_editing = true;
                    self.state.form.focused_field = 0;
                }
            }
            KeyCode::Delete | KeyCode::Char('d') => {
                if let Some(idx) = self.state.form.diagnostics_stages_selected {
                    self.state.form.diagnostics_stages.remove(idx);
                    self.state.form.diagnostics_stages_selected = None;
                    self.sync_list_to_config(FormStep::DiagnosticsStages);
                }
            }
            KeyCode::Enter => { self.sync_list_to_config(FormStep::DiagnosticsStages); self.advance(FormStep::DiagnosticsStages); }
            KeyCode::Esc => { self.sync_list_to_config(FormStep::DiagnosticsStages); self.go_back(FormStep::DiagnosticsStages); }
            _ => {}
        }
    }

    fn handle_diag_stage_popup_key(&mut self, key: KeyEvent) {
        let max_fields = 4;
        let f = &mut self.state.form;
        match key.code {
            KeyCode::Tab => f.focused_field = (f.focused_field + 1) % max_fields,
            KeyCode::Esc => { f.diagnostics_stage_editing = false; f.focused_field = 0; }
            KeyCode::Enter => self.commit_diag_stage(),
            KeyCode::Backspace => {
                match f.focused_field.min(max_fields - 1) {
                    0 => { f.ds_name.pop(); }
                    1 => { f.ds_json.pop(); }
                    2 => { f.ds_steps.pop(); }
                    3 => { f.ds_stage_order.pop(); }
                    _ => {}
                }
            }
            KeyCode::Char(c) => {
                match f.focused_field.min(max_fields - 1) {
                    0 => { f.ds_name.push(c); }
                    1 => { f.ds_json.push(c); }
                    2 => { f.ds_steps.push(c); }
                    3 => { f.ds_stage_order.push(c); }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn commit_diag_stage(&mut self) {
        let f = &self.state.form;
        if f.ds_name.trim().is_empty() {
            self.state.set_error("Stage name is required");
            return;
        }
        let steps = transformers::string_to_steps(&f.ds_steps);
        let value = transformers::build_diagnostics_stage(&f.ds_name, &f.ds_json, &steps, &f.ds_stage_order);
        let item = ListItem::new(&f.ds_name, value);
        if let Some(idx) = self.state.form.diagnostics_stages_selected {
            self.state.form.diagnostics_stages[idx] = item;
        } else {
            self.state.form.diagnostics_stages.push(item);
        }
        self.state.form.diagnostics_stage_editing = false;
        self.state.form.focused_field = 0;
        self.sync_list_to_config(FormStep::DiagnosticsStages);
        self.state.clear_status();
    }

    // ============================================================
    //  Step 5: Cards
    // ============================================================

    fn handle_list_step_cards(&mut self, key: KeyEvent) {
        let len = self.state.form.cards.len();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => { if len > 0 { self.list_prev(FormStep::Cards); } }
            KeyCode::Down | KeyCode::Char('j') => { if len > 0 { self.list_next(FormStep::Cards, len); } }
            KeyCode::Char('a') => {
                self.state.form.clear_card_fields();
                self.state.form.card_part_number = self.state.form.part_number.clone();
                self.state.form.card_editing = true;
                self.state.form.focused_field = 0;
            }
            KeyCode::Char('e') => {
                if let Some(idx) = self.state.form.cards_selected {
                    let data = transformers::extract_card(&self.state.form.cards[idx].value);
                    self.state.form.card_name = data.card_name;
                    self.state.form.card_part_number = data.part_number;
                    self.state.form.card_type = data.card_type;
                    self.state.form.card_parameters = data.parameters;
                    self.state.form.card_xml = data.card_xml;
                    self.state.form.card_tej_dms_xml = data.tej_dms_xml;
                    self.state.form.card_eeprom_read_pos = data.eeprom_read_pos.to_string();
                    self.state.form.card_editing = true;
                    self.state.form.focused_field = 0;
                }
            }
            KeyCode::Delete | KeyCode::Char('d') => {
                if let Some(idx) = self.state.form.cards_selected {
                    self.state.form.cards.remove(idx);
                    self.state.form.cards_selected = None;
                    self.sync_list_to_config(FormStep::Cards);
                }
            }
            KeyCode::Enter => { self.sync_list_to_config(FormStep::Cards); self.advance(FormStep::Cards); }
            KeyCode::Esc => { self.sync_list_to_config(FormStep::Cards); self.go_back(FormStep::Cards); }
            _ => {}
        }
    }

    fn handle_card_popup_key(&mut self, key: KeyEvent) {
        let max_fields = 7;
        let f = &mut self.state.form;
        match key.code {
            KeyCode::Tab => f.focused_field = (f.focused_field + 1) % max_fields,
            KeyCode::Esc => { f.card_editing = false; f.focused_field = 0; }
            KeyCode::Enter => self.commit_card(),
            KeyCode::Backspace => {
                match f.focused_field.min(max_fields - 1) {
                    0 => { f.card_name.pop(); }
                    1 => { f.card_part_number.pop(); }
                    2 => { f.card_type.pop(); }
                    3 => { f.card_parameters.pop(); }
                    4 => { f.card_xml.pop(); }
                    5 => { f.card_tej_dms_xml.pop(); }
                    6 => { f.card_eeprom_read_pos.pop(); }
                    _ => {}
                }
            }
            KeyCode::Char(c) => {
                match f.focused_field.min(max_fields - 1) {
                    0 => { f.card_name.push(c); }
                    1 => { f.card_part_number.push(c); }
                    2 => { f.card_type.push(c); }
                    3 => { f.card_parameters.push(c); }
                    4 => { f.card_xml.push(c); }
                    5 => { f.card_tej_dms_xml.push(c); }
                    6 => { f.card_eeprom_read_pos.push(c); }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn commit_card(&mut self) {
        let f = &self.state.form;
        if f.card_name.trim().is_empty() {
            self.state.set_error("Card name is required");
            return;
        }
        let eeprom: u64 = f.card_eeprom_read_pos.parse().unwrap_or(0);
        let value = transformers::build_card(
            &f.card_name, &f.card_part_number, &f.card_type,
            &f.card_parameters, &f.card_xml, &f.card_tej_dms_xml, eeprom,
        );
        let item = ListItem::new(&f.card_name, value);
        if let Some(idx) = self.state.form.cards_selected {
            self.state.form.cards[idx] = item;
        } else {
            self.state.form.cards.push(item);
        }
        self.state.form.card_editing = false;
        self.state.form.focused_field = 0;
        self.sync_list_to_config(FormStep::Cards);
        self.state.clear_status();
    }

    // ============================================================
    //  Step 6: Interactive Queries
    // ============================================================

    fn handle_list_step_queries(&mut self, key: KeyEvent) {
        let len = self.state.form.interactive_queries.len();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => { if len > 0 { self.list_prev(FormStep::InteractiveQueries); } }
            KeyCode::Down | KeyCode::Char('j') => { if len > 0 { self.list_next(FormStep::InteractiveQueries, len); } }
            KeyCode::Char('a') => {
                self.state.form.clear_query_fields();
                self.state.form.query_editing = true;
                self.state.form.focused_field = 0;
            }
            KeyCode::Char('e') => {
                if let Some(idx) = self.state.form.queries_selected {
                    let data = transformers::extract_interactive_query(
                        &self.state.form.interactive_queries[idx].value,
                    );
                    self.state.form.iq_key = data.key;
                    self.state.form.iq_value = match &data.value {
                        serde_json::Value::Null => String::new(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    self.state.form.iq_message = data.message;
                    self.state.form.iq_is_sfp_test = data.is_sfp_test;
                    self.state.form.iq_id = data.id.to_string();
                    self.state.form.query_editing = true;
                    self.state.form.focused_field = 0;
                }
            }
            KeyCode::Delete | KeyCode::Char('d') => {
                if let Some(idx) = self.state.form.queries_selected {
                    self.state.form.interactive_queries.remove(idx);
                    self.state.form.queries_selected = None;
                    self.sync_list_to_config(FormStep::InteractiveQueries);
                }
            }
            KeyCode::Enter => { self.sync_list_to_config(FormStep::InteractiveQueries); self.advance(FormStep::InteractiveQueries); }
            KeyCode::Esc => { self.sync_list_to_config(FormStep::InteractiveQueries); self.go_back(FormStep::InteractiveQueries); }
            _ => {}
        }
    }

    fn handle_query_popup_key(&mut self, key: KeyEvent) {
        let max_fields = 4; // key, value, message, id  (toggle sfp separately)
        let f = &mut self.state.form;
        match key.code {
            KeyCode::Tab => f.focused_field = (f.focused_field + 1) % (max_fields + 1),
            KeyCode::Esc => { f.query_editing = false; f.focused_field = 0; }
            KeyCode::Char('s') if f.focused_field == 4 => { f.iq_is_sfp_test = !f.iq_is_sfp_test; return; }
            KeyCode::Enter => self.commit_query(),
            KeyCode::Backspace => {
                match f.focused_field.min(max_fields - 1) {
                    0 => { f.iq_key.pop(); }
                    1 => { f.iq_value.pop(); }
                    2 => { f.iq_message.pop(); }
                    3 => { f.iq_id.pop(); }
                    _ => {}
                }
            }
            KeyCode::Char(c) => {
                match f.focused_field.min(max_fields - 1) {
                    0 => { f.iq_key.push(c); }
                    1 => { f.iq_value.push(c); }
                    2 => { f.iq_message.push(c); }
                    3 => { f.iq_id.push(c); }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn commit_query(&mut self) {
        let f = &self.state.form;
        if f.iq_key.trim().is_empty() {
            self.state.set_error("Query key is required");
            return;
        }
        let id: u64 = f.iq_id.parse().unwrap_or(0);
        let val = if f.iq_value.is_empty() {
            json!(null)
        } else if let Ok(n) = f.iq_value.parse::<u64>() {
            json!(n)
        } else {
            json!(f.iq_value)
        };
        let value = transformers::build_interactive_query(&f.iq_key, &val, &f.iq_message, f.iq_is_sfp_test, id);
        let item = ListItem::new(&f.iq_key, value);
        if let Some(idx) = self.state.form.queries_selected {
            self.state.form.interactive_queries[idx] = item;
        } else {
            self.state.form.interactive_queries.push(item);
        }
        self.state.form.query_editing = false;
        self.state.form.focused_field = 0;
        self.sync_list_to_config(FormStep::InteractiveQueries);
        self.state.clear_status();
    }

    // ============================================================
    //  Step 7: Review & Save
    // ============================================================

    fn handle_review_step(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter | KeyCode::Char('s') => self.state.mode = Mode::ConfirmPopup,
            KeyCode::Esc => self.go_back(FormStep::Review),
            _ => {}
        }
    }

    fn handle_confirm_popup_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                if let Err(e) = self.save_config() {
                    self.state.set_error(e);
                    self.state.mode = Mode::Form(FormStep::Review);
                } else {
                    self.state.set_info("Configuration saved successfully!");
                    self.state.mode = Mode::Dashboard;
                }
            }
            KeyCode::Char('n') | KeyCode::Esc => self.state.mode = Mode::Form(FormStep::Review),
            _ => {}
        }
    }

    /// Sync everything and save to disk.
    fn save_config(&mut self) -> Result<(), String> {
        self.sync_basic_info_to_config();
        self.sync_list_to_config(FormStep::OfflineStages);
        self.sync_list_to_config(FormStep::DiagnosticsStages);
        self.sync_list_to_config(FormStep::Cards);
        self.sync_list_to_config(FormStep::InteractiveQueries);

        let pn = self.state.form.part_number.clone();
        let path = fs_svc::config_file_path(&self.info_dir, &pn);
        json_svc::save_config(&path, &self.state.config)
    }

    // ============================================================
    //  Shared list helpers
    // ============================================================

    fn list_prev(&mut self, step: FormStep) {
        match step {
            FormStep::OfflineStages => {
                if let Some(s) = self.state.form.offline_stages_selected {
                    if s > 0 { self.state.form.offline_stages_selected = Some(s - 1); }
                } else if !self.state.form.offline_stages.is_empty() {
                    self.state.form.offline_stages_selected = Some(0);
                }
            }
            FormStep::DiagnosticsStages => {
                if let Some(s) = self.state.form.diagnostics_stages_selected {
                    if s > 0 { self.state.form.diagnostics_stages_selected = Some(s - 1); }
                } else if !self.state.form.diagnostics_stages.is_empty() {
                    self.state.form.diagnostics_stages_selected = Some(0);
                }
            }
            FormStep::Cards => {
                if let Some(s) = self.state.form.cards_selected {
                    if s > 0 { self.state.form.cards_selected = Some(s - 1); }
                } else if !self.state.form.cards.is_empty() {
                    self.state.form.cards_selected = Some(0);
                }
            }
            FormStep::InteractiveQueries => {
                if let Some(s) = self.state.form.queries_selected {
                    if s > 0 { self.state.form.queries_selected = Some(s - 1); }
                } else if !self.state.form.interactive_queries.is_empty() {
                    self.state.form.queries_selected = Some(0);
                }
            }
            _ => {}
        }
    }

    fn list_next(&mut self, step: FormStep, len: usize) {
        let max_idx = len.saturating_sub(1);
        match step {
            FormStep::OfflineStages => {
                let new = self.state.form.offline_stages_selected.unwrap_or(0).saturating_add(1).min(max_idx);
                self.state.form.offline_stages_selected = Some(new);
            }
            FormStep::DiagnosticsStages => {
                let new = self.state.form.diagnostics_stages_selected.unwrap_or(0).saturating_add(1).min(max_idx);
                self.state.form.diagnostics_stages_selected = Some(new);
            }
            FormStep::Cards => {
                let new = self.state.form.cards_selected.unwrap_or(0).saturating_add(1).min(max_idx);
                self.state.form.cards_selected = Some(new);
            }
            FormStep::InteractiveQueries => {
                let new = self.state.form.queries_selected.unwrap_or(0).saturating_add(1).min(max_idx);
                self.state.form.queries_selected = Some(new);
            }
            _ => {}
        }
    }

    /// Sync a list into config["Info"][key].
    fn sync_list_to_config(&mut self, step: FormStep) {
        let key = match step {
            FormStep::OfflineStages => "Offline_Stages",
            FormStep::DiagnosticsStages => "Diagnostics_Stages",
            FormStep::Cards => "cards",
            FormStep::InteractiveQueries => "interactive_queries",
            _ => return,
        };
        let items: Vec<serde_json::Value> = match step {
            FormStep::OfflineStages => self.state.form.offline_stages.iter().map(|i| i.value.clone()).collect(),
            FormStep::DiagnosticsStages => self.state.form.diagnostics_stages.iter().map(|i| i.value.clone()).collect(),
            FormStep::Cards => self.state.form.cards.iter().map(|i| i.value.clone()).collect(),
            FormStep::InteractiveQueries => self.state.form.interactive_queries.iter().map(|i| i.value.clone()).collect(),
            _ => return,
        };
        if let Some(info) = self.state.config.get_mut("Info") {
            if let Some(obj) = info.as_object_mut() {
                obj.insert(key.to_string(), json!(items));
            }
        }
    }

    fn advance(&mut self, current: FormStep) {
        if let Some(next) = FormStep::from_index(current.index() + 1) {
            self.state.mode = Mode::Form(next);
            self.state.form.focused_field = 0;
        }
    }

    fn go_back(&mut self, current: FormStep) {
        if current.index() == 0 {
            self.state.mode = Mode::Dashboard;
        } else if let Some(prev) = FormStep::from_index(current.index() - 1) {
            self.state.mode = Mode::Form(prev);
            self.state.form.focused_field = 0;
        }
    }
}
