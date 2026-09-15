use crate::system::process::SafeCommand;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct PhpMyAdminInfo {
    pub php_installed: bool,
    pub php_version: String,
    pub php_extensions_ok: bool,
    pub missing_extensions: Vec<String>,
    pub pma_installed: bool,
    pub pma_version: Option<String>,
    pub pma_path: Option<String>,
}

impl Default for PhpMyAdminInfo {
    fn default() -> Self {
        Self {
            php_installed: false,
            php_version: "Not installed".to_string(),
            php_extensions_ok: false,
            missing_extensions: Vec::new(),
            pma_installed: false,
            pma_version: None,
            pma_path: None,
        }
    }
}

pub struct PhpMyAdminDetector;

impl PhpMyAdminDetector {
    pub fn default_local_pma_dir() -> PathBuf {
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("mysqldesk").join("phpmyadmin")
    }

    pub fn detect(custom_php: Option<&str>, custom_pma: Option<&str>) -> PhpMyAdminInfo {
        info!("Scanning system for PHP and phpMyAdmin...");

        let mut info = PhpMyAdminInfo::default();

        // 1. Detect PHP
        let php_bin = custom_php.unwrap_or("php");
        if SafeCommand::binary_exists(php_bin) {
            info.php_installed = true;
            if let Some(ver_out) = SafeCommand::run_and_get_stdout(php_bin, &["--version"]) {
                if let Some(first_line) = ver_out.lines().next() {
                    info.php_version = first_line.trim().to_string();
                }
            }

            // Check extensions
            if let Some(modules_out) = SafeCommand::run_and_get_stdout(php_bin, &["-m"]) {
                let required = ["mysqli", "mbstring", "session", "json"];
                let mut missing = Vec::new();
                for req in required {
                    if !modules_out.contains(req) {
                        missing.push(req.to_string());
                    }
                }
                info.php_extensions_ok = missing.is_empty();
                info.missing_extensions = missing;
            }
        }

        // 2. Detect phpMyAdmin
        let mut candidates = Vec::new();
        if let Some(c) = custom_pma {
            candidates.push(PathBuf::from(c));
        }

        let local_dir = Self::default_local_pma_dir();
        candidates.push(local_dir);
        candidates.push(PathBuf::from("/usr/share/phpmyadmin"));
        candidates.push(PathBuf::from("/var/www/html/phpmyadmin"));
        candidates.push(PathBuf::from("/var/www/phpmyadmin"));

        for path in candidates {
            if path.join("index.php").exists() {
                info.pma_installed = true;
                info.pma_path = Some(path.to_string_lossy().to_string());
                info.pma_version = Self::read_pma_version(&path);
                debug!("Found phpMyAdmin at {:?}", path);
                break;
            }
        }

        info
    }

    fn read_pma_version(dir: &Path) -> Option<String> {
        // Try reading RELEASE-DATE-x.x.x
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Some(ver) = name.strip_prefix("RELEASE-DATE-") {
                    return Some(ver.to_string());
                }
            }
        }

        // Try reading package.json
        let pkg_json = dir.join("package.json");
        if pkg_json.exists() {
            if let Ok(content) = fs::read_to_string(&pkg_json) {
                for line in content.lines() {
                    if line.contains("\"version\":") {
                        let parts: Vec<&str> = line.split('"').collect();
                        if parts.len() >= 4 {
                            return Some(parts[3].to_string());
                        }
                    }
                }
            }
        }

        Some("5.x".to_string())
    }
}
