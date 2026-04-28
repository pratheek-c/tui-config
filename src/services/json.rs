//! Services: JSON load / save / template operations.
//!
//! Bridges serde_json with the filesystem service.

use serde_json::Value;
use std::path::Path;

use crate::services::fs;

/// Load a JSON config from disk.
pub fn load_config(path: &Path) -> Result<Value, String> {
    let raw = fs::load_file(path)?;
    serde_json::from_str(&raw)
        .map_err(|e| format!("Invalid JSON in file: {}", e))
}

/// Save a JSON config value (pretty-printed).
pub fn save_config(path: &Path, config: &Value) -> Result<(), String> {
    let pretty = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
    fs::save_file(path, &pretty)
}
