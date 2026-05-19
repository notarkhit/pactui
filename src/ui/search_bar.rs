use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::{app::App, search::SearchMode};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    let icon = match app.search_mode {
        SearchMode::NameFuzzy => "󰊄 ",
        SearchMode::NameExact => "  ",
    };

    let display = if app.search_query.is_empty() {
        format!("{} search... ({})", icon, app.search_mode.label())
    } else {
        format!("{}{}", icon, app.search_query)
    };

    let is_insert = matches!(app.mode, crate::app::AppMode::Insert);

    let text_style = if app.search_query.is_empty() {
        Style::default().fg(theme.dim)
    } else {
        Style::default().fg(theme.fg)
    };

    let cursor_str = if is_insert { "█" } else { "" };

    let para = Paragraph::new(Line::from(vec![
        Span::styled(display, text_style),
        Span::styled(cursor_str, Style::default().fg(theme.accent).add_modifier(Modifier::RAPID_BLINK)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(if is_insert { theme.accent } else { theme.border }))
            .style(Style::default().bg(theme.search_bg)),
    );

    f.render_widget(para, area);
}
