use crate::config::AppSettings;
use crate::database::{DatabaseDetector, DatabaseInfo, DatabaseService};
use crate::phpmyadmin::{PhpMyAdminDetector, PhpMyAdminInfo, PhpMyAdminLauncher};
use crate::system::service::ServiceState;
use std::sync::{Arc, Mutex};
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<Mutex<AppSettings>>,
    pub db_info: Arc<Mutex<DatabaseInfo>>,
    pub db_service: Arc<Mutex<DatabaseService>>,
    pub pma_info: Arc<Mutex<PhpMyAdminInfo>>,
    pub pma_launcher: PhpMyAdminLauncher,
    pub cached_db_state: Arc<Mutex<ServiceState>>,
}

impl AppState {
    pub fn new() -> Self {
        let settings = AppSettings::load();
        let db_info = DatabaseDetector::detect();
        let db_service = DatabaseService::new(db_info.service_name.clone());
        let current_state = db_service.get_status();
        let pma_info = PhpMyAdminDetector::detect(
            settings.custom_php_path.as_deref(),
            settings.custom_pma_path.as_deref(),
        );
        let pma_launcher = PhpMyAdminLauncher::new();

        info!(
            "App state initialized. DB: {} ({:?}), PMA installed: {}",
            db_info.version, current_state, pma_info.pma_installed
        );

        Self {
            settings: Arc::new(Mutex::new(settings)),
            db_info: Arc::new(Mutex::new(db_info)),
            db_service: Arc::new(Mutex::new(db_service)),
            pma_info: Arc::new(Mutex::new(pma_info)),
            pma_launcher,
            cached_db_state: Arc::new(Mutex::new(current_state)),
        }
    }

    /// Refresh detections and status
    pub fn refresh_all(&self) {
        let db_svc = self.db_service.lock().unwrap();
        let state = db_svc.get_status();
        *self.cached_db_state.lock().unwrap() = state;

        let settings = self.settings.lock().unwrap().clone();
        let pma_info = PhpMyAdminDetector::detect(
            settings.custom_php_path.as_deref(),
            settings.custom_pma_path.as_deref(),
        );
        *self.pma_info.lock().unwrap() = pma_info;
    }
}
