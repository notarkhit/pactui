use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures::StreamExt;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::{
    cache::{self, Backend},
    config::Config,
    notify,
    operations::{self, OperationLine},
    package::{Package, PackageInfo, parse_yay_si_output},
    search::{self, SearchMode},
    theme::{self, Theme},
};

// ─── Enums ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Loading,
    Normal,
    Insert,
    OperationPane(OutputMode),
    HelpOverlay,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputMode {
    Beautified,
    Raw,
}

impl OutputMode {
    pub fn next(&self) -> Self {
        match self {
            OutputMode::Beautified => OutputMode::Raw,
            OutputMode::Raw => OutputMode::Beautified,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueuedOperation {
    Install,
    Remove,
}

#[derive(Debug, Clone)]
pub struct SelectedPackage {
    pub package: Package,
    pub operation: QueuedOperation,
}

// ─── Messages from async tasks ────────────────────────────────────────────────

pub enum AppEvent {
    PackageFetched(Vec<Package>),
    PackageFetchProgress(usize),
    PreviewReady(PackageInfo),
    OperationLine(OperationLine),
    Tick,
}

// ─── App ─────────────────────────────────────────────────────────────────────

pub struct App {
    pub packages: Vec<Package>,     // All packages for current panel
    pub filtered: Vec<Package>,     // After search filter
    pub selected: Vec<SelectedPackage>,
    pub cursor: usize,
    pub panels: Vec<String>,
    pub panel_index: usize,
    pub mode: AppMode,
    pub search_query: String,
    pub search_mode: SearchMode,
    pub theme: Theme,
    pub theme_index: usize,
    pub config: Config,
    pub preview_cache: Option<PackageInfo>,
    pub preview_loading: bool,
    pub operation_output: Vec<String>,
    pub operation_done: bool,
    pub operation_progress: (usize, usize), // (done, total)
    pub yay_version: String,
    pub pacman_version: String,
    pub backend: Backend,
    pub tick_count: usize,
    pub loading_message: String,
    pub offline: bool,
    // Internal
    all_packages: Vec<Package>, // Full unfiltered list across repos
}

impl App {
    pub fn new(config: Config, backend: Backend) -> Self {
        let themes = theme::all_themes();
        let theme_index = theme::theme_by_name(&config.theme);
        let theme = themes[theme_index].clone();
        let yay_ver = cache::get_version(backend.bin());
        let pacman_ver = cache::get_version("pacman");

        Self {
            packages: vec![],
            filtered: vec![],
            selected: vec![],
            cursor: 0,
            panels: vec![],
            panel_index: 0,
            mode: AppMode::Loading,
            search_query: String::new(),
            search_mode: SearchMode::NameFuzzy,
            theme,
            theme_index,
            config,
            preview_cache: None,
            preview_loading: false,
            operation_output: vec![],
            operation_done: false,
            operation_progress: (0, 0),
            yay_version: yay_ver,
            pacman_version: pacman_ver,
            backend,
            tick_count: 0,
            loading_message: "Syncing package database...".to_string(),
            offline: false,
            all_packages: vec![],
        }
    }

    // ── Panel helpers ─────────────────────────────────────────────────────────

    pub fn current_panel_name(&self) -> String {
        self.panels
            .get(self.panel_index)
            .cloned()
            .unwrap_or_else(|| "all".to_string())
    }

    pub fn current_panel_is_selected(&self) -> bool {
        self.current_panel_name() == "selected"
    }

    fn panel_packages(&self) -> Vec<Package> {
        let panel = self.current_panel_name();
        if panel == "selected" {
            self.selected.iter().map(|s| s.package.clone()).collect()
        } else {
            self.all_packages
                .iter()
                .filter(|p| p.repo == panel)
                .cloned()
                .collect()
        }
    }

    fn rebuild_filtered(&mut self) {
        let base = self.panel_packages();
        self.packages = base.clone();
        self.filtered = if self.search_query.is_empty() {
            base
        } else {
            search::filter(&base, &self.search_query, self.search_mode)
        };
        self.cursor = self.cursor.min(self.filtered.len().saturating_sub(1));
    }

    // ── Cursor ────────────────────────────────────────────────────────────────

    pub fn scroll_down(&mut self) {
        if !self.filtered.is_empty() {
            self.cursor = (self.cursor + 1).min(self.filtered.len() - 1);
        }
    }

    pub fn scroll_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    // ── Panel cycling ─────────────────────────────────────────────────────────

    pub fn panel_right(&mut self) {
        if !self.panels.is_empty() {
            self.panel_index = (self.panel_index + 1) % self.panels.len();
        }
        self.cursor = 0;
        self.rebuild_filtered();
        self.preview_cache = None;
    }

    pub fn panel_left(&mut self) {
        if !self.panels.is_empty() {
            self.panel_index = self.panels.len() - 1
                - ((self.panels.len() - 1 - self.panel_index) + 1) % self.panels.len();
        }
        self.cursor = 0;
        self.rebuild_filtered();
        self.preview_cache = None;
    }

    // ── Selection ─────────────────────────────────────────────────────────────

    pub fn toggle_select(&mut self) {
        if let Some(pkg) = self.filtered.get(self.cursor).cloned() {
            if let Some(pos) = self.selected.iter().position(|s| s.package.name == pkg.name) {
                self.selected.remove(pos);
            } else {
                let op = if pkg.installed {
                    QueuedOperation::Remove
                } else {
                    QueuedOperation::Install
                };
                self.selected.push(SelectedPackage { package: pkg, operation: op });
            }
        }
    }

    // ── Search ────────────────────────────────────────────────────────────────

    pub fn push_char(&mut self, c: char) {
        self.search_query.push(c);
        self.cursor = 0;
        self.rebuild_filtered();
    }

    pub fn pop_char(&mut self) {
        self.search_query.pop();
        self.cursor = 0;
        self.rebuild_filtered();
    }

    pub fn delete_word(&mut self) {
        let q = self.search_query.trim_end().to_string();
        let pos = q.rfind(' ').map(|i| i + 1).unwrap_or(0);
        self.search_query = q[..pos].to_string();
        self.cursor = 0;
        self.rebuild_filtered();
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.cursor = 0;
        self.rebuild_filtered();
    }

    pub fn cycle_search_mode(&mut self) {
        self.search_mode = self.search_mode.next();
        self.rebuild_filtered();
    }

    // ── Theme ─────────────────────────────────────────────────────────────────

    pub fn cycle_theme(&mut self) {
        let themes = theme::all_themes();
        self.theme_index = (self.theme_index + 1) % themes.len();
        self.theme = themes[self.theme_index].clone();
        self.config.theme = self.theme.name.to_string();
    }

    // ── Current package ───────────────────────────────────────────────────────

    pub fn current_package(&self) -> Option<&Package> {
        self.filtered.get(self.cursor)
    }

    // ── Operations ────────────────────────────────────────────────────────────

    pub fn install_targets(&self) -> Vec<String> {
        let install_queue: Vec<String> = self
            .selected
            .iter()
            .filter(|s| s.operation == QueuedOperation::Install)
            .map(|s| s.package.name.clone())
            .collect();

        if install_queue.is_empty() {
            // Fall back to current highlighted package
            self.current_package()
                .filter(|p| !p.installed)
                .map(|p| vec![p.name.clone()])
                .unwrap_or_default()
        } else {
            install_queue
        }
    }

    pub fn remove_targets(&self) -> Vec<String> {
        let remove_queue: Vec<String> = self
            .selected
            .iter()
            .filter(|s| s.operation == QueuedOperation::Remove)
            .map(|s| s.package.name.clone())
            .collect();

        if remove_queue.is_empty() {
            self.current_package()
                .filter(|p| p.installed)
                .map(|p| vec![p.name.clone()])
                .unwrap_or_default()
        } else {
            remove_queue
        }
    }
}

// ─── Main run loop ────────────────────────────────────────────────────────────

pub async fn run(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    config: Config,
    backend: Backend,
) -> Result<()> {
    let mut app = App::new(config, backend);

    // Channel for async → app events
    let (tx, mut rx) = mpsc::channel::<AppEvent>(256);

    // ── Fetch packages async ──────────────────────────────────────────────────
    {
        let tx = tx.clone();
        let b = app.backend;
        tokio::spawn(async move {
            let (prog_tx, mut prog_rx) = mpsc::channel::<usize>(64);
            let tx_p = tx.clone();
            tokio::spawn(async move {
                while let Some(n) = prog_rx.recv().await {
                    let _ = tx_p.send(AppEvent::PackageFetchProgress(n)).await;
                }
            });
            match cache::fetch_packages(b, prog_tx).await {
                Ok(pkgs) => {
                    let _ = tx.send(AppEvent::PackageFetched(pkgs)).await;
                }
                Err(_) => {
                    // Try cache fallback
                    let cached = cache::load_cache();
                    let _ = tx.send(AppEvent::PackageFetched(cached)).await;
                }
            }
        });
    }

    // ── Tick timer (for spinner, debounce) ───────────────────────────────────
    {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(80));
            loop {
                ticker.tick().await;
                if tx.send(AppEvent::Tick).await.is_err() {
                    break;
                }
            }
        });
    }

