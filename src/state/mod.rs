//! State module — all application state types.

pub mod app_state;
pub mod file_state;
pub mod form_state;

// Re-export for convenience — used by app::app and ui modules.
#[allow(unused_imports)]
pub use app_state::{AppState, FormStep};
#[allow(unused_imports)]
pub use file_state::FileState;
#[allow(unused_imports)]
pub use form_state::FormState;
