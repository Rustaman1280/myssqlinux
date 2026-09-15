use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub db_port: u16,
    pub phpmyadmin_port: u16,
    pub phpmyadmin_host: String,
    pub theme: String, // "system", "dark", "light"
    pub auto_refresh_interval_secs: u64,
    pub custom_php_path: Option<String>,
    pub custom_pma_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            db_port: 3306,
            phpmyadmin_port: 8080,
            phpmyadmin_host: "127.0.0.1".to_string(),
            theme: "system".to_string(),
            auto_refresh_interval_secs: 3,
            custom_php_path: None,
            custom_pma_path: None,
        }
    }
}

impl AppSettings {
    pub fn config_path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("mysqldesk").join("settings.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(file) = File::open(&path) {
                let reader = BufReader::new(file);
                if let Ok(settings) = serde_json::from_reader(reader) {
                    info!("Loaded settings from {:?}", path);
                    return settings;
                }
            }
        }
        info!("Using default settings");
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let file = File::create(&path).map_err(|e| e.to_string())?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self).map_err(|e| {
            error!("Failed to serialize settings: {}", e);
            e.to_string()
        })?;
        info!("Saved settings to {:?}", path);
        Ok(())
    }
}
