use crate::app::AppState;
use crate::database::{DatabaseAuth, DatabaseType};
use crate::error::AppResult;
use crate::system::port::PortChecker;
use crate::system::service::ServiceState;
use crate::ui::components::UiComponents;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Orientation, ScrolledWindow, Spinner};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct DashboardPage {
    pub container: ScrolledWindow,
    pub status_badge: Label,
    pub version_val: Label,
    pub port_val: Label,
    pub service_val: Label,
    pub start_btn: Button,
    pub stop_btn: Button,
    pub restart_btn: Button,
    pub action_spinner: Spinner,
    pub pma_status_badge: Label,
    pub pma_action_btn: Button,
    pub pma_open_btn: Button,
    pub conflict_banner: Box,
    pub conflict_label: Label,
    pub root_auth_badge: Label,
}

impl DashboardPage {
    pub fn new(app_state: AppState, toast_cb: Rc<dyn Fn(&str)>) -> Self {
        let root = Box::new(Orientation::Vertical, 16);
        root.set_margin_start(24);
        root.set_margin_end(24);
        root.set_margin_top(20);
        root.set_margin_bottom(20);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&root)
            .build();

        // 1. Conflict Banner (hidden by default)
        let conflict_banner = Box::new(Orientation::Horizontal, 12);
        conflict_banner.add_css_class("card-frame");
        conflict_banner.add_css_class("status-error");
        conflict_banner.set_visible(false);
        let conflict_icon = Label::new(Some("⚠️"));
        let conflict_label = Label::new(None);
        conflict_label.set_wrap(true);
        conflict_label.set_halign(Align::Start);
        conflict_label.set_hexpand(true);
        conflict_banner.append(&conflict_icon);
        conflict_banner.append(&conflict_label);
        root.append(&conflict_banner);

        // 2. Database Server Card
        let db_card = UiComponents::create_card();

        let db_header = Box::new(Orientation::Horizontal, 12);
        let db_title = Label::new(Some("Database Server"));
        db_title.add_css_class("big-title");
        db_title.set_halign(Align::Start);
        db_title.set_hexpand(true);

        let initial_state = *app_state.cached_db_state.lock().unwrap();
        let status_badge = UiComponents::create_status_badge(initial_state);

        db_header.append(&db_title);
        db_header.append(&status_badge);
        db_card.append(&db_header);

        // Metrics
        let db_info = app_state.db_info.lock().unwrap().clone();
        let (ver_row, version_val) = UiComponents::create_metric_row("Versi Database", &db_info.version);
        let (port_row, port_val) = UiComponents::create_metric_row("Port Koneksi", &db_info.port.to_string());
        let (host_row, _) = UiComponents::create_metric_row("Host", &db_info.bind_address);
        let (svc_row, service_val) = UiComponents::create_metric_row("Systemd Service", &db_info.service_name);

        db_card.append(&ver_row);
        db_card.append(&port_row);
        db_card.append(&host_row);
        db_card.append(&svc_row);

        // Action Buttons Row
        let actions_box = Box::new(Orientation::Horizontal, 10);
        actions_box.set_margin_top(8);

        let start_btn = Button::with_label("Start");
        start_btn.add_css_class("suggested-action");
        start_btn.set_hexpand(true);

        let stop_btn = Button::with_label("Stop");
        stop_btn.add_css_class("destructive-action");
        stop_btn.set_hexpand(true);

        let restart_btn = Button::with_label("Restart");
        restart_btn.set_hexpand(true);

        let action_spinner = Spinner::new();
        action_spinner.set_visible(false);

        actions_box.append(&start_btn);
        actions_box.append(&stop_btn);
        actions_box.append(&restart_btn);
        actions_box.append(&action_spinner);
        db_card.append(&actions_box);

        root.append(&db_card);

        // 3. Credentials & Login Info Card (Informasi Akun Default XAMPP Style)
        let cred_card = UiComponents::create_card();
        let cred_header = Box::new(Orientation::Horizontal, 12);
        let cred_title = Label::new(Some("Kredensial phpMyAdmin & Database (Mode XAMPP)"));
        cred_title.add_css_class("big-title");
        cred_title.set_halign(Align::Start);
        cred_title.set_hexpand(true);

