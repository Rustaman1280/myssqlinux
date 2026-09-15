use mysqldesk::database::DatabaseDetector;

#[test]
fn test_mariadb_version_parsing_debian_ubuntu() {
    let raw = "mariadb from 11.8.6-MariaDB, client 15.2 for debian-linux-gnu (x86_64) using EditLine wrapper";
    let parsed = DatabaseDetector::parse_version(raw);
    assert_eq!(parsed, Some("MariaDB 11.8.6".to_string()));
}

#[test]
fn test_mariadb_version_parsing_fedora_arch() {
    let raw = "mariadbd  Ver 10.11.8-MariaDB for Linux on x86_64 (MariaDB Server)";
    let parsed = DatabaseDetector::parse_version(raw);
    assert_eq!(parsed, Some("MariaDB 10.11.8".to_string()));
}

#[test]
fn test_mysql_version_parsing() {
    let raw = "mysql  Ver 8.0.36-0ubuntu0.24.04.1 for Linux on x86_64 ((Ubuntu))";
    let parsed = DatabaseDetector::parse_version(raw);
    assert_eq!(parsed, Some("MySQL 8.0.36-0ubuntu0.24.04.1".to_string()));
}

#[test]
fn test_empty_version_parsing() {
    assert_eq!(DatabaseDetector::parse_version(""), None);
}
