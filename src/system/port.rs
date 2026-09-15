use crate::error::{AppError, AppResult};
use crate::system::process::SafeCommand;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;
use tracing::debug;

pub struct PortChecker;

impl PortChecker {
    /// Probe if a TCP port is in use by attempting to connect or bind
    pub fn is_in_use(port: u16) -> bool {
        // Fast test: try connecting to 127.0.0.1:port
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        if TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok() {
            return true;
        }

        // Second test: try to bind listener
        match TcpListener::bind(addr) {
            Ok(_) => false,
            Err(e) => {
                debug!("Port {} bind error: {}", port, e);
                true
            }
        }
    }

    /// Try to discover what process is occupying the port (using ss or lsof if available)
    pub fn find_process_on_port(port: u16) -> Option<String> {
        let port_filter = format!(":{}", port);

        // Try ss -tulpn
        if let Some(out) = SafeCommand::run_and_get_stdout("ss", &["-tulpn"]) {
            for line in out.lines() {
                if line.contains(&port_filter) {
                    return Some(line.trim().to_string());
                }
            }
        }

        // Try fuser if available
        let port_spec = format!("{}/tcp", port);
        if let Some(out) = SafeCommand::run_and_get_stdout("fuser", &[&port_spec]) {
            return Some(format!("PID: {}", out.trim()));
        }

        None
    }

    /// Check port conflict
    pub fn check_conflict(port: u16, is_expected_service_running: bool) -> AppResult<()> {
        // If the database service is already running, port is legitimately in use by it.
        // We only check for conflict when the database service is STOPPED.
        if is_expected_service_running {
            return Ok(());
        }

        let in_use = Self::is_in_use(port);
        if in_use {
            let proc_info = Self::find_process_on_port(port)
                .unwrap_or_else(|| "Proses tidak dikenal atau memerlukan izin admin untuk melihat PID".to_string());
            return Err(AppError::PortConflict {
                port,
                details: format!("Port {} aktif, tetapi server database tercatat mati. Kemungkinan proses lain sedang menggunakannya: {}", port, proc_info),
            });
        }
        Ok(())
    }
}
