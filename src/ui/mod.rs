pub mod components;
pub mod dashboard;
pub mod logs;
pub mod phpmyadmin;
pub mod server;
pub mod settings;
pub mod window;

pub use components::UiComponents;
pub use dashboard::DashboardPage;
pub use logs::LogsPage;
pub use phpmyadmin::PhpMyAdminPage;
pub use server::ServerPage;
pub use settings::SettingsPage;
pub use window::MainWindow;
