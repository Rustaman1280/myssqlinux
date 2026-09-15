use std::path::Path;

pub struct MySqlInfo;

impl MySqlInfo {
    pub const DEFAULT_PORT: u16 = 3306;
    pub const DEFAULT_SERVICE: &'static str = "mysql.service";
    pub const CONFIG_PATHS: &'static [&'static str] = &[
        "/etc/mysql/my.cnf",
        "/etc/mysql/mysql.cnf",
        "/etc/my.cnf.d/mysql-server.cnf",
        "/etc/my.cnf",
    ];

    pub fn primary_config_path() -> Option<&'static str> {
        Self::CONFIG_PATHS
            .iter()
            .copied()
            .find(|p| Path::new(p).exists())
    }
}
