use crate::system::process::SafeCommand;
use std::path::Path;
use tracing::{debug, info};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    MariaDB,
    MySQL,
    None,
}

impl DatabaseType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MariaDB => "MariaDB",
            Self::MySQL => "MySQL",
            Self::None => "None",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseInfo {
    pub db_type: DatabaseType,
    pub version: String,
    pub service_name: String,
    pub port: u16,
    pub data_dir: String,
    pub config_file: String,
    pub socket_path: String,
    pub executable_path: String,
    pub bind_address: String,
}

impl Default for DatabaseInfo {
    fn default() -> Self {
        Self {
            db_type: DatabaseType::None,
            version: "Not installed".to_string(),
            service_name: "mariadb.service".to_string(),
            port: 3306,
            data_dir: "Unknown".to_string(),
            config_file: "Unknown".to_string(),
            socket_path: "Unknown".to_string(),
            executable_path: "Unknown".to_string(),
            bind_address: "127.0.0.1".to_string(),
        }
    }
}

pub struct DatabaseDetector;

impl DatabaseDetector {
    /// Detect if MariaDB or MySQL is available on the system
    pub fn detect() -> DatabaseInfo {
        info!("Scanning system for MariaDB and MySQL installations...");

        // Priority 1: MariaDB (very common on modern Linux distributions)
        if let Some(info) = Self::detect_mariadb() {
            info!("Found MariaDB: {}", info.version);
            return info;
        }

        // Priority 2: MySQL
        if let Some(info) = Self::detect_mysql() {
            info!("Found MySQL: {}", info.version);
            return info;
        }

        info!("No database server found on system");
        DatabaseInfo::default()
    }

    /// Detect MariaDB details
    fn detect_mariadb() -> Option<DatabaseInfo> {
        let has_mariadbd = SafeCommand::binary_exists("mariadbd");
        let has_mariadb_cli = SafeCommand::binary_exists("mariadb");

        if !has_mariadbd && !has_mariadb_cli {
            return None;
        }

        let exec_path = which::which("mariadbd")
            .or_else(|_| which::which("mariadb"))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "/usr/sbin/mariadbd".to_string());

        let raw_ver = SafeCommand::run_and_get_stdout("mariadb", &["--version"])
            .or_else(|| SafeCommand::run_and_get_stdout("mariadbd", &["--version"]))
            .unwrap_or_default();

        let version = Self::parse_version(&raw_ver).unwrap_or_else(|| "MariaDB (version unknown)".to_string());

        // Parse variables from mariadbd --help --verbose without requiring root
        let (port, data_dir, socket, bind_addr) = Self::extract_defaults("mariadbd")
            .or_else(|| Self::extract_defaults("mysqld"))
            .unwrap_or((3306, "/var/lib/mysql".to_string(), "/run/mysqld/mysqld.sock".to_string(), "127.0.0.1".to_string()));

        let config_file = Self::find_config_file(&[
            "/etc/mysql/mariadb.cnf",
            "/etc/mysql/my.cnf",
            "/etc/my.cnf.d/server.cnf",
            "/etc/my.cnf",
        ]).unwrap_or_else(|| "/etc/mysql/mariadb.cnf".to_string());

        // Check active service unit
        let service_name = if SafeCommand::run_and_get_stdout("systemctl", &["status", "mariadb.service"]).is_some() {
            "mariadb.service".to_string()
        } else if SafeCommand::run_and_get_stdout("systemctl", &["status", "mysql.service"]).is_some() {
            "mysql.service".to_string()
        } else {
            "mariadb.service".to_string()
        };

