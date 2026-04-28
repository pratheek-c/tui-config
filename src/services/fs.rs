//! Services: File-system operations
//!
//! All disk I/O is isolated here. Nothing else touches the filesystem.

use std::fs;
use std::path::{Path, PathBuf};

/// Name of the folder where config JSON files are stored.
pub const INFO_JSON_DIR: &str = "INFO_JSON";

/// Ensure the INFO_JSON directory exists in `base_dir`.
pub fn ensure_info_json_dir(base_dir: &Path) -> Result<PathBuf, String> {
    let dir = base_dir.join(INFO_JSON_DIR);
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create {}: {}", INFO_JSON_DIR, e))?;
    }
    Ok(dir)
}

/// List all `.json` files in the INFO_JSON directory.
/// Returns sorted (stem, full_path) tuples.
pub fn list_json_files(info_dir: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    if !info_dir.exists() {
        return Ok(vec![]);
    }
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    let entries = fs::read_dir(info_dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("<unknown>")
                .to_string();
            files.push((stem, path));
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

/// Build the full path for a config file.
pub fn config_file_path(info_dir: &Path, part_number: &str) -> PathBuf {
    info_dir.join(format!("{}.json", part_number))
}

/// Save text content to a file.
pub fn save_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent dir: {}", e))?;
    }
    fs::write(path, content)
        .map_err(|e| format!("Failed to write file: {}", e))
}

/// Read a file's contents as a string.
pub fn load_file(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))
}
