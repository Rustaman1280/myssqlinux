#![allow(unused_imports, dead_code)]

mod app;
mod config;
mod database;
mod error;
mod phpmyadmin;
mod system;
mod ui;

use adw::prelude::*;
use adw::{Application, ColorScheme, StyleManager};
use app::AppState;
use gtk4::gdk::Display;
use gtk4::CssProvider;
use tracing::info;
use ui::MainWindow;

const APP_ID: &str = "org.mysqldesk.MySQLDesk";
const CUSTOM_CSS: &str = include_str!("../data/style.css");

fn main() -> glib::ExitCode {
    // Initialize tracing logger
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mysqldesk=info,warn,error".into()),
        )
        .init();

    info!("Starting MySQLDesk - Native Linux MySQL/MariaDB & phpMyAdmin Manager");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_startup(|_| {
        load_custom_css();
    });

    app.connect_activate(|app| {
        let app_state = AppState::new();

        // Apply theme preference from saved settings
        let theme_pref = app_state.settings.lock().unwrap().theme.clone();
        let style_manager = StyleManager::default();
        match theme_pref.as_str() {
            "dark" => style_manager.set_color_scheme(ColorScheme::ForceDark),
            "light" => style_manager.set_color_scheme(ColorScheme::ForceLight),
            _ => style_manager.set_color_scheme(ColorScheme::Default),
        }

        let main_window = MainWindow::new(app, app_state);
        main_window.present();
    });

    app.run()
}

fn load_custom_css() {
    let provider = CssProvider::new();
    provider.load_from_data(CUSTOM_CSS);

    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        info!("Custom CSS styling loaded successfully.");
    }
}
