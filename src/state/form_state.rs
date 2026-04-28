//! State: Form state — all editable fields for the multi-step form.
//!
//! Each field is a String or Vec — no JSON construction here.
//! Transformers convert between this and JSON.

use serde_json::{json, Value};

use crate::domain::transformers;

/// An item in a dynamic list (offline stage, card, query, etc.)
#[derive(Debug, Clone)]
pub struct ListItem {
    /// Display label.
    pub name: String,
    /// Underlying JSON value.
    pub value: Value,
}

impl ListItem {
    pub fn new(name: impl Into<String>, value: Value) -> Self {
        Self { name: name.into(), value }
    }
}

/// The form state — one struct holding every editable field across all steps.
pub struct FormState {
    // --- Step 1: Part Number ---
    pub part_number: String,

    // --- Step 2: Basic Info & Hardware Config ---
    // UCD
    pub ucd_path: String,
    pub ucd_address: String,
    pub ucd_tc_series: String,
    pub ucd_testcase_id: String,
    // CFPGA
    pub cfpga_file_name: String,
    pub cfpga_tcl_script: String,
    // DEDI
    pub dedi_location: String,
    pub dedi_path: String,
    // T-MAMS
    pub tmams_workflow_version: String,
    pub tmams_schema_version: String,
    // Programming details
    pub programming_reqd: bool,
    pub prog_name: String,
    pub prog_json: String,
    pub prog_stage_order: String,
    // Card Setup
    pub card_setup_json: String,
    // Instruction message
    pub instruction_message: String,

    // --- Step 3: Offline Stages ---
    pub offline_stages: Vec<ListItem>,
    pub offline_stages_selected: Option<usize>,
    pub offline_stage_editing: bool,
    // Sub-form fields for offline stage
    pub os_name: String,
    pub os_stage_order: String,
    pub os_tool: String,
    pub os_steps: String, // comma-separated
    // Testcase sub-form (nested inside offline stage editing)
    pub os_testcases: Vec<ListItem>,
    pub os_testcases_selected: Option<usize>,
    pub os_testcase_editing: bool,
    pub tc_id: String,
    pub tc_name: String,
    pub tc_position: String,

    // --- Step 4: Diagnostics Stages ---
    pub diagnostics_stages: Vec<ListItem>,
    pub diagnostics_stages_selected: Option<usize>,
    pub diagnostics_stage_editing: bool,
    pub ds_name: String,
    pub ds_json: String,
    pub ds_steps: String,
    pub ds_stage_order: String,

    // --- Step 5: Cards ---
    pub cards: Vec<ListItem>,
    pub cards_selected: Option<usize>,
    pub card_editing: bool,
    pub card_name: String,
    pub card_part_number: String,
    pub card_type: String,
    pub card_parameters: String,
    pub card_xml: String,
    pub card_tej_dms_xml: String,
    pub card_eeprom_read_pos: String,

    // --- Step 6: Interactive Queries ---
    pub interactive_queries: Vec<ListItem>,
    pub queries_selected: Option<usize>,
    pub query_editing: bool,
    pub iq_key: String,
    pub iq_value: String,
    pub iq_message: String,
    pub iq_is_sfp_test: bool,
    pub iq_id: String,

    /// Which input field is focused in the current step/sub-form.
    pub focused_field: usize,
}

impl FormState {
    pub fn new() -> Self {
        Self {
            part_number: String::new(),
            ucd_path: String::new(),
            ucd_address: String::new(),
            ucd_tc_series: String::new(),
            ucd_testcase_id: String::new(),
            cfpga_file_name: String::new(),
            cfpga_tcl_script: String::new(),
            dedi_location: String::new(),
            dedi_path: String::new(),
            tmams_workflow_version: String::new(),
            tmams_schema_version: String::new(),
            programming_reqd: false,
            prog_name: String::new(),
            prog_json: String::new(),
            prog_stage_order: String::new(),
            card_setup_json: String::new(),
            instruction_message: String::new(),
            offline_stages: Vec::new(),
            offline_stages_selected: None,
            offline_stage_editing: false,
            os_name: String::new(),
            os_stage_order: String::new(),
            os_tool: String::new(),
            os_steps: String::new(),
            os_testcases: Vec::new(),
            os_testcases_selected: None,
            os_testcase_editing: false,
            tc_id: String::new(),
            tc_name: String::new(),
            tc_position: String::new(),
            diagnostics_stages: Vec::new(),
            diagnostics_stages_selected: None,
            diagnostics_stage_editing: false,
            ds_name: String::new(),
            ds_json: String::new(),
            ds_steps: String::new(),
            ds_stage_order: String::new(),
            cards: Vec::new(),
            cards_selected: None,
            card_editing: false,
            card_name: String::new(),
            card_part_number: String::new(),
            card_type: String::new(),
            card_parameters: String::new(),
            card_xml: String::new(),
            card_tej_dms_xml: String::new(),
            card_eeprom_read_pos: String::new(),
            interactive_queries: Vec::new(),
            queries_selected: None,
            query_editing: false,
            iq_key: String::new(),
            iq_value: String::new(),
            iq_message: String::new(),
            iq_is_sfp_test: false,
            iq_id: String::new(),
            focused_field: 0,
        }
    }

