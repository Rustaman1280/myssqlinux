use mysqldesk::system::port::PortChecker;
use std::net::TcpListener;

#[test]
fn test_port_detection_available_port() {
    // Bind to an OS-assigned ephemeral port (0), get assigned port, then close it
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    // Port should now be available
    assert!(!PortChecker::is_in_use(port));
}

#[test]
fn test_port_detection_in_use_port() {
    // Bind to an ephemeral port and keep it open
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    // Port should report in use
    assert!(PortChecker::is_in_use(port));
    drop(listener);
}

#[test]
fn test_port_conflict_reporting() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    // Expected service is NOT running, but port is in use -> conflict error!
    let result = PortChecker::check_conflict(port, false);
    assert!(result.is_err());

    // Expected service IS running, so port in use is expected -> Ok
    let result_expected = PortChecker::check_conflict(port, true);
    assert!(result_expected.is_ok());

    drop(listener);
}
