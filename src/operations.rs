use anyhow::Result;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use crate::cache::Backend;

#[derive(Debug, Clone)]
pub enum OperationLine {
    Stdout(String),
    Stderr(String),
    Done { success: bool },
}

/// Always use `pkexec <bin> [args]` — polkit pops a graphical auth dialog,
/// stdin stays null so the TUI never blocks waiting for a password prompt.
fn build_privileged_command(backend: Backend, args: &[String]) -> Command {
    let mut cmd = Command::new("pkexec");
    cmd.arg(backend.bin());
    cmd.args(args);
    cmd
}

/// Spawn install operation, streaming lines over the returned receiver.
pub async fn install(
    backend: Backend,
    pkgs: &[String],
) -> Result<mpsc::Receiver<OperationLine>> {
    let mut args: Vec<String> = vec!["--needed".into(), "-S".into()];
    args.extend(pkgs.iter().cloned());
    args.push("--noconfirm".into());
    run_operation(backend, args).await
}

/// Spawn remove operation.
pub async fn remove(
    backend: Backend,
    pkgs: &[String],
) -> Result<mpsc::Receiver<OperationLine>> {
    let mut args: Vec<String> = vec!["-R".into()];
    args.extend(pkgs.iter().cloned());
    args.push("--noconfirm".into());
    run_operation(backend, args).await
}

async fn run_operation(
    backend: Backend,
    args: Vec<String>,
) -> Result<mpsc::Receiver<OperationLine>> {
    let mut child = build_privileged_command(backend, &args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null())
        .spawn()?;

    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");

    let (tx, rx) = mpsc::channel::<OperationLine>(1024);

    tokio::spawn(async move {
        let tx_clone = tx.clone();
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

        while let Ok(Some(l)) = out_reader.next_line().await {
            let _ = tx.send(OperationLine::Stdout(l)).await;
        }
        while let Ok(Some(l)) = err_reader.next_line().await {
            let _ = tx.send(OperationLine::Stderr(l)).await;
        }

        let success = child.wait().await.map(|s| s.success()).unwrap_or(false);
        let _ = tx_clone.send(OperationLine::Done { success }).await;
    });

    Ok(rx)
}
