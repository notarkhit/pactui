use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, AppMode};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    let mode_label = match &app.mode {
        AppMode::Normal => "NORMAL",
        AppMode::Insert => "INSERT",
        AppMode::OperationPane(_) => "OPERATION",
        AppMode::HelpOverlay => "HELP",
        AppMode::Loading => "LOADING",
    };

    let mode_color = match &app.mode {
        AppMode::Normal => theme.blue,
        AppMode::Insert => theme.green,
        AppMode::OperationPane(_) => theme.yellow,
        AppMode::HelpOverlay => theme.purple,
        AppMode::Loading => theme.dim,
    };

    let panel = app.current_panel_name();
    let total = app.filtered.len();
    let selected_count = app.selected.len();
    let yay_ver = &app.yay_version;
    let pacman_ver = &app.pacman_version;

    let offline_label = if app.offline {
        Span::styled(" [OFFLINE]", Style::default().fg(theme.red).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("")
    };

    let line = Line::from(vec![
        Span::styled(
            format!(" [{}]", mode_label),
            Style::default().fg(mode_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" [{}]", app.search_mode.label()),
            Style::default().fg(theme.purple),
        ),
        Span::styled(
            format!(" [{}]", panel),
            Style::default().fg(theme.accent),
        ),
        Span::styled(
            format!("  {} pkgs", total),
            Style::default().fg(theme.fg),
        ),
        Span::styled(
            format!(" | {} selected", selected_count),
            Style::default().fg(if selected_count > 0 { theme.yellow } else { theme.dim }),
        ),
        offline_label,
        Span::styled(
            format!("  {} / pacman {}", yay_ver, pacman_ver),
            Style::default().fg(theme.dim),
        ),
        Span::raw(" "),
    ]);

    let para = Paragraph::new(line)
        .style(Style::default().bg(theme.status_bg).fg(theme.fg));
    f.render_widget(para, area);
}
