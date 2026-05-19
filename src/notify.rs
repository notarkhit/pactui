use notify_rust::Notification;

pub fn send(summary: &str, body: &str) {
    let _ = Notification::new()
        .summary(summary)
        .body(body)
        .icon("system-software-install")
        .show();
}
