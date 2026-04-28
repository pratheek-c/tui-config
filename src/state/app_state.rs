//! State: Application state + FormStep enum.

use serde_json::Value;

use crate::app::mode::Mode;
use crate::state::form_state::FormState;

/// Tracks which form step the user is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormStep {
    PartNumber,
    BasicInfo,          // UCD, CFPGA, DEDI, T-MAMS, programming, card_setup, instruction
    OfflineStages,
    DiagnosticsStages,
    Cards,
    InteractiveQueries,
    Review,
}

impl FormStep {
    pub fn all() -> &'static [FormStep] {
        &[
            FormStep::PartNumber,
            FormStep::BasicInfo,
            FormStep::OfflineStages,
            FormStep::DiagnosticsStages,
            FormStep::Cards,
            FormStep::InteractiveQueries,
            FormStep::Review,
        ]
    }

    pub fn index(self) -> usize {
        Self::all().iter().position(|s| *s == self).unwrap_or(0)
    }

    pub fn from_index(i: usize) -> Option<Self> {
        Self::all().get(i).copied()
    }

    pub fn title(self) -> &'static str {
        match self {
            FormStep::PartNumber => "Part Number",
            FormStep::BasicInfo => "Basic Info & Hardware Config",
            FormStep::OfflineStages => "Offline Stages",
            FormStep::DiagnosticsStages => "Diagnostics Stages",
            FormStep::Cards => "Cards",
            FormStep::InteractiveQueries => "Interactive Queries",
            FormStep::Review => "Review & Save",
        }
    }
}

/// The top-level application state.
pub struct AppState {
    /// Current navigation mode.
    pub mode: Mode,
    /// Form state.
    pub form: FormState,
    /// Working JSON config.
    pub config: Value,
    /// Status/error message.
    pub status_message: Option<String>,
    /// Editing an existing file?
    pub is_editing: bool,
    /// File path being edited.
    pub editing_file_path: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mode: Mode::Dashboard,
            form: FormState::new(),
            config: Value::Null,
            status_message: None,
            is_editing: false,
            editing_file_path: None,
        }
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.status_message = Some(format!("ERROR: {}", msg.into()));
    }

    pub fn set_info(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    pub fn reset_for_new(&mut self) {
        self.form = FormState::new();
        self.config = Value::Null;
        self.is_editing = false;
        self.editing_file_path = None;
        self.status_message = None;
        self.mode = Mode::Form(FormStep::PartNumber);
    }

    pub fn reset_for_edit(&mut self, config: Value, file_path: String) {
        self.form = FormState::from_config(&config);
        self.config = config;
        self.is_editing = true;
        self.editing_file_path = Some(file_path);
        self.status_message = None;
        self.mode = Mode::Form(FormStep::BasicInfo);
    }
}
