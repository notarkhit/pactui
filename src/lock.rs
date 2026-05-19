use std::path::PathBuf;

pub fn lock_path() -> PathBuf {
    // Prefer XDG_RUNTIME_DIR, fall back to /tmp
    std::env::var("XDG_RUNTIME_DIR")
        .map(|d| PathBuf::from(d).join("pactui.lock"))
        .unwrap_or_else(|_| PathBuf::from("/tmp/pactui.lock"))
}

/// Returns true if another instance is already running.
pub fn check_existing() -> bool {
    let path = lock_path();
    if !path.exists() {
        return false;
    }
    // Read the stored PID and check if it's alive
    if let Ok(pid_str) = std::fs::read_to_string(&path) {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            // Send signal 0 — just checks existence
            return unsafe { libc::kill(pid as libc::pid_t, 0) } == 0;
        }
    }
    false
}

/// Write our PID to the lock file.
pub fn acquire() -> std::io::Result<()> {
    let path = lock_path();
    let pid = std::process::id();
    std::fs::write(&path, pid.to_string())
}

/// Remove the lock file on exit.
pub fn release() {
    let _ = std::fs::remove_file(lock_path());
}
