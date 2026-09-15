use std::fs;
use std::path::Path;
use tracing::info;

pub struct DatabaseConfig;

impl DatabaseConfig {
    /// Read content of a database configuration file safely
    pub fn read_file(path: &str) -> Option<String> {
        let p = Path::new(path);
        if p.exists() {
            fs::read_to_string(p).ok()
        } else {
            None
        }
    }

    /// Extract port from config file content if specified
    pub fn parse_port_from_content(content: &str) -> Option<u16> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if let Some((key, val)) = line.split_once('=') {
                if key.trim().eq_ignore_ascii_case("port") {
                    if let Ok(p) = val.trim().parse::<u16>() {
                        return Some(p);
                    }
                }
            }
        }
        None
    }

    /// Open configuration file in the user's default desktop editor (e.g. gnome-text-editor, gedit, nano)
    pub fn open_in_editor(path: &str) -> Result<(), String> {
        info!("Opening config file {} in default application", path);
        open::that(path).map_err(|e| e.to_string())
    }
}