    /// Populate form fields from an existing JSON config.
    pub fn from_config(config: &Value) -> Self {
        let info = config.get("Info").cloned().unwrap_or(json!({}));

        let part_number = config
            .get("Info")
            .and_then(|v| v.get("cards"))
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|c| c.get("partNumber"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let (ucd_path, ucd_address, ucd_tc_series, ucd_testcase_id) =
            transformers::extract_ucd(&info);
        let (cfpga_file_name, cfpga_tcl_script) = transformers::extract_cfpga(&info);
        let (dedi_location, dedi_path) = transformers::extract_dedi(&info);
        let (tmams_workflow_version, tmams_schema_version) = transformers::extract_tmams(&info);
        let (prog_name, prog_json, prog_stage_order) =
            transformers::extract_programming_details(&info);

        let programming_reqd = info
            .get("Programming_Reqd")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let card_setup_json = info
            .get("Card_Setup")
            .and_then(|v| v.get("json"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let instruction_message = str_field(&info, "intruction_message");

        let offline_stages = extract_items(&info, "Offline_Stages", "Name");
        let diagnostics_stages = extract_items(&info, "Diagnostics_Stages", "Name");
        let cards = extract_items(&info, "cards", "cardName");
        let interactive_queries = extract_items(&info, "interactive_queries", "key");

        Self {
            part_number,
            ucd_path,
            ucd_address,
            ucd_tc_series,
            ucd_testcase_id,
            cfpga_file_name,
            cfpga_tcl_script,
            dedi_location,
            dedi_path,
            tmams_workflow_version,
            tmams_schema_version,
            programming_reqd,
            prog_name,
            prog_json,
            prog_stage_order,
            card_setup_json,
            instruction_message,
            offline_stages,
            offline_stages_selected: None,
            offline_stage_editing: false,
            os_name: String::new(),
            os_stage_order: String::new(),
            os_tool: String::new(),
            os_steps: String::new(),
            os_testcases: Vec::new(),
            os_testcases_selected: None,
            os_testcase_editing: false,
            tc_id: String::new(),
            tc_name: String::new(),
            tc_position: String::new(),
            diagnostics_stages,
            diagnostics_stages_selected: None,
            diagnostics_stage_editing: false,
            ds_name: String::new(),
            ds_json: String::new(),
            ds_steps: String::new(),
            ds_stage_order: String::new(),
            cards,
            cards_selected: None,
            card_editing: false,
            card_name: String::new(),
            card_part_number: String::new(),
            card_type: String::new(),
            card_parameters: String::new(),
            card_xml: String::new(),
            card_tej_dms_xml: String::new(),
            card_eeprom_read_pos: String::new(),
            interactive_queries,
            queries_selected: None,
            query_editing: false,
            iq_key: String::new(),
            iq_value: String::new(),
            iq_message: String::new(),
            iq_is_sfp_test: false,
            iq_id: String::new(),
            focused_field: 0,
        }
    }

    /// Clear all offline-stage sub-form fields.
    pub fn clear_offline_stage_fields(&mut self) {
        self.os_name.clear();
        self.os_stage_order.clear();
        self.os_tool.clear();
        self.os_steps.clear();
        self.os_testcases.clear();
        self.os_testcases_selected = None;
        self.os_testcase_editing = false;
    }

    /// Clear testcase sub-form fields.
    pub fn clear_testcase_fields(&mut self) {
        self.tc_id.clear();
        self.tc_name.clear();
        self.tc_position.clear();
    }

    /// Clear diagnostics stage sub-form fields.
    pub fn clear_diagnostics_stage_fields(&mut self) {
        self.ds_name.clear();
        self.ds_json.clear();
        self.ds_steps.clear();
        self.ds_stage_order.clear();
    }

    /// Clear card sub-form fields.
    pub fn clear_card_fields(&mut self) {
        self.card_name.clear();
        self.card_part_number.clear();
        self.card_type.clear();
        self.card_parameters.clear();
        self.card_xml.clear();
        self.card_tej_dms_xml.clear();
        self.card_eeprom_read_pos.clear();
    }

    /// Clear interactive query sub-form fields.
    pub fn clear_query_fields(&mut self) {
        self.iq_key.clear();
        self.iq_value.clear();
        self.iq_message.clear();
        self.iq_is_sfp_test = false;
        self.iq_id.clear();
    }
}

fn str_field(v: &Value, key: &str) -> String {
    v.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
}

fn extract_items(config: &Value, field: &str, name_key: &str) -> Vec<ListItem> {
    config
        .get(field)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|item| {
                    let name = item
                        .get(name_key)
                        .and_then(|v| v.as_str())
                        .unwrap_or("<unnamed>")
                        .to_string();
                    ListItem::new(name, item.clone())
                })
                .collect()
        })
        .unwrap_or_default()
}
