use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::app::App;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    // Full-screen dark background
    let bg = Block::default().style(Style::default().bg(theme.bg));
    f.render_widget(bg, area);

    // Center vertically
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Length(7),
            Constraint::Percentage(65),
        ])
        .split(area);

    let spinner = SPINNER_FRAMES[app.tick_count % SPINNER_FRAMES.len()];

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "p a c t u i",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(vec![
            Span::styled(spinner, Style::default().fg(theme.accent)),
            Span::styled(
                format!("  {}", app.loading_message),
                Style::default().fg(theme.fg),
            ),
        ])
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {} packages indexed", app.packages.len()),
            Style::default().fg(theme.dim),
        ))
        .alignment(Alignment::Center),
    ];

    let para = Paragraph::new(lines);
    f.render_widget(para, v[1]);
}
