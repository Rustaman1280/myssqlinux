use crate::error::{AppError, AppResult};
use crate::system::process::SafeCommand;
use tracing::{info, warn};

pub struct DatabaseAuth;

impl DatabaseAuth {
    /// Test if root can connect locally with an empty password
    pub fn test_root_empty_password() -> bool {
        let bin = if SafeCommand::binary_exists("mariadb") {
            "mariadb"
        } else if SafeCommand::binary_exists("mysql") {
            "mysql"
        } else {
            return false;
        };

        match SafeCommand::run(bin, &["-u", "root", "-e", "SELECT 1;"]) {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Configure MariaDB / MySQL root user to have empty password (XAMPP development style)
    /// Uses PolicyKit pkexec for privilege escalation
    pub fn setup_root_empty_password() -> AppResult<()> {
        info!("Setting up root user with empty password for development mode...");

        let sql = "ALTER USER 'root'@'localhost' IDENTIFIED VIA mysql_native_password USING PASSWORD(''); GRANT ALL PRIVILEGES ON *.* TO 'root'@'localhost' WITH GRANT OPTION; FLUSH PRIVILEGES;";

        let bin = if SafeCommand::binary_exists("mariadb") {
            "mariadb"
        } else if SafeCommand::binary_exists("mysql") {
            "mysql"
        } else {
            return Err(AppError::DatabaseNotInstalled);
        };

        let output = if SafeCommand::binary_exists("pkexec") {
            SafeCommand::run("pkexec", &[bin, "-e", sql])
        } else {
            SafeCommand::run(bin, &["-e", sql])
        };

        match output {
            Ok(res) => {
                if res.status.success() {
                    info!("Successfully configured root with empty password.");
                    Ok(())
                } else {
                    let code = res.status.code().unwrap_or(-1);
                    let stderr = String::from_utf8_lossy(&res.stderr).trim().to_string();
                    let stdout = String::from_utf8_lossy(&res.stdout).trim().to_string();
                    let details = if !stderr.is_empty() { stderr } else { stdout };

                    if code == 126 || code == 127 || details.contains("dismissed") || details.contains("cancelled") {
                        warn!("User cancelled PolicyKit authorization for root configuration");
                        Err(AppError::PermissionDenied("Otorisasi PolicyKit dibatalkan.".to_string()))
                    } else {
                        Err(AppError::ServiceActionFailed {
                            service: "mariadb".to_string(),
                            action: "setup root empty password".to_string(),
                            details,
                        })
                    }
                }
            }
            Err(e) => Err(AppError::IoError(e.to_string())),
        }
    }
}
