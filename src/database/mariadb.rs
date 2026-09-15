use std::path::Path;

pub struct MariaDbInfo;

impl MariaDbInfo {
    pub const DEFAULT_PORT: u16 = 3306;
    pub const DEFAULT_SERVICE: &'static str = "mariadb.service";
    pub const CONFIG_PATHS: &'static [&'static str] = &[
        "/etc/mysql/mariadb.cnf",
        "/etc/mysql/my.cnf",
        "/etc/my.cnf.d/mariadb-server.cnf",
        "/etc/my.cnf",
    ];

    pub fn primary_config_path() -> Option<&'static str> {
        Self::CONFIG_PATHS
            .iter()
            .copied()
            .find(|p| Path::new(p).exists())
    }
}
