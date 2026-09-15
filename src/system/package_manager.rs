use crate::system::process::SafeCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxDistroFamily {
    DebianUbuntu,
    FedoraRhel,
    Arch,
    Unknown,
}

pub trait PackageManager: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    fn mariadb_package_name(&self) -> &'static str;
    fn mysql_package_name(&self) -> &'static str;
    fn php_package_names(&self) -> Vec<&'static str>;
    fn install_command(&self, packages: &[&str]) -> String;
}

pub struct AptPackageManager;
impl PackageManager for AptPackageManager {
    fn name(&self) -> &'static str {
        "apt"
    }
    fn is_available(&self) -> bool {
        SafeCommand::binary_exists("apt") || SafeCommand::binary_exists("apt-get")
    }
    fn mariadb_package_name(&self) -> &'static str {
        "mariadb-server"
    }
    fn mysql_package_name(&self) -> &'static str {
        "mysql-server"
    }
    fn php_package_names(&self) -> Vec<&'static str> {
        vec!["php-cli", "php-mysqli", "php-mbstring", "php-zip"]
    }
    fn install_command(&self, packages: &[&str]) -> String {
        format!("sudo apt update && sudo apt install -y {}", packages.join(" "))
    }
}

pub struct DnfPackageManager;
impl PackageManager for DnfPackageManager {
    fn name(&self) -> &'static str {
        "dnf"
    }
    fn is_available(&self) -> bool {
        SafeCommand::binary_exists("dnf")
    }
    fn mariadb_package_name(&self) -> &'static str {
        "mariadb-server"
    }
    fn mysql_package_name(&self) -> &'static str {
        "community-mysql-server"
    }
    fn php_package_names(&self) -> Vec<&'static str> {
        vec!["php-cli", "php-mysqli", "php-mbstring", "php-zip"]
    }
    fn install_command(&self, packages: &[&str]) -> String {
        format!("sudo dnf install -y {}", packages.join(" "))
    }
}

pub struct PacmanPackageManager;
impl PackageManager for PacmanPackageManager {
    fn name(&self) -> &'static str {
        "pacman"
    }
    fn is_available(&self) -> bool {
        SafeCommand::binary_exists("pacman")
    }
    fn mariadb_package_name(&self) -> &'static str {
        "mariadb"
    }
    fn mysql_package_name(&self) -> &'static str {
        "mysql"
    }
    fn php_package_names(&self) -> Vec<&'static str> {
        vec!["php", "php-embed"]
    }
    fn install_command(&self, packages: &[&str]) -> String {
        format!("sudo pacman -S --noconfirm {}", packages.join(" "))
    }
}

pub fn detect_package_manager() -> Box<dyn PackageManager> {
    if SafeCommand::binary_exists("apt") {
        Box::new(AptPackageManager)
    } else if SafeCommand::binary_exists("dnf") {
        Box::new(DnfPackageManager)
    } else if SafeCommand::binary_exists("pacman") {
        Box::new(PacmanPackageManager)
    } else {
        Box::new(AptPackageManager)
    }
}
