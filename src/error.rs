use std::fmt;

#[derive(Debug, Clone)]
pub enum AppError {
    ServiceNotFound(String),
    ServiceActionFailed { service: String, action: String, details: String },
    PermissionDenied(String),
    PortConflict { port: u16, details: String },
    DatabaseNotInstalled,
    PhpNotInstalled,
    PhpMyAdminNotInstalled,
    PhpMyAdminLaunchFailed(String),
    NetworkError(String),
    IoError(String),
    ConfigError(String),
    Cancelled,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ServiceNotFound(name) => write!(
                f,
                "Service systemd '{}' tidak ditemukan pada sistem.",
                name
            ),
            Self::ServiceActionFailed { service, action, details } => write!(
                f,
                "Gagal menjalankan '{}' pada service {}: {}",
                action, service, details
            ),
            Self::PermissionDenied(msg) => write!(
                f,
                "Tidak memiliki izin administrator untuk mengontrol service database. Silakan masukkan password PolicyKit dengan benar: {}",
                msg
            ),
            Self::PortConflict { port, details } => write!(
                f,
                "Port {} sedang digunakan oleh proses lain: {}",
                port, details
            ),
            Self::DatabaseNotInstalled => write!(
                f,
                "Server MariaDB atau MySQL belum terpasang pada sistem ini."
            ),
            Self::PhpNotInstalled => write!(
                f,
                "Runtime PHP belum terpasang. phpMyAdmin membutuhkan PHP CLI."
            ),
            Self::PhpMyAdminNotInstalled => write!(
                f,
                "phpMyAdmin belum terpasang atau belum diunduh."
            ),
            Self::PhpMyAdminLaunchFailed(msg) => write!(
                f,
                "Gagal menjalankan server lokal phpMyAdmin: {}",
                msg
            ),
            Self::NetworkError(msg) => write!(f, "Kesalahan jaringan: {}", msg),
            Self::IoError(msg) => write!(f, "Kesalahan I/O: {}", msg),
            Self::ConfigError(msg) => write!(f, "Kesalahan konfigurasi: {}", msg),
            Self::Cancelled => write!(f, "Operasi dibatalkan oleh pengguna."),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
