//! UI: Form stepper — renders the appropriate form step.
//!
//! This is a pure renderer: it reads from AppState and draws widgets.
//! It never mutates state or performs I/O.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use crate::state::app_state::{AppState, FormStep};
use crate::ui::theme::*;

pub fn render(frame: &mut Frame, state: &AppState, step: FormStep) {
    // Fill background
    let bg = Block::default().style(Style::default().bg(DEEP_BG));
    frame.render_widget(bg, frame.area());

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // header + step indicator
            Constraint::Length(1),  // step bar
            Constraint::Min(5),    // main content
            Constraint::Length(2), // footer
        ])
        .split(frame.area());

    render_step_header(frame, chunks[0], step);
    render_step_bar(frame, chunks[1], step);
    render_step_content(frame, chunks[2], state, step);
    render_footer(frame, chunks[3], state, step);
}

fn render_step_header(frame: &mut Frame, area: Rect, step: FormStep) {
    let total = FormStep::all().len();
    let idx = step.index() + 1;

    let header = Paragraph::new(Line::from(vec![
        Span::styled(" ⚡ ", Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("STEP {}/{} ", idx, total),
            Style::default().fg(Color::Black).bg(NEON_CYAN).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ─ ", Style::default().fg(DIM_CYAN)),
        Span::styled(
            format!(" {} ", step.title()),
            Style::default().fg(NEON_CYAN).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ──────────────────────────────────────", Style::default().fg(DIM_CYAN)),
    ]))
    .style(Style::default().bg(DEEP_BG));
    frame.render_widget(header, area);
}

fn render_step_bar(frame: &mut Frame, area: Rect, step: FormStep) {
    let step_bar: String = FormStep::all()
        .iter()
        .enumerate()
        .map(|(i, s)| {
            if i == step.index() {
                format!("▸{}◂", s.title())
            } else if i < step.index() {
                format!("✓{}", s.title())
            } else {
                format!(" {} ", s.title())
            }
        })
        .collect::<Vec<_>>()
        .join("──");

    let bar = Paragraph::new(step_bar)
        .style(Style::default().fg(DIM_CYAN).bg(DEEP_BG));
    frame.render_widget(bar, area);
}

fn render_step_content(frame: &mut Frame, area: Rect, state: &AppState, step: FormStep) {
    match step {
        FormStep::PartNumber => render_part_number(frame, area, state),
        FormStep::BasicInfo => render_basic_info(frame, area, state),
        FormStep::OfflineStages => render_offline_stages(frame, area, state),
        FormStep::DiagnosticsStages => render_diagnostics_stages(frame, area, state),
        FormStep::Cards => render_cards(frame, area, state),
        FormStep::InteractiveQueries => render_interactive_queries(frame, area, state),
        FormStep::Review => render_review(frame, area, state),
    }
}

// ── Step 1: Part Number ─────────────────────────────────────────────

fn render_part_number(frame: &mut Frame, area: Rect, state: &AppState) {
    let f = &state.form;
    let cursor = "█";
    let text = format!(
        "\n  Part Number: {}{}\n\n  Format: ###-PCA######-X\n  Example: 127-PCA000097-E\n\n  Type the part number and press Enter to continue.",
        f.part_number, cursor
    );
    let para = Paragraph::new(text)
        .style(Style::default().fg(NEON_CYAN).bg(DEEP_BG))
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(" ◈ ", Style::default().fg(NEON_MAGENTA)),
                    Span::styled("ENTER PART NUMBER", Style::default().fg(NEON_CYAN).add_modifier(Modifier::BOLD)),
                ]))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(NEON_MAGENTA)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(para, area);
}

// ── Step 2: Basic Info ──────────────────────────────────────────────

