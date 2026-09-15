use mysqldesk::config::AppSettings;

#[test]
fn test_default_settings() {
    let settings = AppSettings::default();
    assert_eq!(settings.db_port, 3306);
    assert_eq!(settings.phpmyadmin_port, 8080);
    assert_eq!(settings.phpmyadmin_host, "127.0.0.1");
    assert_eq!(settings.theme, "system");
    assert_eq!(settings.auto_refresh_interval_secs, 3);
    assert!(settings.custom_php_path.is_none());
    assert!(settings.custom_pma_path.is_none());
}

#[test]
fn test_settings_serialization_roundtrip() {
    let mut settings = AppSettings::default();
    settings.db_port = 3307;
    settings.phpmyadmin_port = 8888;
    settings.theme = "dark".to_string();

    let json = serde_json::to_string(&settings).unwrap();
    let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.db_port, 3307);
    assert_eq!(deserialized.phpmyadmin_port, 8888);
    assert_eq!(deserialized.theme, "dark");
}
