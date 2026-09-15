use crate::error::{AppError, AppResult};
use crate::system::port::PortChecker;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tracing::{error, info};

#[derive(Clone, Default)]
pub struct PhpMyAdminLauncher {
    child: Arc<Mutex<Option<Child>>>,
}

impl PhpMyAdminLauncher {
    pub fn new() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
        }
    }

    /// Check if the local PHP development server is running
    pub fn is_running(&self, port: u16) -> bool {
        let mut guard = self.child.lock().unwrap();
        if let Some(child) = guard.as_mut() {
            match child.try_wait() {
                Ok(None) => true, // Still running
                _ => {
                    *guard = None;
                    false
                }
            }
        } else {
            // Also check if port is in use
            PortChecker::is_in_use(port)
        }
    }

    /// Start the local phpMyAdmin server
    pub fn start(&self, php_bin: &str, pma_dir: &str, host: &str, port: u16) -> AppResult<()> {
        let mut guard = self.child.lock().unwrap();
        if guard.is_some() {
            info!("phpMyAdmin server is already running");
            return Ok(());
        }

        // Safety: ensure binding is to localhost / 127.0.0.1
        let safe_host = if host.trim().is_empty() {
            "127.0.0.1"
        } else {
            host.trim()
        };

        let bind_target = format!("{}:{}", safe_host, port);
        info!("Launching phpMyAdmin built-in server on http://{} from {}", bind_target, pma_dir);

        let child = Command::new(php_bin)
            .args(["-S", &bind_target, "-t", pma_dir])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                error!("Failed to spawn PHP built-in server: {}", e);
                AppError::PhpMyAdminLaunchFailed(format!("Gagal mengeksekusi PHP server: {}", e))
            })?;

        *guard = Some(child);
        info!("phpMyAdmin server started successfully on http://{}", bind_target);
        Ok(())
    }

    /// Stop the local phpMyAdmin server
    pub fn stop(&self) -> AppResult<()> {
        let mut guard = self.child.lock().unwrap();
        if let Some(mut child) = guard.take() {
            info!("Stopping phpMyAdmin child process (PID: {})...", child.id());
            let _ = child.kill();
            let _ = child.wait();
            info!("phpMyAdmin server stopped");
        }
        Ok(())
    }

    /// Open phpMyAdmin in user's default Linux browser
    pub fn open_browser(host: &str, port: u16) -> AppResult<()> {
        let url = format!("http://{}:{}", host, port);
        info!("Opening {} in default web browser...", url);
        open::that(&url).map_err(|e| AppError::NetworkError(format!("Gagal membuka browser: {}", e)))
    }
}

impl Drop for PhpMyAdminLauncher {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
