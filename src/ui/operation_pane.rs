use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, AppMode, OutputMode};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    let output_mode = match &app.mode {
        AppMode::OperationPane(m) => m.clone(),
        _ => OutputMode::Beautified,
    };

    // Split: title(3) | output(fill) | progress(3) | hint(1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    // Header
    let op_label = if app.operation_done {
        Span::styled("Operation Complete", Style::default().fg(theme.green).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("Operation in Progress...", Style::default().fg(theme.yellow).add_modifier(Modifier::BOLD))
    };
    let mode_label = match output_mode {
        OutputMode::Beautified => Span::styled(" [beautified]", Style::default().fg(theme.accent)),
        OutputMode::Raw => Span::styled(" [raw]", Style::default().fg(theme.dim)),
    };

    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));

    let header_para = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        op_label,
        mode_label,
        Span::styled("  (F to cycle view)", Style::default().fg(theme.dim)),
    ]))
    .block(header_block);
    f.render_widget(header_para, chunks[0]);

    // Output area
    let output_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));

    match output_mode {
        OutputMode::Beautified => {
            // Parse lines for status markers
            let items: Vec<ListItem> = app
                .operation_output
                .iter()
                .map(|line| {
                    let style = if line.starts_with("✓") || line.contains("installed") || line.contains("removed") {
                        Style::default().fg(theme.green)
                    } else if line.starts_with("✗") || line.contains("error") || line.contains("failed") {
                        Style::default().fg(theme.red)
                    } else if line.starts_with("⟳") || line.contains("installing") || line.contains("removing") {
                        Style::default().fg(theme.yellow)
                    } else {
                        Style::default().fg(theme.fg)
                    };
                    ListItem::new(Line::from(Span::styled(format!("  {}", line), style)))
                })
                .collect();

            // Show last N lines that fit
            let list = List::new(items).block(output_block);
            f.render_widget(list, chunks[1]);
        }
        OutputMode::Raw => {
            // Raw: just scroll to bottom, show last N lines
            let inner_height = chunks[1].height.saturating_sub(2) as usize;
            let start = app.operation_output.len().saturating_sub(inner_height);
            let visible: Vec<Line> = app.operation_output[start..]
                .iter()
                .map(|l| Line::from(Span::styled(format!("  {}", l), Style::default().fg(theme.fg))))
                .collect();
            let para = Paragraph::new(visible).block(output_block).wrap(Wrap { trim: true });
            f.render_widget(para, chunks[1]);
        }
    }

    // Progress bar
    let progress_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));

    let (done, total) = app.operation_progress;
    let ratio = if total > 0 { done as f64 / total as f64 } else { 0.0 };
    let label = format!("{}/{}", done, total);
    let gauge = Gauge::default()
        .block(progress_block)
        .gauge_style(Style::default().fg(theme.accent).bg(theme.selected_bg))
        .ratio(ratio)
        .label(label);
    f.render_widget(gauge, chunks[2]);

    // Hint
    let hint = if app.operation_done {
        "  q / Escape to close"
    } else {
        "  operation running..."
    };
    let hint_para = Paragraph::new(Span::styled(hint, Style::default().fg(theme.dim)))
        .style(Style::default().bg(theme.status_bg));
    f.render_widget(hint_para, chunks[3]);
}
