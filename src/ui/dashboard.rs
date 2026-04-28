//! UI: Dashboard screen — Futuristic Cyberpunk Design
//!
//! Neon colors, glowing borders, animated-feel ASCII art,
//! pulsing selection indicators, and sci-fi aesthetic.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, List, ListItem, Padding, Paragraph,
};

use crate::ui::theme::*;

const DASHBOARD_ITEMS: &[&str] = &["Info Creator", "Edit Existing Config"];

pub fn render(frame: &mut Frame, selected: usize) {
    let outer = frame.area();

    // Fill background
    let bg = Block::default().style(Style::default().bg(DEEP_BG));
    frame.render_widget(bg, outer);

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(10), // ASCII art banner
            Constraint::Length(2),  // separator line
            Constraint::Min(6),     // menu
            Constraint::Length(3),  // status bar
            Constraint::Length(2),  // footer keybindings
        ])
        .split(outer);

    render_banner(frame, chunks[0]);
    render_separator(frame, chunks[1]);
    render_menu(frame, chunks[2], selected);
    render_status_bar(frame, chunks[3]);
    render_footer(frame, chunks[4]);
}

// ── ASCII Art Banner ────────────────────────────────────────────────

fn render_banner(frame: &mut Frame, area: Rect) {
    let banner_lines = vec![
        Line::from(vec![
            Span::styled("  ╔══════════════════════════════════════════════════════════════╗", Style::default().fg(NEON_CYAN).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  ║  ", Style::default().fg(DIM_CYAN)),
            Span::styled("██████╗ ", Style::default().fg(NEON_MAGENTA).add_modifier(Modifier::BOLD)),
            Span::styled("██████╗ ", Style::default().fg(NEON_CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("██╗   ██╗", Style::default().fg(NEON_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" ██████╗ ", Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(" ███████╗", Style::default().fg(NEON_PURPLE).add_modifier(Modifier::BOLD)),
            Span::styled("  ║", Style::default().fg(DIM_CYAN)),
        ]),
        Line::from(vec![
            Span::styled("  ║  ", Style::default().fg(DIM_CYAN)),
            Span::styled("██╔════╝", Style::default().fg(NEON_MAGENTA).add_modifier(Modifier::BOLD)),
            Span::styled("██╔════╝", Style::default().fg(NEON_CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("██║   ██║", Style::default().fg(NEON_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("██╔═══██╗", Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(" ██╔════╝", Style::default().fg(NEON_PURPLE).add_modifier(Modifier::BOLD)),
            Span::styled("  ║", Style::default().fg(DIM_CYAN)),
        ]),
        Line::from(vec![
            Span::styled("  ║  ", Style::default().fg(DIM_CYAN)),
            Span::styled("██║    ", Style::default().fg(NEON_MAGENTA).add_modifier(Modifier::BOLD)),
            Span::styled("█████╗  ", Style::default().fg(NEON_CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("██║   ██║", Style::default().fg(NEON_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("██║   ██║", Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(" ███████╗", Style::default().fg(NEON_PURPLE).add_modifier(Modifier::BOLD)),
            Span::styled("  ║", Style::default().fg(DIM_CYAN)),
        ]),
        Line::from(vec![
            Span::styled("  ║  ", Style::default().fg(DIM_CYAN)),
            Span::styled("██║    ", Style::default().fg(NEON_MAGENTA).add_modifier(Modifier::BOLD)),
            Span::styled("██╔═██╗ ", Style::default().fg(NEON_CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("╚██╗ ██╔╝", Style::default().fg(NEON_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("██║   ██║", Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(" ╚════██║", Style::default().fg(NEON_PURPLE).add_modifier(Modifier::BOLD)),
            Span::styled("  ║", Style::default().fg(DIM_CYAN)),
        ]),
        Line::from(vec![
            Span::styled("  ║  ", Style::default().fg(DIM_CYAN)),
            Span::styled("██║    ", Style::default().fg(NEON_MAGENTA).add_modifier(Modifier::BOLD)),
            Span::styled("██║  ██╗", Style::default().fg(NEON_CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(" ╚████╔╝ ", Style::default().fg(NEON_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("╚██████╔╝", Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(" ███████║", Style::default().fg(NEON_PURPLE).add_modifier(Modifier::BOLD)),
            Span::styled("  ║", Style::default().fg(DIM_CYAN)),
        ]),
        Line::from(vec![
            Span::styled("  ║  ", Style::default().fg(DIM_CYAN)),
            Span::styled("╚═╝    ", Style::default().fg(NEON_MAGENTA).add_modifier(Modifier::BOLD)),
            Span::styled("╚═══╝  ", Style::default().fg(NEON_CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(" ╚═══╝  ", Style::default().fg(NEON_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" ╚═════╝ ", Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(" ╚══════╝", Style::default().fg(NEON_PURPLE).add_modifier(Modifier::BOLD)),
            Span::styled("  ║", Style::default().fg(DIM_CYAN)),
        ]),
        Line::from(vec![
            Span::styled("  ║  ", Style::default().fg(DIM_CYAN)),
            Span::styled("  C O N F I G   C R E A T O R   v1.0", Style::default().fg(NEON_CYAN).add_modifier(Modifier::BOLD | Modifier::ITALIC)),
            Span::styled("    ║", Style::default().fg(DIM_CYAN)),
        ]),
        Line::from(vec![
            Span::styled("  ╚══════════════════════════════════════════════════════════════╝", Style::default().fg(NEON_CYAN).add_modifier(Modifier::BOLD)),
        ]),
    ];

    let banner = Paragraph::new(banner_lines)
        .style(Style::default().bg(DEEP_BG));
    frame.render_widget(banner, area);
}

// ── Glowing separator ───────────────────────────────────────────────

fn render_separator(frame: &mut Frame, area: Rect) {
    // Use a single-char-wide separator to avoid multi-byte slicing issues.
    // "─" is 3 bytes in UTF-8 but 1 display column wide.
    let width = area.width as usize;
    // Fill the middle with dashes (char-boundary safe via .chars())
    let inner_width = width.saturating_sub(2); // leave room for ◈ on each side
    let dashes: String = "─".repeat(inner_width);
    let spans = vec![
        Span::styled("◈", Style::default().fg(NEON_MAGENTA)),
        Span::styled(dashes, Style::default().fg(DIM_CYAN)),
        Span::styled("◈", Style::default().fg(NEON_MAGENTA)),
    ];
    let sep = Paragraph::new(Line::from(spans)).style(Style::default().bg(DEEP_BG));
    frame.render_widget(sep, area);
}

// ── Neon menu ───────────────────────────────────────────────────────

fn render_menu(frame: &mut Frame, area: Rect, selected: usize) {
    // Build decorated menu items
    let items: Vec<ListItem> = DASHBOARD_ITEMS
        .iter()
        .enumerate()
        .map(|(i, label)| {
            if i == selected {
                // Selected item: bright neon with glow effect
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("  ◆ ", Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("▓▓▓  {}  ▓▓▓", label), Style::default().fg(Color::Black).bg(NEON_CYAN).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![
                        Span::styled("  ◇ ", Style::default().fg(NEON_CYAN)),
                        Span::styled(format!("     {}     ", "─".repeat(label.len())), Style::default().fg(DIM_CYAN)),
                    ]),
                ])
            } else {
                // Unselected: dim neon
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("  ◇ ", Style::default().fg(DIM_MAGENTA)),
                        Span::styled(format!("    {}    ", label), Style::default().fg(NEON_PURPLE)),
                    ]),
                ])
            }
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(" ⚡ ", Style::default().fg(NEON_YELLOW)),
                    Span::styled("SELECT OPERATION", Style::default().fg(NEON_CYAN).add_modifier(Modifier::BOLD)),
                    Span::styled(" ⚡ ", Style::default().fg(NEON_YELLOW)),
                ]))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(NEON_MAGENTA))
                .padding(Padding::uniform(1))
                .style(Style::default().bg(DEEP_BG)),
        );
    frame.render_widget(list, area);
}

// ── Status bar ──────────────────────────────────────────────────────

fn render_status_bar(frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Length(14),
            Constraint::Min(10),
            Constraint::Length(20),
        ])
        .split(area);

    // System status
    let sys = Paragraph::new(Line::from(vec![
        Span::styled(" ▣ ", Style::default().fg(NEON_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled("SYSTEM ONLINE", Style::default().fg(NEON_GREEN)),
    ]))
    .style(Style::default().bg(Color::Rgb(0, 20, 10)));
    frame.render_widget(sys, chunks[0]);

    // Center spacer
    let center = Paragraph::new(Line::from(vec![
        Span::styled("─".repeat(chunks[1].width as usize), Style::default().fg(DIM_CYAN)),
    ]))
    .style(Style::default().bg(DEEP_BG));
    frame.render_widget(center, chunks[1]);

    // Right side info
    let right = Paragraph::new(Line::from(vec![
        Span::styled(" T-MAMS ", Style::default().fg(NEON_ORANGE).add_modifier(Modifier::BOLD)),
        Span::styled("// ", Style::default().fg(DIM_CYAN)),
        Span::styled("DMS INTERFACE", Style::default().fg(NEON_BLUE)),
    ]))
    .style(Style::default().bg(Color::Rgb(10, 0, 20)));
    frame.render_widget(right, chunks[2]);
}

// ── Footer with keybinding hints ────────────────────────────────────

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [", Style::default().fg(NEON_MAGENTA)),
        Span::styled("↑↓", Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled("] Navigate  ", Style::default().fg(DIM_CYAN)),
        Span::styled("[", Style::default().fg(NEON_MAGENTA)),
        Span::styled("Enter", Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled("] Select  ", Style::default().fg(DIM_CYAN)),
        Span::styled("[", Style::default().fg(NEON_MAGENTA)),
        Span::styled("q", Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled("] Quit", Style::default().fg(DIM_CYAN)),
        Span::styled("  ── ", Style::default().fg(DIM_CYAN)),
        Span::styled("⟨ DMS Config Creator ⟩", Style::default().fg(NEON_PURPLE).add_modifier(Modifier::ITALIC)),
    ]))
    .style(Style::default().bg(DEEP_BG));
    frame.render_widget(footer, area);
}
