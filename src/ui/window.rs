use crate::app::AppState;
use crate::ui::dashboard::DashboardPage;
use crate::ui::logs::LogsPage;
use crate::ui::phpmyadmin::PhpMyAdminPage;
use crate::ui::server::ServerPage;
use crate::ui::settings::SettingsPage;
use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar, Toast, ToastOverlay, ViewStack, ViewSwitcher};
use gtk4::{Box, Orientation};
use std::rc::Rc;
use tracing::info;

pub struct MainWindow {
    pub window: ApplicationWindow,
}

impl MainWindow {
    pub fn new(app: &Application, app_state: AppState) -> Self {
        info!("Creating MySQLDesk main window...");

        let window = ApplicationWindow::builder()
            .application(app)
            .title("MySQLDesk")
            .default_width(960)
            .default_height(680)
            .build();

        let toast_overlay = ToastOverlay::new();

        // Toast sender callback (single-threaded GTK callback)
        let overlay_c = toast_overlay.clone();
        let toast_cb: Rc<dyn Fn(&str)> = Rc::new(move |msg: &str| {
            let toast = Toast::new(msg);
            toast.set_timeout(3);
            overlay_c.add_toast(toast);
        });

        let main_box = Box::new(Orientation::Vertical, 0);

        // HeaderBar with ViewSwitcher
        let header_bar = HeaderBar::new();
        let view_stack = ViewStack::new();

        let view_switcher = ViewSwitcher::builder()
            .stack(&view_stack)
            .policy(adw::ViewSwitcherPolicy::Wide)
            .build();

        header_bar.set_title_widget(Some(&view_switcher));
        main_box.append(&header_bar);

        // 1. Dashboard Page
        let dashboard_page = Rc::new(DashboardPage::new(app_state.clone(), toast_cb.clone()));
        let dash_page = view_stack.add_titled(&dashboard_page.container, Some("dashboard"), "Dashboard");
        dash_page.set_icon_name(Some("view-grid-symbolic"));

        // 2. Server Page
        let server_page = ServerPage::new(app_state.clone(), toast_cb.clone());
        let srv_page = view_stack.add_titled(&server_page.container, Some("server"), "Server");
        srv_page.set_icon_name(Some("drive-harddisk-symbolic"));

        // 3. phpMyAdmin Page
        let pma_page = PhpMyAdminPage::new(app_state.clone(), toast_cb.clone());
        let pma_stack_page = view_stack.add_titled(&pma_page.container, Some("phpmyadmin"), "phpMyAdmin");
        pma_stack_page.set_icon_name(Some("network-server-symbolic"));

        // 4. Logs Page
        let logs_page = LogsPage::new(app_state.clone(), toast_cb.clone());
        let logs_stack_page = view_stack.add_titled(&logs_page.container, Some("logs"), "Logs");
        logs_stack_page.set_icon_name(Some("text-x-generic-symbolic"));

        // 5. Settings Page
        let settings_page = SettingsPage::new(app_state.clone(), toast_cb);
        let set_stack_page = view_stack.add_titled(&settings_page.container, Some("settings"), "Settings");
        set_stack_page.set_icon_name(Some("emblem-system-symbolic"));

        view_stack.set_vexpand(true);
        main_box.append(&view_stack);

        toast_overlay.set_child(Some(&main_box));
        window.set_content(Some(&toast_overlay));

        // Setup periodic auto-refresh for Dashboard
        let interval_secs = app_state.settings.lock().unwrap().auto_refresh_interval_secs.max(1);
        let state_ticker = app_state;
        let dash_ticker = dashboard_page;

        glib::timeout_add_seconds_local(interval_secs as u32, move || {
            dash_ticker.update_status(&state_ticker);
            glib::ControlFlow::Continue
        });

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