fn render_basic_info(frame: &mut Frame, area: Rect, state: &AppState) {
    let f = &state.form;
    let focused = f.focused_field;

    let fields = [
        ("◈ UCD Path", &f.ucd_path, NEON_CYAN),
        ("◈ UCD Address", &f.ucd_address, NEON_CYAN),
        ("◈ UCD TC Series", &f.ucd_tc_series, NEON_CYAN),
        ("◈ UCD Testcase ID", &f.ucd_testcase_id, NEON_CYAN),
        ("◈ CFPGA File Name", &f.cfpga_file_name, NEON_GREEN),
        ("◈ CFPGA TCL Script", &f.cfpga_tcl_script, NEON_GREEN),
        ("◈ DEDI Location", &f.dedi_location, NEON_ORANGE),
        ("◈ DEDI Path", &f.dedi_path, NEON_ORANGE),
        ("◈ T-MAMS Workflow Ver", &f.tmams_workflow_version, NEON_PURPLE),
        ("◈ T-MAMS Schema Ver", &f.tmams_schema_version, NEON_PURPLE),
        ("◈ Prog Name", &f.prog_name, NEON_PINK),
        ("◈ Prog JSON", &f.prog_json, NEON_PINK),
        ("◈ Prog Stage Order", &f.prog_stage_order, NEON_PINK),
        ("◈ Card Setup JSON", &f.card_setup_json, NEON_BLUE),
        ("◈ Instruction Message", &f.instruction_message, NEON_YELLOW),
    ];

    let mut lines: Vec<Line> = Vec::new();
    for (i, (label, value, color)) in fields.iter().enumerate() {
        let marker = if i == focused { "▸" } else { " " };
        let cursor = if i == focused { "█" } else { "" };
        let (fg, bg) = if i == focused {
            (Color::Black, *color)
        } else {
            (*color, DEEP_BG)
        };
        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", marker), Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{}: ", label), Style::default().fg(*color)),
            Span::styled(format!("{}{}", value, cursor), Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD)),
        ]));
    }

    // Programming_Reqd toggle (field 15)
    {
        let prog_label = if f.programming_reqd { "[●] Programming Required" } else { "[○] Programming Required" };
        let marker = if focused == 15 { "▸" } else { " " };
        let (fg, bg) = if focused == 15 {
            (Color::Black, NEON_MAGENTA)
        } else {
            (NEON_MAGENTA, DEEP_BG)
        };
        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", marker), Style::default().fg(NEON_YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(prog_label, Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD)),
            Span::styled("  (Space to toggle)", Style::default().fg(DIM_TEXT)),
        ]));
    }

    let para = Paragraph::new(lines)
        .style(Style::default().bg(DEEP_BG))
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(" ⚙ ", Style::default().fg(NEON_GREEN)),
                    Span::styled("HARDWARE CONFIGURATION", Style::default().fg(NEON_GREEN).add_modifier(Modifier::BOLD)),
                ]))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(NEON_GREEN)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(para, area);
}

// ── Step 3: Offline Stages ──────────────────────────────────────────

fn render_offline_stages(frame: &mut Frame, area: Rect, state: &AppState) {
    let f = &state.form;

    if f.offline_stage_editing {
        render_offline_stage_list(frame, area, state);
        if f.os_testcase_editing {
            let fields = [
                ("Testcase ID", f.tc_id.as_str()),
                ("Testcase Name", f.tc_name.as_str()),
                ("Position", f.tc_position.as_str()),
            ];
            crate::ui::components::popup::render_form_popup(
                frame, "⟨ Add/Edit Testcase ⟩", &fields, f.focused_field,
            );
        } else {
            let tc_count = f.os_testcases.len();
            let tc_summary: String = f.os_testcases.iter()
                .map(|t| t.name.clone())
                .collect::<Vec<_>>()
                .join("\n    ");
            let all_fields = format!(
                "Name: {}\nStage Order: {}\nTool: {}\nSteps (comma-sep): {}\n\nTestcases ({}):\n    {}\n\n[Press 't' to add testcase]",
                f.os_name, f.os_stage_order, f.os_tool, f.os_steps, tc_count,
                if tc_summary.is_empty() { "(none)" } else { &tc_summary }
            );
            crate::ui::components::popup::render_form_popup(
                frame, "⟨ Add/Edit Offline Stage ⟩", &[("Details", all_fields.as_str())], 0,
            );
        }
        return;
    }

    render_offline_stage_list(frame, area, state);
}

