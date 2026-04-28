//! UI: File browser screen — cyberpunk themed.
//!
//! Renders differently based on BrowserPurpose:
//!   - Create: shows existing files + prominent "press n to create new" prompt
//!   - Edit: shows files to select for editing

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use crate::app::mode::BrowserPurpose;
use crate::state::file_state::FileState;
use crate::ui::theme::*;

pub fn render(frame: &mut Frame, fs: &FileState, status: &Option<String>, purpose: BrowserPurpose) {
    // Fill background
    let bg = Block::default().style(Style::default().bg(DEEP_BG));
    frame.render_widget(bg, frame.area());

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(frame.area());

    // Header varies by purpose
    let (title, subtitle) = match purpose {
        BrowserPurpose::Create => (
            "INFO JSON CREATOR",
            "Select existing file to continue editing, or press 'n' to create new",
        ),
        BrowserPurpose::Edit => (
            "EDIT EXISTING CONFIG",
            "Select a file to load and edit",
        ),
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(" ⚡ ", Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled(title, Style::default().fg(NEON_CYAN).add_modifier(Modifier::BOLD)),
        Span::styled(" ─ ", Style::default().fg(DIM_CYAN)),
        Span::styled("INFO_JSON/", Style::default().fg(NEON_PURPLE)),
        Span::styled(" ────────────────────────", Style::default().fg(DIM_CYAN)),
    ]))
    .style(Style::default().bg(DEEP_BG));
    frame.render_widget(header, chunks[0]);

    // File list area
    let border_color = match purpose {
        BrowserPurpose::Create => NEON_GREEN,
        BrowserPurpose::Edit => NEON_MAGENTA,
    };

    let title_icon = match purpose {
        BrowserPurpose::Create => "◈",
        BrowserPurpose::Edit => "◈",
    };
    let title_label = match purpose {
        BrowserPurpose::Create => "SCAN & LIST / CREATE",
        BrowserPurpose::Edit => "SELECT FILE TO EDIT",
    };

    if fs.files.is_empty() {
        let empty_msg = match purpose {
            BrowserPurpose::Create => format!(
                "\n  No JSON files found in INFO_JSON/\n\n  {}\n\n  Press 'n' to create a new configuration.\n  Press Esc to go back.",
                subtitle
            ),
            BrowserPurpose::Edit => String::from(
                "\n  No JSON files found in INFO_JSON/\n\n  Nothing to edit.\n  Press Esc to go back."
            ),
        };
        let empty = Paragraph::new(empty_msg)
            .style(Style::default().fg(NEON_YELLOW).bg(DEEP_BG))
            .block(
                Block::default()
                    .title(Line::from(vec![
                        Span::styled(format!(" {} ", title_icon), Style::default().fg(border_color)),
                        Span::styled(title_label, Style::default().fg(border_color).add_modifier(Modifier::BOLD)),
                    ]))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(empty, chunks[1]);
    } else {
        let items: Vec<ListItem> = fs
            .files
            .iter()
            .enumerate()
            .map(|(i, (name, _path))| {
                if i == fs.selected {
                    ListItem::new(Line::from(Span::styled(
                        format!("  ▸ {}  ", name),
                        Style::default().fg(Color::Black).bg(border_color).add_modifier(Modifier::BOLD),
                    )))
                } else {
                    ListItem::new(Line::from(Span::styled(
                        format!("  ◇ {}  ", name),
                        Style::default().fg(border_color).bg(DEEP_BG),
                    )))
                }
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(Line::from(vec![
                        Span::styled(format!(" {} ", title_icon), Style::default().fg(border_color)),
                        Span::styled(
                            format!("{} ({} found)", title_label, fs.files.len()),
                            Style::default().fg(border_color).add_modifier(Modifier::BOLD),
                        ),
                    ]))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .style(Style::default().bg(DEEP_BG)),
            );
        frame.render_widget(list, chunks[1]);
    }

    // Footer — show different keybindings per purpose
    let status_text = status.as_deref().unwrap_or("");
    if !status_text.is_empty() {
        let style = if status_text.starts_with("ERROR") {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD).bg(DEEP_BG)
        } else {
            Style::default().fg(NEON_YELLOW).bg(DEEP_BG)
        };
        let footer = Paragraph::new(status_text).style(style);
        frame.render_widget(footer, chunks[2]);
    } else {
        let footer_spans = match purpose {
            BrowserPurpose::Create => vec![
                Span::styled(" [", Style::default().fg(NEON_MAGENTA)),
                Span::styled("↑↓", Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
                Span::styled("] Navigate  ", Style::default().fg(DIM_TEXT)),
                Span::styled("[", Style::default().fg(NEON_MAGENTA)),
                Span::styled("Enter", Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
                Span::styled("] Open  ", Style::default().fg(DIM_TEXT)),
                Span::styled("[", Style::default().fg(NEON_MAGENTA)),
                Span::styled("n", Style::default().fg(NEON_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("] Create New  ", Style::default().fg(DIM_TEXT)),
                Span::styled("[", Style::default().fg(NEON_MAGENTA)),
                Span::styled("Esc", Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
                Span::styled("] Back", Style::default().fg(DIM_TEXT)),
            ],
            BrowserPurpose::Edit => vec![
                Span::styled(" [", Style::default().fg(NEON_MAGENTA)),
                Span::styled("↑↓", Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
                Span::styled("] Navigate  ", Style::default().fg(DIM_TEXT)),
                Span::styled("[", Style::default().fg(NEON_MAGENTA)),
                Span::styled("Enter", Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
                Span::styled("] Open & Edit  ", Style::default().fg(DIM_TEXT)),
                Span::styled("[", Style::default().fg(NEON_MAGENTA)),
                Span::styled("Esc", Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
                Span::styled("] Back", Style::default().fg(DIM_TEXT)),
            ],
        };
        let footer = Paragraph::new(Line::from(footer_spans)).style(Style::default().bg(DEEP_BG));
        frame.render_widget(footer, chunks[2]);
    }
}