        let root_ok = DatabaseAuth::test_root_empty_password();
        let root_auth_badge = Label::new(Some(if root_ok { "● Root Password Kosong Aktif" } else { "● Perlu Konfigurasi Root" }));
        root_auth_badge.add_css_class("status-badge");
        root_auth_badge.add_css_class(if root_ok { "status-running" } else { "status-starting" });

        cred_header.append(&cred_title);
        cred_header.append(&root_auth_badge);
        cred_card.append(&cred_header);

        let (user_row, _) = UiComponents::create_metric_row("Username", "root");
        let (pass_row, _) = UiComponents::create_metric_row("Password", "(kosong / no password)");
        let (host_cred_row, _) = UiComponents::create_metric_row("Host / Server", "localhost / 127.0.0.1");

        cred_card.append(&user_row);
        cred_card.append(&pass_row);
        cred_card.append(&host_cred_row);

        // Button to automatically configure / re-set root password to empty
        let cred_btn_row = Box::new(Orientation::Horizontal, 10);
        cred_btn_row.set_margin_top(8);
        let fix_root_btn = Button::with_label("Atur / Reset Root Password Kosong Otomatis");
        fix_root_btn.add_css_class("suggested-action");
        fix_root_btn.set_hexpand(true);
        cred_btn_row.append(&fix_root_btn);
        cred_card.append(&cred_btn_row);

        root.append(&cred_card);

        // 4. phpMyAdmin Quick Card
        let pma_card = UiComponents::create_card();
        let pma_header = Box::new(Orientation::Horizontal, 12);
        let pma_title = Label::new(Some("phpMyAdmin"));
        pma_title.add_css_class("big-title");
        pma_title.set_halign(Align::Start);
        pma_title.set_hexpand(true);

        let pma_info = app_state.pma_info.lock().unwrap().clone();
        let pma_initial_state = if !pma_info.pma_installed {
            ServiceState::Unknown
        } else if app_state.pma_launcher.is_running(app_state.settings.lock().unwrap().phpmyadmin_port) {
            ServiceState::Running
        } else {
            ServiceState::Stopped
        };

        let pma_status_badge = UiComponents::create_status_badge(pma_initial_state);
        pma_header.append(&pma_title);
        pma_header.append(&pma_status_badge);
        pma_card.append(&pma_header);

        let pma_port = app_state.settings.lock().unwrap().phpmyadmin_port;
        let pma_url = format!("http://127.0.0.1:{}", pma_port);
        let (pma_url_row, _) = UiComponents::create_metric_row("URL Akses Lokal", &pma_url);
        pma_card.append(&pma_url_row);

        let pma_actions_box = Box::new(Orientation::Horizontal, 10);
        pma_actions_box.set_margin_top(8);

        let pma_action_btn = Button::with_label("Start Server");
        pma_action_btn.add_css_class("suggested-action");
        pma_action_btn.set_hexpand(true);

        let pma_open_btn = Button::with_label("Open in Browser");
        pma_open_btn.set_hexpand(true);

        pma_actions_box.append(&pma_action_btn);
        pma_actions_box.append(&pma_open_btn);
        pma_card.append(&pma_actions_box);

        root.append(&pma_card);

        // 5. System Detection Summary
        let detect_card = UiComponents::create_card();
        let detect_title = Label::new(Some("Hasil Deteksi Sistem"));
        detect_title.add_css_class("section-header");
        detect_title.set_halign(Align::Start);
        detect_card.append(&detect_title);

        let has_mariadb = db_info.db_type == DatabaseType::MariaDB;
        let has_mysql = db_info.db_type == DatabaseType::MySQL;
        let has_php = pma_info.php_installed;
        let has_pma = pma_info.pma_installed;