        Some(DatabaseInfo {
            db_type: DatabaseType::MariaDB,
            version,
            service_name,
            port,
            data_dir,
            config_file,
            socket_path: socket,
            executable_path: exec_path,
            bind_address: bind_addr,
        })
    }

    /// Detect MySQL details
    fn detect_mysql() -> Option<DatabaseInfo> {
        let has_mysqld = SafeCommand::binary_exists("mysqld");
        let has_mysql_cli = SafeCommand::binary_exists("mysql");

        if !has_mysqld && !has_mysql_cli {
            return None;
        }

        // Ensure it's not MariaDB masquerading as mysql
        let raw_ver = SafeCommand::run_and_get_stdout("mysql", &["--version"]).unwrap_or_default();
        if raw_ver.contains("MariaDB") {
            // Already handled in MariaDB detector
            return None;
        }

        let exec_path = which::which("mysqld")
            .or_else(|_| which::which("mysql"))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "/usr/sbin/mysqld".to_string());

        let version = Self::parse_version(&raw_ver).unwrap_or_else(|| "MySQL (version unknown)".to_string());

        let (port, data_dir, socket, bind_addr) = Self::extract_defaults("mysqld")
            .unwrap_or((3306, "/var/lib/mysql".to_string(), "/run/mysqld/mysqld.sock".to_string(), "127.0.0.1".to_string()));

        let config_file = Self::find_config_file(&[
            "/etc/mysql/my.cnf",
            "/etc/mysql/mysql.cnf",
            "/etc/my.cnf",
        ]).unwrap_or_else(|| "/etc/mysql/my.cnf".to_string());

        Some(DatabaseInfo {
            db_type: DatabaseType::MySQL,
            version,
            service_name: "mysql.service".to_string(),
            port,
            data_dir,
            config_file,
            socket_path: socket,
            executable_path: exec_path,
            bind_address: bind_addr,
        })
    }

    /// Extract version string cleanly from --version output
    pub fn parse_version(raw: &str) -> Option<String> {
        if raw.is_empty() {
            return None;
        }

        // Example MariaDB: "mariadb from 11.8.6-MariaDB, client 15.2 for debian-linux-gnu (x86_64) using EditLine wrapper"
        // Example MySQL: "mysql  Ver 8.0.36-0ubuntu0.24.04.1 for Linux on x86_64 ((Ubuntu))"
        for part in raw.split_whitespace() {
            if part.contains("MariaDB") {
                let cleaned = part.trim_end_matches(',');
                return Some(format!("MariaDB {}", cleaned.replace("-MariaDB", "")));
            }
            if part.starts_with("8.") || part.starts_with("5.7") || part.starts_with("9.") {
                let cleaned = part.trim_end_matches(',');
                return Some(format!("MySQL {}", cleaned));
            }
        }

        // Fallback: first 40 chars
        Some(raw.lines().next().unwrap_or(raw).trim().chars().take(40).collect())
    }

    /// Extract variables (port, datadir, socket, bind-address) via daemon --help --verbose
    fn extract_defaults(daemon_bin: &str) -> Option<(u16, String, String, String)> {
        let out = SafeCommand::run_and_get_stdout(daemon_bin, &["--help", "--verbose"])?;

        let mut port = 3306;
        let mut datadir = "/var/lib/mysql".to_string();
        let mut socket = "/run/mysqld/mysqld.sock".to_string();
        let mut bind_addr = "127.0.0.1".to_string();

        for line in out.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                match parts[0] {
                    "port" => {
                        if let Ok(p) = parts[1].parse::<u16>() {
                            if p > 0 {
                                port = p;
                            }
                        }
                    }
                    "datadir" => {
                        datadir = parts[1].to_string();
                    }
                    "socket" => {
                        socket = parts[1].to_string();
                    }
                    "bind-address" => {
                        bind_addr = parts[1].to_string();
                    }
                    _ => {}
                }
            }
        }

        debug!("Extracted from {}: port={}, datadir={}, socket={}", daemon_bin, port, datadir, socket);
        Some((port, datadir, socket, bind_addr))
    }

    /// Find first existing config file path
    pub fn find_config_file(candidates: &[&str]) -> Option<String> {
        for candidate in candidates {
            if Path::new(candidate).exists() {
                return Some(candidate.to_string());
            }
        }
        None
    }
}
