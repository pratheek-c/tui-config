//! UI: Shared cyberpunk theme constants.
//!
//! Single source of truth for all neon colors and styling.
//! All UI modules should reference these instead of defining their own.

use ratatui::style::Color;

// ── Neon accent colors ──────────────────────────────────────────────
pub const NEON_CYAN: Color = Color::Rgb(0, 255, 255);
pub const NEON_MAGENTA: Color = Color::Rgb(255, 0, 255);
pub const NEON_PINK: Color = Color::Rgb(255, 20, 147);
pub const NEON_GREEN: Color = Color::Rgb(57, 255, 20);
pub const NEON_YELLOW: Color = Color::Rgb(255, 255, 0);
pub const NEON_PURPLE: Color = Color::Rgb(180, 0, 255);
pub const NEON_ORANGE: Color = Color::Rgb(255, 165, 0);
pub const NEON_BLUE: Color = Color::Rgb(0, 150, 255);

// ── Backgrounds & dim colors ────────────────────────────────────────
pub const DEEP_BG: Color = Color::Rgb(10, 0, 20);
pub const DIM_CYAN: Color = Color::Rgb(0, 80, 100);
pub const DIM_MAGENTA: Color = Color::Rgb(80, 0, 80);
pub const DIM_TEXT: Color = Color::Rgb(100, 100, 120);