    let mut event_stream = EventStream::new();
    let mut preview_debounce: Option<Instant> = None;
    let mut preview_pkg: Option<String> = None;

    loop {
        // ── Draw ──────────────────────────────────────────────────────────────
        terminal.draw(|f| crate::ui::render(f, &app))?;

        // ── Preview debounce flush ────────────────────────────────────────────
        if let Some(t) = preview_debounce {
            if t.elapsed() >= Duration::from_millis(200) {
                preview_debounce = None;
                if let Some(pkg_name) = preview_pkg.take() {
                    let tx = tx.clone();
                    let bin = app.backend.bin().to_string();
                    tokio::spawn(async move {
                        let output = tokio::process::Command::new(&bin)
                            .args(["-Si", &pkg_name])
                            .output()
                            .await;
                        if let Ok(out) = output {
                            let s = String::from_utf8_lossy(&out.stdout).to_string();
                            let info = parse_yay_si_output(&s);
                            let _ = tx.send(AppEvent::PreviewReady(info)).await;
                        }
                    });
                }
            }
        }

        // ── Event select ──────────────────────────────────────────────────────
        tokio::select! {
            Some(app_ev) = rx.recv() => {
                match app_ev {
                    AppEvent::Tick => {
                        app.tick_count = app.tick_count.wrapping_add(1);
                    }
                    AppEvent::PackageFetchProgress(n) => {
                        app.loading_message = format!("Syncing package database... ({} lines)", n);
                    }
                    AppEvent::PackageFetched(pkgs) => {
                        // Save cache
                        let _ = cache::save_cache(&pkgs);

                        app.all_packages = pkgs;

                        // Build panel list from pacman.conf + aur + selected
                        let mut repos = cache::read_pacman_repos();
                        if !repos.contains(&"aur".to_string()) {
                            repos.push("aur".to_string());
                        }
                        repos.push("selected".to_string());
                        app.panels = repos;

                        app.rebuild_filtered();
                        app.mode = AppMode::Normal;
                    }
                    AppEvent::PreviewReady(info) => {
                        app.preview_cache = Some(info);
                        app.preview_loading = false;
                    }
                    AppEvent::OperationLine(line) => {
                        match line {
                            OperationLine::Stdout(s) | OperationLine::Stderr(s) => {
                                app.operation_output.push(s);
                                // Try to update progress heuristically
                                let done = app.operation_output.iter()
                                    .filter(|l| l.contains("installed") || l.contains("removed"))
                                    .count();
                                app.operation_progress.0 = done;
                            }
                            OperationLine::Done { success } => {
                                app.operation_done = true;
                                if app.config.notify_on_complete {
                                    let summary = if success {
                                        "pactui: operation complete"
                                    } else {
                                        "pactui: operation failed"
                                    };
                                    notify::send(summary, "");
                                }
                                // Refresh packages
                                let tx = tx.clone();
                                let b = app.backend;
                                tokio::spawn(async move {
                                    let (prog_tx, _) = mpsc::channel::<usize>(1);
                                    if let Ok(pkgs) = cache::fetch_packages(b, prog_tx).await {
                                        let _ = tx.send(AppEvent::PackageFetched(pkgs)).await;
                                    }
                                });
                            }
                        }
                    }
                }
            }

            Some(Ok(ev)) = event_stream.next() => {
                match ev {
                    Event::Key(key) => {
                        if handle_key(&mut app, key, tx.clone()).await? {
                            // Quit signal
                            break;
                        }
                        // Trigger preview debounce if cursor moved
                        if matches!(app.mode, AppMode::Normal) {
                            if let Some(pkg) = app.current_package() {
                                let name = pkg.name.clone();
                                if preview_pkg.as_deref() != Some(&name) {
                                    preview_pkg = Some(name);
                                    preview_debounce = Some(Instant::now());
                                    app.preview_loading = true;
                                    app.preview_cache = None;
                                }
                            }
                        }
                    }
                    Event::Resize(_, _) => {} // ratatui handles resize automatically
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

async fn handle_key(
    app: &mut App,
    key: KeyEvent,
    tx: mpsc::Sender<AppEvent>,
) -> Result<bool> {
    // Global quit
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Ok(true);
    }

    match &app.mode.clone() {
        AppMode::Loading => {} // ignore keys while loading

        AppMode::Normal => match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('j') | KeyCode::Down => app.scroll_down(),
            KeyCode::Char('k') | KeyCode::Up => app.scroll_up(),
            KeyCode::Char('h') | KeyCode::Left => app.panel_left(),
            KeyCode::Char('l') | KeyCode::Right => app.panel_right(),
            KeyCode::Char(' ') => app.toggle_select(),
            KeyCode::Char('I') => app.mode = AppMode::Insert,
            KeyCode::Tab => app.cycle_search_mode(),
            KeyCode::Char('T') => app.cycle_theme(),
            KeyCode::Char('?') => app.mode = AppMode::HelpOverlay,
            KeyCode::Char('F') => {
                app.operation_output.clear();
                app.operation_done = false;
                app.operation_progress = (0, 0);
                app.mode = AppMode::OperationPane(OutputMode::Beautified);
            }
            KeyCode::Char('A') => {
                let targets = app.install_targets();
                if !targets.is_empty() {
                    let total = targets.len();
                    app.operation_output.clear();
                    app.operation_done = false;
                    app.operation_progress = (0, total);
                    app.mode = AppMode::OperationPane(OutputMode::Beautified);
                    let tx = tx.clone();
                    let b = app.backend;
                    tokio::spawn(async move {
                        match operations::install(b, &targets).await {
                            Ok(mut rx) => {
                                while let Some(line) = rx.recv().await {
                                    let _ = tx.send(AppEvent::OperationLine(line)).await;
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(AppEvent::OperationLine(OperationLine::Stderr(
                                    format!("Error: {}", e),
                                ))).await;
                                let _ = tx.send(AppEvent::OperationLine(OperationLine::Done { success: false })).await;
                            }
                        }
                    });
                }
            }
            KeyCode::Char('R') => {
                let targets = app.remove_targets();
                if !targets.is_empty() {
                    let total = targets.len();
                    app.operation_output.clear();
                    app.operation_done = false;
                    app.operation_progress = (0, total);
                    app.mode = AppMode::OperationPane(OutputMode::Beautified);
                    let tx = tx.clone();
                    let b = app.backend;
                    tokio::spawn(async move {
                        match operations::remove(b, &targets).await {
                            Ok(mut rx) => {
                                while let Some(line) = rx.recv().await {
                                    let _ = tx.send(AppEvent::OperationLine(line)).await;
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(AppEvent::OperationLine(OperationLine::Stderr(
                                    format!("Error: {}", e),
                                ))).await;
                                let _ = tx.send(AppEvent::OperationLine(OperationLine::Done { success: false })).await;
                            }
                        }
                    });
                }
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.mode = AppMode::Loading;
                app.loading_message = "Refreshing package database...".to_string();
                let tx = tx.clone();
                let b = app.backend;
                tokio::spawn(async move {
                    let (prog_tx, _) = mpsc::channel::<usize>(1);
                    match cache::fetch_packages(b, prog_tx).await {
                        Ok(pkgs) => { let _ = tx.send(AppEvent::PackageFetched(pkgs)).await; }
                        Err(_) => {}
                    }
                });
            }
            _ => {}
        },

        AppMode::Insert => match key.code {
            KeyCode::Esc => app.mode = AppMode::Normal,
            KeyCode::Backspace => app.pop_char(),
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => app.delete_word(),
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => app.clear_search(),
            KeyCode::Char(c) => app.push_char(c),
            _ => {}
        },

        AppMode::OperationPane(current_output_mode) => {
            let current = current_output_mode.clone();
            match key.code {
                KeyCode::Char('F') => {
                    app.mode = AppMode::OperationPane(current.next());
                }
                KeyCode::Char('q') | KeyCode::Esc if app.operation_done => {
                    app.mode = AppMode::Normal;
                }
                _ => {}
            }
        }

        AppMode::HelpOverlay => match key.code {
            KeyCode::Char('?') | KeyCode::Esc => app.mode = AppMode::Normal,
            _ => {}
        },
    }

    Ok(false)
}