fn render_offline_stage_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let f = &state.form;
    if f.offline_stages.is_empty() {
        let para = Paragraph::new("\n  No offline stages configured.\n\n  Press 'a' to add a new stage.")
            .style(Style::default().fg(NEON_YELLOW).bg(DEEP_BG))
            .block(
                Block::default()
                    .title(Line::from(vec![
                        Span::styled(" ◈ ", Style::default().fg(NEON_ORANGE)),
                        Span::styled("OFFLINE STAGES", Style::default().fg(NEON_ORANGE).add_modifier(Modifier::BOLD)),
                    ]))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(NEON_ORANGE)),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(para, area);
    } else {
        let items: Vec<ListItem> = f.offline_stages.iter().enumerate().map(|(i, item)| {
            let sel = f.offline_stages_selected == Some(i);
            let order = item.value.get("stageOrder").and_then(|v| v.as_str()).unwrap_or("?");
            let tool = item.value.get("Tool").and_then(|v| v.as_str()).unwrap_or("?");
            let tc_count = item.value.get("Testcases").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);

            if sel {
                ListItem::new(Line::from(Span::styled(
                    format!("  ▸ #{} {} │ tool:{} │ {} testcases", order, item.name, tool, tc_count),
                    Style::default().fg(Color::Black).bg(NEON_ORANGE).add_modifier(Modifier::BOLD),
                )))
            } else {
                ListItem::new(Line::from(Span::styled(
                    format!("  ◇ #{} {} │ tool:{} │ {} testcases", order, item.name, tool, tc_count),
                    Style::default().fg(NEON_ORANGE).bg(DEEP_BG),
                )))
            }
        }).collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(Line::from(vec![
                        Span::styled(" ◈ ", Style::default().fg(NEON_ORANGE)),
                        Span::styled(format!("OFFLINE STAGES ({})", f.offline_stages.len()), Style::default().fg(NEON_ORANGE).add_modifier(Modifier::BOLD)),
                    ]))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(NEON_ORANGE))
                    .style(Style::default().bg(DEEP_BG)),
            );
        frame.render_widget(list, area);
    }
}

// ── Step 4: Diagnostics Stages ──────────────────────────────────────

fn render_diagnostics_stages(frame: &mut Frame, area: Rect, state: &AppState) {
    let f = &state.form;

    if f.diagnostics_stage_editing {
        render_diag_list(frame, area, state);
        let fields = [
            ("Name", f.ds_name.as_str()),
            ("JSON File", f.ds_json.as_str()),
            ("Steps (comma-sep)", f.ds_steps.as_str()),
            ("Stage Order", f.ds_stage_order.as_str()),
        ];
        crate::ui::components::popup::render_form_popup(
            frame, "⟨ Add/Edit Diagnostics Stage ⟩", &fields, f.focused_field,
        );
        return;
    }

    render_diag_list(frame, area, state);
}

fn render_diag_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let f = &state.form;
    if f.diagnostics_stages.is_empty() {
        let para = Paragraph::new("\n  No diagnostics stages configured.\n\n  Press 'a' to add a new stage.")
            .style(Style::default().fg(NEON_YELLOW).bg(DEEP_BG))
            .block(
                Block::default()
                    .title(Line::from(vec![
                        Span::styled(" ◈ ", Style::default().fg(NEON_PINK)),
                        Span::styled("DIAGNOSTICS STAGES", Style::default().fg(NEON_PINK).add_modifier(Modifier::BOLD)),
                    ]))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(NEON_PINK)),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(para, area);
    } else {
        let items: Vec<ListItem> = f.diagnostics_stages.iter().enumerate().map(|(i, item)| {
            let sel = f.diagnostics_stages_selected == Some(i);
            let order = item.value.get("stageOrder").and_then(|v| v.as_str()).unwrap_or("?");
            if sel {
                ListItem::new(Line::from(Span::styled(
                    format!("  ▸ #{} {}", order, item.name),
                    Style::default().fg(Color::Black).bg(NEON_PINK).add_modifier(Modifier::BOLD),
                )))
            } else {
                ListItem::new(Line::from(Span::styled(
                    format!("  ◇ #{} {}", order, item.name),
                    Style::default().fg(NEON_PINK).bg(DEEP_BG),
                )))
            }
        }).collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(Line::from(vec![
                        Span::styled(" ◈ ", Style::default().fg(NEON_PINK)),
                        Span::styled(format!("DIAGNOSTICS STAGES ({})", f.diagnostics_stages.len()), Style::default().fg(NEON_PINK).add_modifier(Modifier::BOLD)),
                    ]))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(NEON_PINK))
                    .style(Style::default().bg(DEEP_BG)),
            );
        frame.render_widget(list, area);
    }
}

