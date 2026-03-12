pub fn send_notification(summary: &str, body: &str) {
    // Use notify-send directly — notify-rust's zbus backend can silently
    // fail to display on some desktops (e.g. Cinnamon).
    match std::process::Command::new("notify-send")
        .arg("--app-name=Rust Clock")
        .arg("-t")
        .arg("10000")
        .arg(summary)
        .arg(body)
        .spawn()
    {
        Ok(mut child) => {
            std::thread::spawn(move || match child.wait() {
                Ok(status) if !status.success() => {
                    eprintln!("notify-send exited with status: {status}");
                }
                Ok(_) => {}
                Err(error) => eprintln!("Failed to wait for notify-send: {error}"),
            });
        }
        Err(error) => eprintln!("Failed to send notification: {error}"),
    }
}
