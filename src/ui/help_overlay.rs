use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

const HELP_TEXT: &[(&str, &str)] = &[
    ("NORMAL MODE", ""),
    ("I", "Enter insert (search) mode"),
    ("j / k", "Scroll list down / up"),
    ("h / l", "Cycle panel left / right"),
    ("Space", "Select / deselect package"),
    ("A", "Install selected packages"),
    ("R", "Remove selected packages"),
    ("F", "Toggle operation pane"),
    ("T", "Cycle theme"),
    ("Tab", "Cycle search mode"),
    ("Ctrl+R", "Refresh package list"),
    ("?", "Toggle this help"),
    ("q / Ctrl+C", "Quit"),
    ("", ""),
    ("INSERT MODE", ""),
    ("Escape", "Return to normal mode"),
    ("Ctrl+W", "Delete last word"),
    ("Ctrl+U", "Clear search"),
    ("Backspace", "Delete last character"),
    ("", ""),
    ("OPERATION PANE", ""),
    ("F", "Cycle output mode (beautified → raw → hide)"),
    ("q / Escape", "Hide pane (if operation complete)"),
];

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    // Floating centered window: 60% wide, 70% tall
    let popup_area = centered_rect(60, 70, area);

    // Clear behind popup
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled("KEYBINDINGS", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw("  press ? or Escape to close "),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let lines: Vec<Line> = HELP_TEXT
        .iter()
        .map(|(key, action)| {
            if action.is_empty() {
                // Section header or blank
                if key.is_empty() {
                    Line::from("")
                } else {
                    Line::from(Span::styled(
                        format!("  {} ", key),
                        Style::default().fg(theme.purple).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                    ))
                }
            } else {
                Line::from(vec![
                    Span::styled(
                        format!("  {:12}", key),
                        Style::default().fg(theme.yellow).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("  ", Style::default().fg(theme.dim)),
                    Span::styled(action.to_string(), Style::default().fg(theme.fg)),
                ])
            }
        })
        .collect();

    let para = Paragraph::new(lines)
        .wrap(Wrap { trim: false });
    f.render_widget(para, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(v[1])[1]
}