// ── Step 5: Cards ───────────────────────────────────────────────────

fn render_cards(frame: &mut Frame, area: Rect, state: &AppState) {
    let f = &state.form;

    if f.card_editing {
        render_cards_list(frame, area, state);
        let fields = [
            ("Card Name", f.card_name.as_str()),
            ("Part Number", f.card_part_number.as_str()),
            ("Card Type", f.card_type.as_str()),
            ("Parameters", f.card_parameters.as_str()),
            ("Card XML", f.card_xml.as_str()),
            ("TejDMS XML", f.card_tej_dms_xml.as_str()),
            ("EEPROM Read Pos", f.card_eeprom_read_pos.as_str()),
        ];
        crate::ui::components::popup::render_form_popup(
            frame, "⟨ Add/Edit Card ⟩", &fields, f.focused_field,
        );
        return;
    }

    render_cards_list(frame, area, state);
}

fn render_cards_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let f = &state.form;
    if f.cards.is_empty() {
        let para = Paragraph::new("\n  No cards configured.\n\n  Press 'a' to add a card.")
            .style(Style::default().fg(NEON_YELLOW).bg(DEEP_BG))
            .block(
                Block::default()
                    .title(Line::from(vec![
                        Span::styled(" ◈ ", Style::default().fg(NEON_BLUE)),
                        Span::styled("CARDS", Style::default().fg(NEON_BLUE).add_modifier(Modifier::BOLD)),
                    ]))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(NEON_BLUE)),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(para, area);
    } else {
        let items: Vec<ListItem> = f.cards.iter().enumerate().map(|(i, item)| {
            let sel = f.cards_selected == Some(i);
            let ct = item.value.get("cardType").and_then(|v| v.as_str()).unwrap_or("?");
            let pn = item.value.get("partNumber").and_then(|v| v.as_str()).unwrap_or("?");
            if sel {
                ListItem::new(Line::from(Span::styled(
                    format!("  ▸ {} [{}] type={}", item.name, pn, ct),
                    Style::default().fg(Color::Black).bg(NEON_BLUE).add_modifier(Modifier::BOLD),
                )))
            } else {
                ListItem::new(Line::from(Span::styled(
                    format!("  ◇ {} [{}] type={}", item.name, pn, ct),
                    Style::default().fg(NEON_BLUE).bg(DEEP_BG),
                )))
            }
        }).collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(Line::from(vec![
                        Span::styled(" ◈ ", Style::default().fg(NEON_BLUE)),
                        Span::styled(format!("CARDS ({})", f.cards.len()), Style::default().fg(NEON_BLUE).add_modifier(Modifier::BOLD)),
                    ]))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(NEON_BLUE))
                    .style(Style::default().bg(DEEP_BG)),
            );
        frame.render_widget(list, area);
    }
}

// ── Step 6: Interactive Queries ─────────────────────────────────────

fn render_interactive_queries(frame: &mut Frame, area: Rect, state: &AppState) {
    let f = &state.form;

    if f.query_editing {
        render_queries_list(frame, area, state);
        let sfp_label = if f.iq_is_sfp_test { "[●] SFP Test" } else { "[○] SFP Test" };
        let fields = [
            ("Key", f.iq_key.as_str()),
            ("Value", f.iq_value.as_str()),
            ("Message", f.iq_message.as_str()),
            ("ID", f.iq_id.as_str()),
            (sfp_label, "(press 's' to toggle)"),
        ];
        crate::ui::components::popup::render_form_popup(
            frame, "⟨ Add/Edit Interactive Query ⟩", &fields, f.focused_field,
        );
        return;
    }

    render_queries_list(frame, area, state);
}

