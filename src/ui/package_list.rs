use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
    Frame,
};

use crate::{
    app::{App, QueuedOperation},
    package::Package,
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    let packages: Vec<&Package> = if app.current_panel_is_selected() {
        // Selected panel — show selected queue
        // We'll borrow the selected list as packages for display
        app.selected.iter().map(|s| &s.package).collect()
    } else {
        app.filtered.iter().collect()
    };

    let items: Vec<ListItem> = packages
        .iter()
        .enumerate()
        .map(|(i, pkg)| {
            let is_cursor = i == app.cursor;
            let queued_op = app.selected.iter().find(|s| s.package.name == pkg.name);
            let is_selected = queued_op.is_some();

            let repo_span = Span::styled(
                format!("{:<10}", pkg.repo),
                Style::default().fg(theme.dim),
            );
            let name_span = Span::styled(
                format!("{:<30}", pkg.name),
                if pkg.installed {
                    Style::default().fg(theme.green)
                } else if is_selected {
                    Style::default().fg(theme.accent)
                } else {
                    Style::default().fg(theme.fg)
                },
            );
            let ver_span = Span::styled(
                format!("{:<16}", pkg.version),
                Style::default().fg(theme.dim),
            );

            let check_span = if pkg.installed {
                Span::styled(" ✓", Style::default().fg(theme.green))
            } else if let Some(sel) = queued_op {
                match sel.operation {
                    QueuedOperation::Install => Span::styled(" +", Style::default().fg(theme.accent)),
                    QueuedOperation::Remove => Span::styled(" -", Style::default().fg(theme.red)),
                }
            } else {
                Span::raw("  ")
            };

            let line = Line::from(vec![repo_span, name_span, ver_span, check_span]);
            let style = if is_cursor {
                Style::default().bg(theme.selected_bg).add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().bg(theme.selected_bg)
            } else {
                Style::default()
            };
            ListItem::new(line).style(style)
        })
        .collect();

    let panel_label = app.current_panel_name();
    let block = Block::default()
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled(
                panel_label,
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));

    let mut state = ListState::default();
    state.select(Some(app.cursor));

    f.render_stateful_widget(List::new(items).block(block).highlight_style(
        Style::default().bg(theme.selected_bg).add_modifier(Modifier::BOLD),
    ), area, &mut state);
}