        detect_card.append(&UiComponents::create_detection_badge(
            if has_mariadb { "MariaDB Server Terdeteksi" } else { "MariaDB Tidak Terpasang" },
            has_mariadb,
        ));
        detect_card.append(&UiComponents::create_detection_badge(
            if has_mysql { "MySQL Server Terdeteksi" } else { "MySQL Tidak Terpasang" },
            has_mysql,
        ));
        detect_card.append(&UiComponents::create_detection_badge(
            "systemd Service Manager Terdeteksi",
            true,
        ));
        detect_card.append(&UiComponents::create_detection_badge(
            if has_php { "Runtime PHP CLI Terdeteksi" } else { "Runtime PHP Tidak Ditemukan" },
            has_php,
        ));
        detect_card.append(&UiComponents::create_detection_badge(
            if has_pma { "phpMyAdmin Terpasang" } else { "phpMyAdmin Belum Dipasang" },
            has_pma,
        ));

        root.append(&detect_card);

        // Button handlers with async-channel and glib::MainContext::spawn_local
        let is_busy = Arc::new(AtomicBool::new(false));

        // Start button handler
        {
            let state = app_state.clone();
            let toast = toast_cb.clone();
            let busy = is_busy.clone();
            let sp = action_spinner.clone();
            let st_btn = start_btn.clone();
            let sp_btn = stop_btn.clone();
            let rs_btn = restart_btn.clone();

            start_btn.connect_clicked(move |_| {
                if busy.load(Ordering::SeqCst) {
                    return;
                }
                busy.store(true, Ordering::SeqCst);
                sp.set_visible(true);
                sp.start();
                st_btn.set_sensitive(false);
                sp_btn.set_sensitive(false);
                rs_btn.set_sensitive(false);

                let (sender, receiver) = async_channel::bounded::<AppResult<()>>(1);
                let toast_c = toast.clone();
                let busy_c = busy.clone();
                let sp_c = sp.clone();
                let st_c = st_btn.clone();
                let sp_b = sp_btn.clone();
                let rs_c = rs_btn.clone();

                glib::MainContext::default().spawn_local(async move {
                    if let Ok(res) = receiver.recv().await {
                        busy_c.store(false, Ordering::SeqCst);
                        sp_c.stop();
                        sp_c.set_visible(false);
                        st_c.set_sensitive(true);
                        sp_b.set_sensitive(true);
                        rs_c.set_sensitive(true);

                        match res {
                            Ok(_) => toast_c("Database server berhasil dijalankan."),
                            Err(e) => toast_c(&e.to_string()),
                        }
                    }
                });

                let state_c = state.clone();
                std::thread::spawn(move || {
                    let res = state_c.db_service.lock().unwrap().start();
                    let _ = sender.send_blocking(res);
                });
            });
        }

        // Stop button handler
        {
            let state = app_state.clone();
            let toast = toast_cb.clone();
            let busy = is_busy.clone();
            let sp = action_spinner.clone();
            let st_btn = start_btn.clone();
            let sp_btn = stop_btn.clone();
            let rs_btn = restart_btn.clone();

            stop_btn.connect_clicked(move |_| {
                if busy.load(Ordering::SeqCst) {
                    return;
                }
                busy.store(true, Ordering::SeqCst);
                sp.set_visible(true);
                sp.start();
                st_btn.set_sensitive(false);
                sp_btn.set_sensitive(false);
                rs_btn.set_sensitive(false);

                let (sender, receiver) = async_channel::bounded::<AppResult<()>>(1);
                let toast_c = toast.clone();
                let busy_c = busy.clone();
                let sp_c = sp.clone();
                let st_c = st_btn.clone();
                let sp_b = sp_btn.clone();
                let rs_c = rs_btn.clone();

                glib::MainContext::default().spawn_local(async move {
                    if let Ok(res) = receiver.recv().await {
                        busy_c.store(false, Ordering::SeqCst);
                        sp_c.stop();
                        sp_c.set_visible(false);
                        st_c.set_sensitive(true);
                        sp_b.set_sensitive(true);
                        rs_c.set_sensitive(true);

                        match res {
                            Ok(_) => toast_c("Database server berhasil dihentikan."),
                            Err(e) => toast_c(&e.to_string()),
                        }
                    }
                });

                let state_c = state.clone();
                std::thread::spawn(move || {
                    let res = state_c.db_service.lock().unwrap().stop();
                    let _ = sender.send_blocking(res);
                });
            });
        }

