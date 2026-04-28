//! Domain: JSON transformers
//!
//! Single source of truth for the DMS config JSON schema.
//! Converts between flat form fields and serde_json::Value.
//! The UI never constructs JSON directly — it goes through here.

use serde_json::{json, Value};

/// Build an empty DMS config template for a given part number.
/// Matches the real schema the DMS system expects.
pub fn default_template(part_number: &str) -> Value {
    json!({
        "Info": {
            "Offline_Stages": [],
            "Programming_Reqd": false,
            "programming_details": {
                "name": "",
                "json": "",
                "stageOrder": ""
            },
            "Card_Setup": {
                "json": ""
            },
            "Diagnostics_Stages": [],
            "Recovery_Stages": [],
            "UCD": {
                "ucdPath": "",
                "ucdAddress": "",
                "tcSeries": "",
                "testcaseId": ""
            },
            "CFPGA": {
                "fileName": "",
                "tclScript": ""
            },
            "DEDI": {
                "dediLocation": "",
                "dediPath": ""
            },
            "T-MAMS": {
                "work_flow_version": "",
                "schema_version": ""
            },
            "interactive_queries": [],
            "cards": [{
                "cardName": "",
                "partNumber": part_number,
                "cardType": "",
                "Parameters": "",
                "cardXml": "",
                "tejDMSxml": "",
                "eeprom_read_pos": 0
            }],
            "intruction_message": ""
        }
    })
}

// ---- Offline Stages ----

/// Build a single Offline Stage.
/// `steps` and `testcases` are full JSON values (arrays).
pub fn build_offline_stage(
    name: &str,
    stage_order: &str,
    tool: &str,
    steps: &Value,
    testcases: &Value,
) -> Value {
    json!({
        "Name": name,
        "stageOrder": stage_order,
        "Tool": tool,
        "steps": steps,
        "Testcases": testcases
    })
}

/// Extract fields from an offline stage Value.
pub fn extract_offline_stage(v: &Value) -> OfflineStageData {
    OfflineStageData {
        name: str_field(v, "Name"),
        stage_order: str_field(v, "stageOrder"),
        tool: str_field(v, "Tool"),
        steps: v.get("steps").cloned().unwrap_or(json!([])),
        testcases: v.get("Testcases").cloned().unwrap_or(json!([])),
    }
}

/// Data held by one offline stage (flat, UI-friendly).
#[derive(Debug, Clone)]
pub struct OfflineStageData {
    pub name: String,
    pub stage_order: String,
    pub tool: String,
    pub steps: Value,
    pub testcases: Value,
}

// ---- Testcases (nested inside offline stages) ----

/// Build a testcase JSON object.
pub fn build_testcase(testcase_id: u64, testcase_name: &str, testcase_position: u64) -> Value {
    json!({
        "testcaseId": testcase_id,
        "testcaseName": testcase_name,
        "testcasePosition": testcase_position
    })
}

/// Extract testcase fields.
pub fn extract_testcase(v: &Value) -> (u64, String, u64) {
    let id = v.get("testcaseId").and_then(|v| v.as_u64()).unwrap_or(0);
    let name = str_field(v, "testcaseName");
    let pos = v.get("testcasePosition").and_then(|v| v.as_u64()).unwrap_or(0);
    (id, name, pos)
}

// ---- Diagnostics Stages ----

/// Build a single Diagnostics Stage.
pub fn build_diagnostics_stage(
    name: &str,
    json_file: &str,
    steps: &Value,
    stage_order: &str,
) -> Value {
    json!({
        "Name": name,
        "Json": json_file,
        "steps": steps,
        "stageOrder": stage_order
    })
}

/// Extract fields from a diagnostics stage Value.
pub fn extract_diagnostics_stage(v: &Value) -> DiagnosticsStageData {
    DiagnosticsStageData {
        name: str_field(v, "Name"),
        json_file: str_field(v, "Json"),
        steps: v.get("steps").cloned().unwrap_or(json!([])),
        stage_order: str_field(v, "stageOrder"),
    }
}

/// Data held by one diagnostics stage.
#[derive(Debug, Clone)]
pub struct DiagnosticsStageData {
    pub name: String,
    pub json_file: String,
    pub steps: Value,
    pub stage_order: String,
}

// ---- Cards ----

/// Build a card JSON object.
pub fn build_card(
    card_name: &str,
    part_number: &str,
    card_type: &str,
    parameters: &str,
    card_xml: &str,
    tej_dms_xml: &str,
    eeprom_read_pos: u64,
) -> Value {
    json!({
        "cardName": card_name,
        "partNumber": part_number,
        "cardType": card_type,
        "Parameters": parameters,
        "cardXml": card_xml,
        "tejDMSxml": tej_dms_xml,
        "eeprom_read_pos": eeprom_read_pos
    })
}

