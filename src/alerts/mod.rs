//! Desktop notification system with per-category cooldown timers.

use std::collections::HashMap;
use log;

/// Manages alert dispatching with cooldown to prevent spam.
pub struct AlertManager {
    last_alert: HashMap<String, i64>,
    cooldown_secs: u64,
    enabled: bool,
}

impl AlertManager {
    pub fn new(enabled: bool, cooldown_secs: u64) -> Self {
        AlertManager {
            last_alert: HashMap::new(),
            cooldown_secs,
            enabled,
        }
    }

    /// Try to send a notification.  Returns `true` if the notification
    /// was actually dispatched (i.e., not suppressed by cooldown).
    pub fn try_send(&mut self, category: &str, title: &str, body: &str) -> bool {
        if !self.enabled {
            return false;
        }

        let now = chrono::Utc::now().timestamp();

        if let Some(&last) = self.last_alert.get(category) {
            if now - last < self.cooldown_secs as i64 {
                log::debug!("Alert '{}' suppressed (cooldown active)", category);
                return false;
            }
        }

        self.last_alert.insert(category.to_string(), now);
        send_notification(title, body);
        log::info!("Alert sent [{}]: {}", category, title);
        true
    }
}

/// Platform-specific notification dispatch.
fn send_notification(title: &str, body: &str) {
    #[cfg(target_os = "macos")]
    {
        let script = std::format!(
            "display notification \"{}\" with title \"{}\"",
            body.replace('"', "\\\""),
            title.replace('"', "\\\""),
        );
        let _ = std::process::Command::new("osascript")
            .args(["-e", &script])
            .output();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("notify-send")
            .args([title, body])
            .output();
    }

    #[cfg(target_os = "windows")]
    {
        let ps = std::format!(
            "Add-Type -AssemblyName System.Windows.Forms; \
             $n = New-Object System.Windows.Forms.NotifyIcon; \
             $n.Icon = [System.Drawing.SystemIcons]::Information; \
             $n.Visible = $true; \
             $n.ShowBalloonTip(5000, '{}', '{}', 'Info'); \
             Start-Sleep -Seconds 6; $n.Dispose()",
            title.replace('\'', "''"),
            body.replace('\'', "''"),
        );
        let _ = std::process::Command::new("powershell")
            .args(["-Command", &ps])
            .output();
    }
}