        // Restart button handler
        {
            let state = app_state.clone();
            let toast = toast_cb.clone();
            let busy = is_busy.clone();
            let sp = action_spinner.clone();
            let st_btn = start_btn.clone();
            let sp_btn = stop_btn.clone();
            let rs_btn = restart_btn.clone();

            restart_btn.connect_clicked(move |_| {
                if busy.load(Ordering::SeqCst) {
                    return;
                }
                busy.store(true, Ordering::SeqCst);
                sp.set_visible(true);
                sp.start();
                st_btn.set_sensitive(false);
                sp_btn.set_sensitive(false);
                rs_btn.set_sensitive(false);

                let (sender, receiver) = async_channel::bounded::<AppResult<()>>(1);
                let toast_c = toast.clone();
                let busy_c = busy.clone();
                let sp_c = sp.clone();
                let st_c = st_btn.clone();
                let sp_b = sp_btn.clone();
                let rs_c = rs_btn.clone();

                glib::MainContext::default().spawn_local(async move {
                    if let Ok(res) = receiver.recv().await {
                        busy_c.store(false, Ordering::SeqCst);
                        sp_c.stop();
                        sp_c.set_visible(false);
                        st_c.set_sensitive(true);
                        sp_b.set_sensitive(true);
                        rs_c.set_sensitive(true);

                        match res {
                            Ok(_) => toast_c("Database server berhasil direstart."),
                            Err(e) => toast_c(&e.to_string()),
                        }
                    }
                });

                let state_c = state.clone();
                std::thread::spawn(move || {
                    let res = state_c.db_service.lock().unwrap().restart();
                    let _ = sender.send_blocking(res);
                });
            });
        }

        // Fix / Reset Root button handler
        {
            let toast = toast_cb.clone();
            let root_badge = root_auth_badge.clone();
            let btn = fix_root_btn.clone();

            fix_root_btn.connect_clicked(move |_| {
                btn.set_sensitive(false);
                let (sender, receiver) = async_channel::bounded::<AppResult<()>>(1);
                let toast_c = toast.clone();
                let badge_c = root_badge.clone();
                let btn_c = btn.clone();

                glib::MainContext::default().spawn_local(async move {
                    if let Ok(res) = receiver.recv().await {
                        btn_c.set_sensitive(true);
                        match res {
                            Ok(_) => {
                                badge_c.set_label("● Root Password Kosong Aktif");
                                badge_c.remove_css_class("status-starting");
                                badge_c.add_css_class("status-running");
                                toast_c("Berhasil! User 'root' telah diatur dengan password kosong.");
                            }
                            Err(e) => toast_c(&format!("Gagal mengatur root: {}", e)),
                        }
                    }
                });

                std::thread::spawn(move || {
                    let res = DatabaseAuth::setup_root_empty_password();
                    let _ = sender.send_blocking(res);
                });
            });
        }

        // phpMyAdmin Action Button (Start / Stop)
        {
            let state = app_state.clone();
            let toast = toast_cb.clone();
            let act_btn = pma_action_btn.clone();

            pma_action_btn.connect_clicked(move |_| {
                let pma_info = state.pma_info.lock().unwrap().clone();
                let settings = state.settings.lock().unwrap().clone();
                let port = settings.phpmyadmin_port;
                let host = settings.phpmyadmin_host.clone();

                if !pma_info.pma_installed {
                    toast("phpMyAdmin belum terpasang. Silakan lakukan setup di tab phpMyAdmin.");
                    return;
                }

                if state.pma_launcher.is_running(port) {
                    let _ = state.pma_launcher.stop();
                    act_btn.set_label("Start Server");
                    toast("Server lokal phpMyAdmin telah dihentikan.");
                } else {
                    let php_bin = settings.custom_php_path.as_deref().unwrap_or("php");
                    let pma_dir = pma_info.pma_path.as_deref().unwrap_or("");
                    match state.pma_launcher.start(php_bin, pma_dir, &host, port) {
                        Ok(_) => {
                            act_btn.set_label("Stop Server");
                            toast("Server lokal phpMyAdmin berhasil dijalankan.");
                        }
                        Err(e) => {
                            toast(&e.to_string());
                        }
                    }
                }
            });
        }