/// Extract card fields.
pub fn extract_card(v: &Value) -> CardData {
    CardData {
        card_name: str_field(v, "cardName"),
        part_number: str_field(v, "partNumber"),
        card_type: str_field(v, "cardType"),
        parameters: str_field(v, "Parameters"),
        card_xml: str_field(v, "cardXml"),
        tej_dms_xml: str_field(v, "tejDMSxml"),
        eeprom_read_pos: v.get("eeprom_read_pos").and_then(|v| v.as_u64()).unwrap_or(0),
    }
}

/// Data held by one card.
#[derive(Debug, Clone)]
pub struct CardData {
    pub card_name: String,
    pub part_number: String,
    pub card_type: String,
    pub parameters: String,
    pub card_xml: String,
    pub tej_dms_xml: String,
    pub eeprom_read_pos: u64,
}

// ---- Interactive Queries ----

/// Build an interactive query JSON object.
pub fn build_interactive_query(
    key: &str,
    value: &Value,
    message: &str,
    is_sfp_test: bool,
    id: u64,
) -> Value {
    json!({
        "key": key,
        "value": value,
        "message": message,
        "isSfpTest": is_sfp_test,
        "id": id
    })
}

/// Extract interactive query fields.
pub fn extract_interactive_query(v: &Value) -> InteractiveQueryData {
    InteractiveQueryData {
        key: str_field(v, "key"),
        value: v.get("value").cloned().unwrap_or(Value::Null),
        message: str_field(v, "message"),
        is_sfp_test: v.get("isSfpTest").and_then(|v| v.as_bool()).unwrap_or(false),
        id: v.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
    }
}

/// Data held by one interactive query.
#[derive(Debug, Clone)]
pub struct InteractiveQueryData {
    pub key: String,
    pub value: Value,
    pub message: String,
    pub is_sfp_test: bool,
    pub id: u64,
}

// ---- Nested config sections (UCD, CFPGA, DEDI, T-MAMS, etc.) ----

/// Extract UCD fields from config.
pub fn extract_ucd(info: &Value) -> (String, String, String, String) {
    let ucd = info.get("UCD").cloned().unwrap_or(json!({}));
    (
        str_field(&ucd, "ucdPath"),
        str_field(&ucd, "ucdAddress"),
        str_field(&ucd, "tcSeries"),
        str_field(&ucd, "testcaseId"),
    )
}

/// Build UCD JSON.
pub fn build_ucd(path: &str, address: &str, series: &str, tc_id: &str) -> Value {
    json!({
        "ucdPath": path,
        "ucdAddress": address,
        "tcSeries": series,
        "testcaseId": tc_id
    })
}

/// Extract CFPGA fields.
pub fn extract_cfpga(info: &Value) -> (String, String) {
    let cfpga = info.get("CFPGA").cloned().unwrap_or(json!({}));
    (
        str_field(&cfpga, "fileName"),
        str_field(&cfpga, "tclScript"),
    )
}

/// Build CFPGA JSON.
pub fn build_cfpga(file_name: &str, tcl_script: &str) -> Value {
    json!({
        "fileName": file_name,
        "tclScript": tcl_script
    })
}

/// Extract DEDI fields.
pub fn extract_dedi(info: &Value) -> (String, String) {
    let dedi = info.get("DEDI").cloned().unwrap_or(json!({}));
    (
        str_field(&dedi, "dediLocation"),
        str_field(&dedi, "dediPath"),
    )
}

/// Build DEDI JSON.
pub fn build_dedi(location: &str, path: &str) -> Value {
    json!({
        "dediLocation": location,
        "dediPath": path
    })
}

/// Extract T-MAMS fields.
pub fn extract_tmams(info: &Value) -> (String, String) {
    let tmams = info.get("T-MAMS").cloned().unwrap_or(json!({}));
    (
        str_field(&tmams, "work_flow_version"),
        str_field(&tmams, "schema_version"),
    )
}

/// Build T-MAMS JSON.
pub fn build_tmams(workflow_ver: &str, schema_ver: &str) -> Value {
    json!({
        "work_flow_version": workflow_ver,
        "schema_version": schema_ver
    })
}

/// Extract programming_details fields.
pub fn extract_programming_details(info: &Value) -> (String, String, String) {
    let pd = info.get("programming_details").cloned().unwrap_or(json!({}));
    (
        str_field(&pd, "name"),
        str_field(&pd, "json"),
        str_field(&pd, "stageOrder"),
    )
}

/// Build programming_details JSON.
pub fn build_programming_details(name: &str, json_file: &str, stage_order: &str) -> Value {
    json!({
        "name": name,
        "json": json_file,
        "stageOrder": stage_order
    })
}

/// Extract the `steps` array from a Value as a comma-separated string
/// for easier editing in a single text field.
pub fn steps_to_string(steps: &Value) -> String {
    steps
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}

/// Parse a comma-separated string back into a JSON array of strings.
pub fn string_to_steps(s: &str) -> Value {
    let arr: Vec<Value> = s
        .split(',')
        .map(|s| json!(s.trim()))
        .filter(|v| v.as_str().map(|s| !s.is_empty()).unwrap_or(false))
        .collect();
    json!(arr)
}

// ---- Helpers ----

fn str_field(v: &Value, key: &str) -> String {
    v.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
}
