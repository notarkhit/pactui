mod app;
mod cache;
mod config;
mod lock;
mod notify;
mod operations;
mod package;
mod search;
mod theme;
mod ui;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::panic;

#[tokio::main]
async fn main() -> Result<()> {
    // ── Config ────────────────────────────────────────────────────────────────
    let cfg = config::load().unwrap_or_default();

    // ── Single instance lock ──────────────────────────────────────────────────
    if lock::check_existing() {
        eprintln!("pactui: another instance is already running.");
        eprintln!("  Lock file: {}", lock::lock_path().display());
        eprintln!("  Use Ctrl+C to force quit the other instance first.");
        std::process::exit(1);
    }
    lock::acquire()?;

    // ── Backend detection ─────────────────────────────────────────────────────
    let backend = cache::detect_backend();

    // ── Terminal setup ────────────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend_term = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend_term)?;

    // ── Panic hook — always restore terminal ──────────────────────────────────
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    // ── Run ───────────────────────────────────────────────────────────────────
    let result = app::run(&mut terminal, cfg.clone(), backend).await;

    // ── Teardown ──────────────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Save config (persists current theme)
    let _ = config::save(&cfg);

    lock::release();

    result
}
