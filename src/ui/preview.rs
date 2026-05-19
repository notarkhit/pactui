use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::{app::App, package::PackageInfo, theme::Theme};

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    let block = Block::default()
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled("PACKAGE INFO", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));

    if app.preview_loading {
        let spinner = SPINNER_FRAMES[app.tick_count % SPINNER_FRAMES.len()];
        let text = Paragraph::new(Line::from(vec![
            Span::styled(spinner, Style::default().fg(theme.accent)),
            Span::styled(" Fetching package info...", Style::default().fg(theme.dim)),
        ]))
        .block(block);
        f.render_widget(text, area);
        return;
    }

    if let Some(ref info) = app.preview_cache {
        let lines = build_info_lines(info, theme);
        let para = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(para, area);
    } else {
        let text = Paragraph::new(Span::styled(
            "  Navigate to a package to see details",
            Style::default().fg(theme.dim),
        ))
        .block(block);
        f.render_widget(text, area);
    }
}

fn build_info_lines<'a>(info: &'a PackageInfo, theme: &'a Theme) -> Vec<Line<'a>> {
    let key_style = Style::default().fg(theme.blue).add_modifier(Modifier::BOLD);
    let val_style = Style::default().fg(theme.fg);
    let sep_style = Style::default().fg(theme.dim);

    let divider = Line::from(Span::styled(
        "─".repeat(40),
        sep_style,
    ));

    let field = |k: &'a str, v: String| -> Line<'a> {
        Line::from(vec![
            Span::styled(format!("  {:<14}: ", k), key_style),
            Span::styled(v, val_style),
        ])
    };

    let installed_label = if info.installed {
        format!("Yes ({})", info.version)
    } else {
        "No".to_string()
    };

    let mut lines = vec![
        Line::from(""),
        field("Name", info.name.clone()),
        field("Version", info.version.clone()),
        field("Repository", info.repo.clone()),
        divider.clone(),
        field("Description", info.description.clone()),
        field("URL", info.url.clone()),
        field("Licenses", info.licenses.join("  ")),
        divider.clone(),
        field("Depends On", info.depends.join("  ")),
        field("Optional", info.optional_deps.first().cloned().unwrap_or_default()),
        field("Conflicts", info.conflicts.join("  ")),
        field("Provides", info.provides.join("  ")),
        divider.clone(),
        field("Installed", installed_label),
    ];

    if let Some(ref d) = info.install_date {
        lines.push(field("Install Date", d.clone()));
    }
    if let Some(ref d) = info.build_date {
        lines.push(field("Build Date", d.clone()));
    }
    if let Some(ref s) = info.installed_size {
        lines.push(field("Size", s.clone()));
    }
    if let Some(ref p) = info.packager {
        lines.push(field("Packager", p.clone()));
    }

    lines
}
