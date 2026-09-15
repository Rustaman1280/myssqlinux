use mysqldesk::database::DatabaseConfig;
use mysqldesk::system::detect_package_manager;

#[test]
fn test_parse_port_from_my_cnf() {
    let sample_cnf = r#"
[client]
port = 3306
socket = /run/mysqld/mysqld.sock

[mysqld]
user = mysql
port = 3308
bind-address = 127.0.0.1
"#;

    let port = DatabaseConfig::parse_port_from_content(sample_cnf);
    assert_eq!(port, Some(3306)); // first found or parsed
}

#[test]
fn test_package_manager_detection() {
    let pm = detect_package_manager();
    assert!(pm.is_available());
    assert!(!pm.name().is_empty());
    assert!(!pm.mariadb_package_name().is_empty());
}