fn render_queries_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let f = &state.form;
    if f.interactive_queries.is_empty() {
        let para = Paragraph::new("\n  No interactive queries configured.\n\n  Press 'a' to add a query.")
            .style(Style::default().fg(NEON_YELLOW).bg(DEEP_BG))
            .block(
                Block::default()
                    .title(Line::from(vec![
                        Span::styled(" ◈ ", Style::default().fg(NEON_PURPLE)),
                        Span::styled("INTERACTIVE QUERIES", Style::default().fg(NEON_PURPLE).add_modifier(Modifier::BOLD)),
                    ]))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(NEON_PURPLE)),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(para, area);
    } else {
        let items: Vec<ListItem> = f.interactive_queries.iter().enumerate().map(|(i, item)| {
            let sel = f.queries_selected == Some(i);
            let id = item.value.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
            let msg = item.value.get("message").and_then(|v| v.as_str()).unwrap_or("");
            let sfp = item.value.get("isSfpTest").and_then(|v| v.as_bool()).unwrap_or(false);
            let tag = if sfp { " [SFP]" } else { "" };
            if sel {
                ListItem::new(Line::from(Span::styled(
                    format!("  ▸ {} (id={}){} {}", item.name, id, tag, msg),
                    Style::default().fg(Color::Black).bg(NEON_PURPLE).add_modifier(Modifier::BOLD),
                )))
            } else {
                ListItem::new(Line::from(Span::styled(
                    format!("  ◇ {} (id={}){} {}", item.name, id, tag, msg),
                    Style::default().fg(NEON_PURPLE).bg(DEEP_BG),
                )))
            }
        }).collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(Line::from(vec![
                        Span::styled(" ◈ ", Style::default().fg(NEON_PURPLE)),
                        Span::styled(format!("INTERACTIVE QUERIES ({})", f.interactive_queries.len()), Style::default().fg(NEON_PURPLE).add_modifier(Modifier::BOLD)),
                    ]))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(NEON_PURPLE))
                    .style(Style::default().bg(DEEP_BG)),
            );
        frame.render_widget(list, area);
    }
}

// ── Step 7: Review ──────────────────────────────────────────────────

fn render_review(frame: &mut Frame, area: Rect, state: &AppState) {
    let pretty = serde_json::to_string_pretty(&state.config)
        .unwrap_or_else(|e| format!("(serialize error: {})", e));

    let para = Paragraph::new(pretty)
        .style(Style::default().fg(NEON_GREEN).bg(DEEP_BG))
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(" ✓ ", Style::default().fg(NEON_GREEN)),
                    Span::styled("REVIEW CONFIGURATION", Style::default().fg(NEON_GREEN).add_modifier(Modifier::BOLD)),
                    Span::styled("  (Enter/s to save)", Style::default().fg(DIM_TEXT)),
                ]))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(NEON_GREEN)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(para, area);
}

// ── Footer ──────────────────────────────────────────────────────────

fn render_footer(frame: &mut Frame, area: Rect, state: &AppState, step: FormStep) {
    let keybindings = match step {
        FormStep::PartNumber => "Enter Next | Esc Back",
        FormStep::BasicInfo => "Tab/↑↓ Fields | Enter Next | Esc Back",
        FormStep::OfflineStages | FormStep::DiagnosticsStages
        | FormStep::Cards | FormStep::InteractiveQueries => {
            "↑↓ Select | a Add | e Edit | d Delete | Enter Next | Esc Back"
        }
        FormStep::Review => "Enter/s Save | Esc Back",
    };

    if let Some(ref msg) = state.status_message {
        let style = if msg.starts_with("ERROR") {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD).bg(DEEP_BG)
        } else {
            Style::default().fg(NEON_YELLOW).bg(DEEP_BG)
        };
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(" ⚠ ", Style::default().fg(if msg.starts_with("ERROR") { Color::Red } else { NEON_YELLOW })),
            Span::styled(msg.as_str(), style),
        ]));
        frame.render_widget(footer, area);
    } else {
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(" [", Style::default().fg(NEON_MAGENTA)),
            Span::styled(keybindings, Style::default().fg(DIM_TEXT)),
            Span::styled("]", Style::default().fg(NEON_MAGENTA)),
        ]))
        .style(Style::default().bg(DEEP_BG));
        frame.render_widget(footer, area);
    }
}
