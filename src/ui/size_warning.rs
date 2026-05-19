use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(7),
            Constraint::Percentage(60),
        ])
        .split(area);

    let h = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(v[1]);

    let popup = h[1];
    f.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.yellow))
        .style(Style::default().bg(theme.bg));

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "⚠  Terminal Too Small",
            Style::default()
                .fg(theme.yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(Span::styled(
            format!(
                "  Minimum size: {}×{}  (current: {}×{})",
                app.config.min_width,
                app.config.min_height,
                area.width,
                area.height
            ),
            Style::default().fg(theme.fg),
        ))
        .alignment(Alignment::Center),
        Line::from(""),
    ];

    let para = Paragraph::new(lines).block(block);
    f.render_widget(para, popup);
}
