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
            let is_queued = queued_op.is_some();

            // Determine queued operation for this package
            let queue_install = queued_op
                .map(|s| s.operation == QueuedOperation::Install)
                .unwrap_or(false);
            let queue_remove = queued_op
                .map(|s| s.operation == QueuedOperation::Remove)
                .unwrap_or(false);

            // Repo badge (dim, left-padded)
            let repo_span = Span::styled(
                format!("{:<10}", pkg.repo),
                Style::default().fg(theme.dim),
            );

            // Name: colour priority — queued remove (red) > installed (green) >
            //        queued install (accent) > normal (fg)
            let name_color = if queue_remove {
                theme.red
            } else if pkg.installed {
                theme.green
            } else if queue_install {
                theme.accent
            } else {
                theme.fg
            };
            let name_modifier = if is_queued {
                Modifier::BOLD
            } else {
                Modifier::empty()
            };
            let name_span = Span::styled(
                format!("{:<30}", pkg.name),
                Style::default().fg(name_color).add_modifier(name_modifier),
            );

            // Version
            let ver_span = Span::styled(
                format!("{:<16}", pkg.version),
                Style::default().fg(theme.dim),
            );

            // Indicator glyph on the right
            let glyph_span = if queue_remove {
                Span::styled(
                    " [-]",
                    Style::default()
                        .fg(theme.red)
                        .add_modifier(Modifier::BOLD),
                )
            } else if queue_install {
                Span::styled(
                    " [+]",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )
            } else if pkg.installed {
                Span::styled(" ✓", Style::default().fg(theme.green))
            } else {
                Span::raw("  ")
            };

            let line = Line::from(vec![repo_span, name_span, ver_span, glyph_span]);

            // Row background: cursor > queued > default
            let row_style = if is_cursor && is_queued {
                // Cursor on a queued item — brightest highlight
                Style::default()
                    .bg(theme.selected_bg)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else if is_cursor {
                Style::default()
                    .bg(theme.selected_bg)
                    .add_modifier(Modifier::BOLD)
            } else if is_queued {
                // Queued but not cursor — accent background to stand out
                Style::default().bg(theme.selected_bg).fg(theme.fg)
            } else {
                Style::default()
            };

            ListItem::new(line).style(row_style)
        })
        .collect();

    let panel_label = app.current_panel_name().to_uppercase();
    let block = Block::default()
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled(
                panel_label,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));

    let mut state = ListState::default();
    state.select(Some(app.cursor));

    f.render_stateful_widget(
        List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(theme.selected_bg)
                    .add_modifier(Modifier::BOLD),
            ),
        area,
        &mut state,
    );
}
