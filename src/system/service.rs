use crate::error::{AppError, AppResult};
use crate::system::process::SafeCommand;
use tracing::{error, info};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    Running,
    Stopped,
    Starting,
    Stopping,
    Error,
    Unknown,
}

impl ServiceState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Stopped => "Stopped",
            Self::Starting => "Starting",
            Self::Stopping => "Stopping",
            Self::Error => "Error",
            Self::Unknown => "Unknown",
        }
    }

    pub fn badge_class(&self) -> &'static str {
        match self {
            Self::Running => "status-running",
            Self::Stopped => "status-stopped",
            Self::Starting => "status-starting",
            Self::Stopping => "status-stopping",
            Self::Error => "status-error",
            Self::Unknown => "status-unknown",
        }
    }
}

pub struct SystemdManager;

impl SystemdManager {
    /// Check if systemd / systemctl is available on this system
    pub fn is_available() -> bool {
        SafeCommand::binary_exists("systemctl")
    }

    /// Parse active state and substate into ServiceState enum
    pub fn parse_state(active_state: &str, sub_state: &str) -> ServiceState {
        match active_state.trim() {
            "active" => match sub_state.trim() {
                "running" => ServiceState::Running,
                "reloading" => ServiceState::Starting,
                _ => ServiceState::Running,
            },
            "inactive" => ServiceState::Stopped,
            "activating" => ServiceState::Starting,
            "deactivating" => ServiceState::Stopping,
            "failed" => ServiceState::Error,
            _ => ServiceState::Unknown,
        }
    }

    /// Query the status of a systemd unit
    pub fn get_status(service_name: &str) -> ServiceState {
        if !Self::is_available() {
            return ServiceState::Unknown;
        }

        // Fast check using systemctl is-active
        if let Some(active_out) = SafeCommand::run_and_get_stdout("systemctl", &["is-active", service_name]) {
            match active_out.as_str() {
                "active" => return ServiceState::Running,
                "inactive" => return ServiceState::Stopped,
                "activating" => return ServiceState::Starting,
                "deactivating" => return ServiceState::Stopping,
                "failed" => return ServiceState::Error,
                _ => {}
            }
        }

        // Detailed check using systemctl show
        let output = SafeCommand::run(
            "systemctl",
            &["show", service_name, "--property=ActiveState,SubState"],
        );

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let mut active_state = "unknown";
                let mut sub_state = "unknown";

                for line in stdout.lines() {
                    if let Some(val) = line.strip_prefix("ActiveState=") {
                        active_state = val;
                    } else if let Some(val) = line.strip_prefix("SubState=") {
                        sub_state = val;
                    }
                }

                Self::parse_state(active_state, sub_state)
            }
            _ => ServiceState::Unknown,
        }
    }

    /// Execute a privileged service action (start, stop, restart) via PolicyKit pkexec
    pub fn execute_action(service_name: &str, action: &str) -> AppResult<()> {
        if !Self::is_available() {
            return Err(AppError::ServiceNotFound(
                "systemctl tidak ditemukan".to_string(),
            ));
        }

        info!("Requesting privileged action '{}' on service '{}'", action, service_name);

        let output = if SafeCommand::binary_exists("pkexec") {
            // Use PolicyKit escalation without running the GUI as root
            SafeCommand::run("pkexec", &["systemctl", action, service_name])
        } else {
            // Fallback to direct systemctl
            SafeCommand::run("systemctl", &[action, service_name])
        };

        match output {
            Ok(res) => {
                if res.status.success() {
                    info!("Successfully executed '{}' on '{}'", action, service_name);
                    Ok(())
                } else {
                    let code = res.status.code().unwrap_or(-1);
                    let stderr = String::from_utf8_lossy(&res.stderr).trim().to_string();
                    let stdout = String::from_utf8_lossy(&res.stdout).trim().to_string();
                    let details = if !stderr.is_empty() { stderr } else { stdout };

                    // pkexec exit code 126 or 127 usually means user dismissed/cancelled authorization dialog
                    if code == 126 || code == 127 || details.contains("dismissed") || details.contains("cancelled") {
                        error!("User cancelled or rejected pkexec authentication");
                        Err(AppError::PermissionDenied("Otorisasi PolicyKit dibatalkan atau ditolak.".to_string()))
                    } else {
                        error!("Failed to {} {}: code={}, details={}", action, service_name, code, details);
                        Err(AppError::ServiceActionFailed {
                            service: service_name.to_string(),
                            action: action.to_string(),
                            details,
                        })
                    }
                }
            }
            Err(e) => {
                error!("IO error while attempting {} on {}: {}", action, service_name, e);
                Err(AppError::IoError(e.to_string()))
            }
        }
    }

    /// Read journalctl logs for a service
    pub fn get_journal_logs(service_name: &str, lines: usize) -> AppResult<String> {
        let lines_arg = format!("-n{}", lines);
        let output = SafeCommand::run(
            "journalctl",
            &["-u", service_name, &lines_arg, "--no-pager"],
        );

        match output {
            Ok(res) => {
                let stdout = String::from_utf8_lossy(&res.stdout).to_string();
                if stdout.trim().is_empty() {
                    let stderr = String::from_utf8_lossy(&res.stderr).to_string();
                    if !stderr.trim().is_empty() {
                        Ok(stderr)
                    } else {
                        Ok(format!("Belum ada log journalctl untuk service '{}'.", service_name))
                    }
                } else {
                    Ok(stdout)
                }
            }
            Err(e) => Err(AppError::IoError(e.to_string())),
        }
    }
}
