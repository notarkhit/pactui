use anyhow::Result;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum OperationLine {
    Stdout(String),
    Stderr(String),
    Done { success: bool },
}

/// Install packages:
/// - `pacman_pkgs` → `pkexec pacman --needed -S ... --noconfirm`  (official repos)
/// - `aur_pkgs`   → `yay --needed -S ... --noconfirm`             (AUR, no pkexec — yay refuses root)
///
/// Both streams are merged into one receiver. A single `Done` is sent after
/// all commands finish (or immediately if both lists are empty).
pub async fn install(
    pacman_pkgs: &[String],
    aur_pkgs: &[String],
) -> Result<mpsc::Receiver<OperationLine>> {
    let mut base_args: Vec<String> = vec!["--needed".into(), "-S".into()];
    base_args.push("--noconfirm".into());
    run_split(pacman_pkgs, aur_pkgs, &base_args, &base_args).await
}

/// Remove packages:
/// - `pacman_pkgs` → `pkexec pacman -R ... --noconfirm`
/// - `aur_pkgs`   → `yay -R ... --noconfirm`
pub async fn remove(
    pacman_pkgs: &[String],
    aur_pkgs: &[String],
) -> Result<mpsc::Receiver<OperationLine>> {
    let pacman_args: Vec<String> = vec!["-R".into(), "--noconfirm".into()];
    let yay_args: Vec<String> = vec!["-R".into(), "--noconfirm".into()];
    run_split(pacman_pkgs, aur_pkgs, &pacman_args, &yay_args).await
}

/// Spawn one or two processes depending on which lists are non-empty,
/// merge their stdout/stderr into a single channel, send Done at the end.
async fn run_split(
    pacman_pkgs: &[String],
    aur_pkgs: &[String],
    pacman_extra_args: &[String],
    yay_extra_args: &[String],
) -> Result<mpsc::Receiver<OperationLine>> {
    let (tx, rx) = mpsc::channel::<OperationLine>(1024);

    // Capture owned vecs so they can move into the async task
    let pacman_pkgs = pacman_pkgs.to_vec();
    let aur_pkgs = aur_pkgs.to_vec();
    let pacman_extra = pacman_extra_args.to_vec();
    let yay_extra = yay_extra_args.to_vec();

    tokio::spawn(async move {
        let mut overall_success = true;

        // ── Official repo packages via pkexec pacman ──────────────────────────
        if !pacman_pkgs.is_empty() {
            let mut args = pacman_extra.clone();
            args.extend(pacman_pkgs.iter().cloned());

            match spawn_streamed("pkexec", &["pacman"], &args, tx.clone()).await {
                Ok(success) => overall_success &= success,
                Err(e) => {
                    let _ = tx
                        .send(OperationLine::Stderr(format!("[pacman] spawn error: {}", e)))
                        .await;
                    overall_success = false;
                }
            }
        }

        // ── AUR packages via yay (no pkexec — yay handles its own escalation) ─
        if !aur_pkgs.is_empty() {
            let mut args = yay_extra.clone();
            args.extend(aur_pkgs.iter().cloned());

            match spawn_streamed("yay", &[], &args, tx.clone()).await {
                Ok(success) => overall_success &= success,
                Err(e) => {
                    let _ = tx
                        .send(OperationLine::Stderr(format!("[yay] spawn error: {}", e)))
                        .await;
                    overall_success = false;
                }
            }
        }

        let _ = tx.send(OperationLine::Done { success: overall_success }).await;
    });

    Ok(rx)
}

/// Spawn `bin [prefix_args] [args]` with piped stdout/stderr and null stdin.
/// Streams each line to `tx`. Returns the process exit success flag.
async fn spawn_streamed(
    bin: &str,
    prefix_args: &[&str],
    args: &[String],
    tx: mpsc::Sender<OperationLine>,
) -> Result<bool> {
    let mut child = Command::new(bin)
        .args(prefix_args)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null())
        .spawn()?;

    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");

    let mut out_reader = BufReader::new(stdout).lines();
    let mut err_reader = BufReader::new(stderr).lines();

    loop {
        tokio::select! {
            line = out_reader.next_line() => {
                match line {
                    Ok(Some(l)) => { let _ = tx.send(OperationLine::Stdout(l)).await; }
                    _ => break,
                }
            }
            line = err_reader.next_line() => {
                match line {
                    Ok(Some(l)) => { let _ = tx.send(OperationLine::Stderr(l)).await; }
                    _ => break,
                }
            }
        }
    }

    // Drain remaining
    while let Ok(Some(l)) = out_reader.next_line().await {
        let _ = tx.send(OperationLine::Stdout(l)).await;
    }
    while let Ok(Some(l)) = err_reader.next_line().await {
        let _ = tx.send(OperationLine::Stderr(l)).await;
    }

    let success = child.wait().await.map(|s| s.success()).unwrap_or(false);
    Ok(success)
}
