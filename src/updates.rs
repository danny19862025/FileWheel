use gtk::prelude::*;
const REPOSITORY: &str = "https://github.com/danny19862025/FileWheel";

fn version(value: &str) -> Result<[u64; 3], String> {
    let parts = value
        .trim_start_matches('v')
        .split('.')
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Invalid release version")?;
    parts
        .try_into()
        .map_err(|_| "Invalid release version".into())
}

fn release_summary(json: &str, current: &str) -> Result<String, String> {
    let release: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    if release["draft"] == true || release["prerelease"] == true {
        return Err("No stable release available".into());
    }
    let tag = release["tag_name"]
        .as_str()
        .ok_or("Release version unavailable")?;
    Ok(if version(tag)? > version(current)? {
        format!("{tag} is available. Open Releases to download it. Installed: {current}.")
    } else {
        format!("You're up to date. Installed: {current}; latest stable: {tag}.")
    })
}

pub fn add_controls(body: &gtk::Box) {
    body.append(&gtk::LinkButton::with_label(
        REPOSITORY,
        "FileWheel on GitHub ↗",
    ));
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let check = gtk::Button::with_label("Check for updates");
    check.set_widget_name("check-app-updates");
    row.append(&check);
    row.append(&gtk::LinkButton::with_label(
        &format!("{REPOSITORY}/releases/latest"),
        "Releases ↗",
    ));
    body.append(&row);
    let status = gtk::Label::new(Some("Check for new FileWheel releases on GitHub."));
    status.set_wrap(true);
    status.set_xalign(0.0);
    status.set_widget_name("app-update-status");
    body.append(&status);
    check.connect_clicked(move |button| {
        button.set_sensitive(false);
        status.set_text("Checking GitHub for the latest stable release…");
        let weak = button.downgrade();
        let done = status.clone();
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = (|| {
                let out = std::process::Command::new("curl")
                    .args([
                        "--fail",
                        "--silent",
                        "--show-error",
                        "--max-time",
                        "15",
                        "--max-filesize",
                        "1048576",
                        "--proto",
                        "=https",
                        "-H",
                        "Accept: application/vnd.github+json",
                        "-H",
                        "User-Agent: FileWheel",
                        "https://api.github.com/repos/danny19862025/FileWheel/releases/latest",
                    ])
                    .output()
                    .map_err(|e| {
                        format!("Update check failed: {e}. Open Releases to check manually.")
                    })?;
                if !out.status.success() {
                    return Err("Update check unavailable. Try again or open Releases.".to_string());
                }
                release_summary(
                    &String::from_utf8_lossy(&out.stdout),
                    env!("CARGO_PKG_VERSION"),
                )
            })();
            let _ = send.send(result);
        });
        let done = done.downgrade();
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            let Some(done) = done.upgrade() else {
                return gtk::glib::ControlFlow::Break;
            };
            match receive.try_recv() {
                Ok(result) => {
                    done.set_text(&result.unwrap_or_else(|error| error));
                    if let Some(button) = weak.upgrade() {
                        button.set_sensitive(true);
                    }
                    gtk::glib::ControlFlow::Break
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => gtk::glib::ControlFlow::Continue,
                Err(_) => {
                    done.set_text("Update check interrupted. Try again or open Releases.");
                    if let Some(button) = weak.upgrade() {
                        button.set_sensitive(true);
                    }
                    gtk::glib::ControlFlow::Break
                }
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn release_versions_are_compared_numerically() {
        assert!(
            release_summary(r#"{"tag_name":"v0.10.0"}"#, "0.9.0")
                .unwrap()
                .contains("is available")
        );
        assert!(
            release_summary(r#"{"tag_name":"v0.1.0"}"#, "0.1.0")
                .unwrap()
                .contains("up to date")
        );
        assert!(
            release_summary(r#"{"tag_name":"v0.1.0"}"#, "0.2.0")
                .unwrap()
                .contains("up to date")
        );
        assert!(release_summary(r#"{"message":"rate limit"}"#, "0.1.0").is_err());
        assert!(
            release_summary(r#"{"tag_name":"v1.0.0-rc1","prerelease":true}"#, "0.1.0").is_err()
        );
    }
}
