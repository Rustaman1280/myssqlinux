use mysqldesk::system::service::{ServiceState, SystemdManager};

#[test]
fn test_systemd_status_parsing_running() {
    let state = SystemdManager::parse_state("active", "running");
    assert_eq!(state, ServiceState::Running);
}

#[test]
fn test_systemd_status_parsing_stopped() {
    let state = SystemdManager::parse_state("inactive", "dead");
    assert_eq!(state, ServiceState::Stopped);
}

#[test]
fn test_systemd_status_parsing_activating() {
    let state = SystemdManager::parse_state("activating", "start-pre");
    assert_eq!(state, ServiceState::Starting);
}

#[test]
fn test_systemd_status_parsing_deactivating() {
    let state = SystemdManager::parse_state("deactivating", "stop-sigterm");
    assert_eq!(state, ServiceState::Stopping);
}

#[test]
fn test_systemd_status_parsing_failed() {
    let state = SystemdManager::parse_state("failed", "failed");
    assert_eq!(state, ServiceState::Error);
}

#[test]
fn test_systemd_status_parsing_unknown() {
    let state = SystemdManager::parse_state("invalid", "state");
    assert_eq!(state, ServiceState::Unknown);
}
