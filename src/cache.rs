use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::package::{parse_yay_sl_line, Package};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Yay,
    Pacman,
}

impl Backend {
    pub fn bin(&self) -> &'static str {
        match self {
            Backend::Yay => "yay",
            Backend::Pacman => "pacman",
        }
    }
}

pub fn detect_backend() -> Backend {
    if which("yay") {
        Backend::Yay
    } else {
        Backend::Pacman
    }
}

fn which(bin: &str) -> bool {
    std::process::Command::new("which")
        .arg(bin)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("~/.cache"))
        .join("pactui")
        .join("packages.db")
}

pub fn load_cache() -> Vec<Package> {
    let path = cache_path();
    if !path.exists() {
        return vec![];
    }
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save_cache(pkgs: &[Package]) -> Result<()> {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string(pkgs)?;
    std::fs::write(&path, raw)?;
    Ok(())
}

/// Fetch the full package list by running `<backend> -Sl` and parsing each line.
/// Returns a progress callback with `(lines_read)`.
pub async fn fetch_packages(
    backend: Backend,
    progress_tx: tokio::sync::mpsc::Sender<usize>,
) -> Result<Vec<Package>> {
    let mut child = Command::new(backend.bin())
        .arg("-Sl")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .with_context(|| format!("spawning {} -Sl", backend.bin()))?;

    let stdout = child.stdout.take().expect("stdout piped");
    let mut reader = BufReader::new(stdout).lines();
    let mut packages = Vec::with_capacity(60_000);
    let mut count = 0usize;

    while let Some(line) = reader.next_line().await? {
        if let Some(pkg) = parse_yay_sl_line(&line) {
            packages.push(pkg);
        }
        count += 1;
        if count % 500 == 0 {
            let _ = progress_tx.try_send(count);
        }
    }

    child.wait().await?;
    let _ = progress_tx.send(count).await;
    Ok(packages)
}

/// Get the version string of a binary (e.g. `yay 12.3.5`).
pub fn get_version(bin: &str) -> String {
    std::process::Command::new(bin)
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8(o.stdout)
                .ok()
                .or_else(|| String::from_utf8(o.stderr).ok())
        })
        .and_then(|s| s.lines().next().map(|l| l.trim().to_string()))
        .unwrap_or_else(|| format!("{} (unknown)", bin))
}

/// Read enabled repos from /etc/pacman.conf (lines starting with `[`, not `[options]`).
pub fn read_pacman_repos() -> Vec<String> {
    let content = std::fs::read_to_string("/etc/pacman.conf").unwrap_or_default();
    let mut repos = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            let name = &line[1..line.len() - 1];
            if name.to_lowercase() != "options" {
                repos.push(name.to_string());
            }
        }
    }
    repos
}
