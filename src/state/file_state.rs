//! State: File browser state.

use std::path::PathBuf;

pub struct FileState {
    pub files: Vec<(String, PathBuf)>,
    pub selected: usize,
}

impl FileState {
    pub fn new() -> Self {
        Self { files: Vec::new(), selected: 0 }
    }

    pub fn selected_file(&self) -> Option<&(String, PathBuf)> {
        self.files.get(self.selected)
    }

    pub fn select_next(&mut self) {
        if !self.files.is_empty() {
            self.selected = (self.selected + 1).min(self.files.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected > 0 { self.selected -= 1; }
    }
}
