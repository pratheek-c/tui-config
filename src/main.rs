//! Main entry point.
//!
//! Sets up the terminal, runs the event loop, and tears down cleanly.
//! Uses ratatui's crossterm backend.

mod app;
mod domain;
mod services;
mod state;
mod ui;

use std::io;

use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ---- Setup terminal ----
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // ---- Create app ----
    let mut app = match App::new() {
        Ok(a) => a,
        Err(e) => {
            disable_raw_mode()?;
            eprintln!("Failed to initialize app: {}", e);
            return Err(e.into());
        }
    };

    // ---- Event loop ----
    while !app.should_quit {
        terminal.draw(|f| app.render(f))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press — ignore release/repeat to prevent
                // double-entry on platforms that emit both events.
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
        }
    }

    // ---- Restore terminal ----
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