        // phpMyAdmin Open Button
        {
            let state = app_state.clone();
            let toast = toast_cb;
            pma_open_btn.connect_clicked(move |_| {
                let settings = state.settings.lock().unwrap().clone();
                let host = settings.phpmyadmin_host.clone();
                let port = settings.phpmyadmin_port;
                if let Err(e) = crate::phpmyadmin::PhpMyAdminLauncher::open_browser(&host, port) {
                    toast(&e.to_string());
                }
            });
        }

        Self {
            container: scrolled,
            status_badge,
            version_val,
            port_val,
            service_val,
            start_btn,
            stop_btn,
            restart_btn,
            action_spinner,
            pma_status_badge,
            pma_action_btn,
            pma_open_btn,
            conflict_banner,
            conflict_label,
            root_auth_badge,
        }
    }

    /// Update dynamic status on timer tick
    pub fn update_status(&self, app_state: &AppState) {
        let current_state = app_state.db_service.lock().unwrap().get_status();
        *app_state.cached_db_state.lock().unwrap() = current_state;

        // Update badge
        self.status_badge.set_label(&format!("● {}", current_state.as_str()));
        self.status_badge.remove_css_class("status-running");
        self.status_badge.remove_css_class("status-stopped");
        self.status_badge.remove_css_class("status-starting");
        self.status_badge.remove_css_class("status-stopping");
        self.status_badge.remove_css_class("status-error");
        self.status_badge.remove_css_class("status-unknown");
        self.status_badge.add_css_class(current_state.badge_class());

        // Update button states
        match current_state {
            ServiceState::Running => {
                self.start_btn.set_sensitive(false);
                self.stop_btn.set_sensitive(true);
                self.restart_btn.set_sensitive(true);
            }
            ServiceState::Stopped => {
                self.start_btn.set_sensitive(true);
                self.stop_btn.set_sensitive(false);
                self.restart_btn.set_sensitive(false);
            }
            _ => {
                self.start_btn.set_sensitive(true);
                self.stop_btn.set_sensitive(true);
                self.restart_btn.set_sensitive(true);
            }
        }

        // Check root empty password status
        if current_state == ServiceState::Running {
            let root_ok = DatabaseAuth::test_root_empty_password();
            if root_ok {
                self.root_auth_badge.set_label("● Root Password Kosong Aktif");
                self.root_auth_badge.remove_css_class("status-starting");
                self.root_auth_badge.add_css_class("status-running");
            } else {
                self.root_auth_badge.set_label("● Perlu Konfigurasi Root");
                self.root_auth_badge.remove_css_class("status-running");
                self.root_auth_badge.add_css_class("status-starting");
            }
        }

        // Port conflict verification
        let db_port = app_state.db_info.lock().unwrap().port;
        if let Err(e) = PortChecker::check_conflict(db_port, current_state == ServiceState::Running) {
            self.conflict_label.set_label(&e.to_string());
            self.conflict_banner.set_visible(true);
        } else {
            self.conflict_banner.set_visible(false);
        }

        // Update phpMyAdmin status
        let pma_port = app_state.settings.lock().unwrap().phpmyadmin_port;
        let pma_running = app_state.pma_launcher.is_running(pma_port);
        let pma_installed = app_state.pma_info.lock().unwrap().pma_installed;

        let pma_state = if !pma_installed {
            ServiceState::Unknown
        } else if pma_running {
            ServiceState::Running
        } else {
            ServiceState::Stopped
        };

        self.pma_status_badge.set_label(&format!("● {}", pma_state.as_str()));
        self.pma_status_badge.remove_css_class("status-running");
        self.pma_status_badge.remove_css_class("status-stopped");
        self.pma_status_badge.remove_css_class("status-unknown");
        self.pma_status_badge.add_css_class(pma_state.badge_class());

        if pma_running {
            self.pma_action_btn.set_label("Stop Server");
            self.pma_open_btn.set_sensitive(true);
        } else {
            self.pma_action_btn.set_label("Start Server");
            self.pma_open_btn.set_sensitive(false);
        }
    }
}
