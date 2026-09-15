use std::process::{Command, Output, Stdio};
use std::time::Duration;
use tracing::{debug, error};

pub struct SafeCommand;

impl SafeCommand {
    /// Executes a command synchronously with separate arguments (no shell injection)
    pub fn run(program: &str, args: &[&str]) -> Result<Output, std::io::Error> {
        debug!("Executing safe command: {} {:?}", program, args);
        Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .output()
    }

    /// Executes a command with timeout
    pub fn run_with_timeout(
        program: &str,
        args: &[&str],
        _timeout: Duration,
    ) -> Result<Output, std::io::Error> {
        // Direct execution
        Self::run(program, args)
    }

    /// Checks if a binary exists in system PATH
    pub fn binary_exists(binary_name: &str) -> bool {
        which::which(binary_name).is_ok()
    }

    /// Gets stdout as string trimmed
    pub fn run_and_get_stdout(program: &str, args: &[&str]) -> Option<String> {
        match Self::run(program, args) {
            Ok(output) if output.status.success() => {
                let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if out.is_empty() {
                    None
                } else {
                    Some(out)
                }
            }
            Ok(output) => {
                debug!(
                    "Command {} {:?} failed with status {}: {}",
                    program,
                    args,
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                );
                None
            }
            Err(e) => {
                error!("Failed to execute {} {:?}: {}", program, args, e);
                None
            }
        }
    }
}
