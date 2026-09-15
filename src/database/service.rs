use crate::error::AppResult;
use crate::system::service::{ServiceState, SystemdManager};
use tracing::info;

pub struct DatabaseService {
    service_name: String,
}

impl DatabaseService {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn set_service_name(&mut self, name: String) {
        self.service_name = name;
    }

    pub fn get_status(&self) -> ServiceState {
        SystemdManager::get_status(&self.service_name)
    }

    pub fn is_running(&self) -> bool {
        self.get_status() == ServiceState::Running
    }

    pub fn start(&self) -> AppResult<()> {
        info!("Starting database service '{}'", self.service_name);
        SystemdManager::execute_action(&self.service_name, "start")
    }

    pub fn stop(&self) -> AppResult<()> {
        info!("Stopping database service '{}'", self.service_name);
        SystemdManager::execute_action(&self.service_name, "stop")
    }

    pub fn restart(&self) -> AppResult<()> {
        info!("Restarting database service '{}'", self.service_name);
        SystemdManager::execute_action(&self.service_name, "restart")
    }

    pub fn get_logs(&self, lines: usize) -> AppResult<String> {
        SystemdManager::get_journal_logs(&self.service_name, lines)
    }
}
