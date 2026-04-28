//! UI: Reusable popup component — cyberpunk themed.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::ui::theme::*;

/// Render a centered confirmation popup.
pub fn render_confirm(frame: &mut Frame, message: &str) {
    let area = centered_rect(60, 20, frame.area());
    frame.render_widget(Clear, area);
    let popup = Paragraph::new(message)
        .style(Style::default().fg(NEON_YELLOW).bg(DEEP_BG).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .title(" ⚡ Confirm ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(NEON_CYAN)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(popup, area);
}

/// Render a form popup with multiple fields.
pub fn render_form_popup(frame: &mut Frame, title: &str, fields: &[(&str, &str)], focused: usize) {
    let height = (fields.len() as u16 * 3 + 6).min(frame.area().height.saturating_sub(4));
    let area = centered_rect(75, height, frame.area());
    frame.render_widget(Clear, area);

    let mut lines = String::new();
    for (i, (label, value)) in fields.iter().enumerate() {
        let marker = if i == focused { "▸" } else { " " };
        let cursor = if i == focused { "█" } else { "" };
        lines.push_str(&format!("{} {}: {}{}\n", marker, label, value, cursor));
        if i < fields.len() - 1 {
            lines.push('\n');
        }
    }

    let para = Paragraph::new(lines)
        .style(Style::default().fg(Color::White).bg(DEEP_BG))
        .block(
            Block::default()
                .title(format!(" {} ", title))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(NEON_GREEN)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(para, area);
}

/// Compute a centered rect.
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ratatui::layout::Constraint::Percentage(percent_y),
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ratatui::layout::Constraint::Percentage(percent_x),
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
