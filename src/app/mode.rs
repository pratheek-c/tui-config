//! App: Navigation mode (state machine).
//!
//! Every screen is a Mode variant. Transitions are driven by user input.

use crate::state::app_state::FormStep;

/// What the file browser is being used for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserPurpose {
    /// Creating new config — shows existing files + "n" to create new.
    Create,
    /// Editing existing config — select a file to load and modify.
    Edit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Dashboard,
    FileBrowser(BrowserPurpose),
    Form(FormStep),
    ConfirmPopup,
    Quit,
}
