use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub header: Rect,
    pub main: Rect,   // split into list + preview, or full for operation pane
    pub list: Rect,
    pub preview: Rect,
    pub search: Rect, // inside list area at top
    pub status: Rect,
}

pub fn compute(area: Rect) -> AppLayout {
    // Vertical split: header(1) | main(fill) | status(1)
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(0),    // main
            Constraint::Length(1), // status bar
        ])
        .split(area);

    let header = outer[0];
    let main_area = outer[1];
    let status = outer[2];

    // Horizontal split: list(40%) | preview(60%)
    let h_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main_area);

    let list_area = h_split[0];
    let preview = h_split[1];

    // Search bar inside list area — top 3 rows
    let list_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // search bar
            Constraint::Min(0),    // package list
        ])
        .split(list_area);

    AppLayout {
        header,
        main: main_area,
        list: list_split[1],
        preview,
        search: list_split[0],
        status,
    }
}
