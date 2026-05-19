pub mod help_overlay;
pub mod layout;
pub mod loading;
pub mod operation_pane;
pub mod package_list;
pub mod preview;
pub mod search_bar;
pub mod size_warning;
pub mod status_bar;

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, AppMode};

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    // Minimum size guard
    if area.width < app.config.min_width || area.height < app.config.min_height {
        size_warning::render(f, area, app);
        return;
    }

    // Full-screen loading
    if matches!(app.mode, AppMode::Loading) {
        loading::render(f, area, app);
        return;
    }

    let lay = layout::compute(area);

    // Header bar
    render_header(f, lay.header, app);

    // Operation pane replaces list+preview when active
    if matches!(app.mode, AppMode::OperationPane(_)) {
        operation_pane::render(f, lay.main, app);
    } else {
        search_bar::render(f, lay.search, app);
        package_list::render(f, lay.list, app);
        preview::render(f, lay.preview, app);
    }

    // Status bar always visible
    status_bar::render(f, lay.status, app);

    // Help overlay (on top of everything)
    if matches!(app.mode, AppMode::HelpOverlay) {
        help_overlay::render(f, area, app);
    }
}

fn render_header(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let theme = &app.theme;

    let panels_str = app
        .panels
        .iter()
        .map(|p| {
            if *p == app.current_panel_name() {
                format!("[{}]", p)
            } else {
                p.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("  ");

    let line = Line::from(vec![
        Span::styled(
            " pactui v0.1.0  ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(panels_str, Style::default().fg(theme.dim)),
    ]);

    let para = Paragraph::new(line).style(Style::default().bg(theme.header_bg));
    f.render_widget(para, area);
}
