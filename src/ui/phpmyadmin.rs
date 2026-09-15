use crate::app::AppState;
use crate::database::DatabaseAuth;
use crate::error::AppResult;
use crate::phpmyadmin::{PhpMyAdminDetector, PhpMyAdminLauncher, PhpMyAdminManager};
use crate::system::service::ServiceState;
use crate::ui::components::UiComponents;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Orientation, ProgressBar, ScrolledWindow};
use std::rc::Rc;

pub struct PhpMyAdminPage {
    pub container: ScrolledWindow,
}

impl PhpMyAdminPage {
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

        let pma_info = app_state.pma_info.lock().unwrap().clone();
        let settings = app_state.settings.lock().unwrap().clone();

        // 1. Status Card
        let status_card = UiComponents::create_card();
        let header = Box::new(Orientation::Horizontal, 12);
        let title = Label::new(Some("phpMyAdmin Manager"));
        title.add_css_class("big-title");
        title.set_halign(Align::Start);
        title.set_hexpand(true);

        let is_running = app_state.pma_launcher.is_running(settings.phpmyadmin_port);
        let pma_state = if !pma_info.pma_installed {
            ServiceState::Unknown
        } else if is_running {
            ServiceState::Running
        } else {
            ServiceState::Stopped
        };

        let status_badge = UiComponents::create_status_badge(pma_state);
        header.append(&title);
        header.append(&status_badge);
        status_card.append(&header);

        let pma_status_text = if pma_info.pma_installed {
            "Terpasang dan siap digunakan"
        } else {
            "Belum terpasang di sistem"
        };
        let (inst_row, _) = UiComponents::create_metric_row("Status Instalasi", pma_status_text);
        let (php_row, _) = UiComponents::create_metric_row("Versi PHP", &pma_info.php_version);
        let pma_ver_text = pma_info.pma_version.unwrap_or_else(|| "N/A".to_string());
        let (pma_ver_row, _) = UiComponents::create_metric_row("Versi phpMyAdmin", &pma_ver_text);
        let pma_path_text = pma_info.pma_path.unwrap_or_else(|| "Belum ada direktori".to_string());
        let (path_row, _) = UiComponents::create_metric_row("Lokasi Berkas", &pma_path_text);
        let pma_url = format!("http://{}:{}", settings.phpmyadmin_host, settings.phpmyadmin_port);
        let (url_row, _) = UiComponents::create_metric_row("URL Server Lokal", &pma_url);

        status_card.append(&inst_row);
        status_card.append(&php_row);
        status_card.append(&pma_ver_row);
        status_card.append(&path_row);
        status_card.append(&url_row);

        // Control Buttons
        let actions_box = Box::new(Orientation::Horizontal, 10);
        actions_box.set_margin_top(8);

        let toggle_btn = Button::with_label(if is_running { "Stop Server" } else { "Start Server" });
        toggle_btn.add_css_class("suggested-action");
        toggle_btn.set_hexpand(true);

        let open_btn = Button::with_label("Open in Browser");
        open_btn.set_hexpand(true);
        open_btn.set_sensitive(is_running);

        actions_box.append(&toggle_btn);
        actions_box.append(&open_btn);
        status_card.append(&actions_box);

        root.append(&status_card);

        // 2. Credentials Card (XAMPP Mode: User root, password kosong)
        let cred_card = UiComponents::create_card();
        let cred_title = Label::new(Some("Informasi Login phpMyAdmin (XAMPP Mode)"));
        cred_title.add_css_class("big-title");
        cred_title.set_halign(Align::Start);
        cred_card.append(&cred_title);

        let (user_row, _) = UiComponents::create_metric_row("Username Default", "root");
        let (pass_row, _) = UiComponents::create_metric_row("Password Default", "(kosong / no password)");
        let (server_row, _) = UiComponents::create_metric_row("Server", "127.0.0.1 / localhost");

        cred_card.append(&user_row);
        cred_card.append(&pass_row);
        cred_card.append(&server_row);

        let fix_root_btn = Button::with_label("Atur Ulang User 'root' ke Password Kosong");
        fix_root_btn.set_margin_top(8);
        fix_root_btn.add_css_class("suggested-action");
        {
            let toast = toast_cb.clone();
            let btn = fix_root_btn.clone();
            fix_root_btn.connect_clicked(move |_| {
                btn.set_sensitive(false);
                let (sender, receiver) = async_channel::bounded::<AppResult<()>>(1);
                let toast_c = toast.clone();
                let btn_c = btn.clone();

                glib::MainContext::default().spawn_local(async move {
                    if let Ok(res) = receiver.recv().await {
                        btn_c.set_sensitive(true);
                        match res {
                            Ok(_) => toast_c("Berhasil! Akun 'root' sekarang dapat login tanpa password."),
                            Err(e) => toast_c(&format!("Gagal: {}", e)),
                        }
                    }
                });

                std::thread::spawn(move || {
                    let res = DatabaseAuth::setup_root_empty_password();
                    let _ = sender.send_blocking(res);
                });
            });
        }
        cred_card.append(&fix_root_btn);
        root.append(&cred_card);

        // 3. Setup / Install Card
        let setup_card = UiComponents::create_card();
        let setup_title = Label::new(Some("Pemasangan phpMyAdmin Mandiri"));
        setup_title.add_css_class("section-header");
        setup_title.set_halign(Align::Start);
        setup_card.append(&setup_title);

        let desc_lbl = Label::new(Some(
            "Jika phpMyAdmin belum terpasang atau Anda ingin menggunakan versi terisolasi, klik tombol di bawah untuk mengunduh rilis resmi phpMyAdmin dengan verifikasi checksum SHA256 yang aman ke direktori ~/.local/share/mysqldesk/phpmyadmin."
        ));
        desc_lbl.set_wrap(true);
        desc_lbl.set_halign(Align::Start);
        setup_card.append(&desc_lbl);

        let progress_box = Box::new(Orientation::Vertical, 6);
        progress_box.set_margin_top(8);
        progress_box.set_visible(false);

        let progress_lbl = Label::new(Some("Mengunduh..."));
        progress_lbl.set_halign(Align::Start);
        let progress_bar = ProgressBar::new();
        progress_bar.set_pulse_step(0.1);

        progress_box.append(&progress_lbl);
        progress_box.append(&progress_bar);
        setup_card.append(&progress_box);

        let setup_btn = Button::with_label("Setup / Download phpMyAdmin Resmi");
        setup_btn.set_margin_top(8);
        setup_card.append(&setup_btn);

        root.append(&setup_card);

        // Wire Toggle Server Button
        {
            let state = app_state.clone();
            let toast = toast_cb.clone();
            let btn = toggle_btn.clone();
            let op_btn = open_btn.clone();
            let b_badge = status_badge.clone();

            toggle_btn.connect_clicked(move |_| {
                let settings = state.settings.lock().unwrap().clone();
                let pma_info = state.pma_info.lock().unwrap().clone();
                let port = settings.phpmyadmin_port;
                let host = settings.phpmyadmin_host.clone();

                if !pma_info.pma_installed {
                    toast("phpMyAdmin belum terpasang. Silakan lakukan setup terlebih dahulu.");
                    return;
                }

                if state.pma_launcher.is_running(port) {
                    let _ = state.pma_launcher.stop();
                    btn.set_label("Start Server");
                    op_btn.set_sensitive(false);
                    b_badge.set_label("● Stopped");
                    b_badge.remove_css_class("status-running");
                    b_badge.add_css_class("status-stopped");
                    toast("Server lokal phpMyAdmin telah dihentikan.");
                } else {
                    let php_bin = settings.custom_php_path.as_deref().unwrap_or("php");
                    let pma_dir = pma_info.pma_path.as_deref().unwrap_or("");
                    match state.pma_launcher.start(php_bin, pma_dir, &host, port) {
                        Ok(_) => {
                            btn.set_label("Stop Server");
                            op_btn.set_sensitive(true);
                            b_badge.set_label("● Running");
                            b_badge.remove_css_class("status-stopped");
                            b_badge.add_css_class("status-running");
                            toast("Server lokal phpMyAdmin berhasil dijalankan.");
                        }
                        Err(e) => {
                            toast(&e.to_string());
                        }
                    }
                }
            });
        }

        // Wire Open Browser Button
        {
            let state = app_state.clone();
            let toast = toast_cb.clone();
            open_btn.connect_clicked(move |_| {
                let settings = state.settings.lock().unwrap().clone();
                if let Err(e) = PhpMyAdminLauncher::open_browser(&settings.phpmyadmin_host, settings.phpmyadmin_port) {
                    toast(&e.to_string());
                }
            });
        }

        // Wire Setup phpMyAdmin Button with async-channel
        {
            let state = app_state;
            let toast = toast_cb;
            let s_btn = setup_btn.clone();
            let p_box = progress_box;
            let p_lbl = progress_lbl.clone();

            setup_btn.connect_clicked(move |_| {
                s_btn.set_sensitive(false);
                p_box.set_visible(true);
                p_lbl.set_label("Memulai pengunduhan phpMyAdmin...");

                let (prog_sender, prog_receiver) = async_channel::unbounded::<String>();
                let (done_sender, done_receiver) = async_channel::bounded::<AppResult<()>>(1);

                let p_lbl_c = p_lbl.clone();
                let s_btn_c = s_btn.clone();
                let p_box_c = p_box.clone();
                let toast_c = toast.clone();
                let state_c = state.clone();

                glib::MainContext::default().spawn_local(async move {
                    while let Ok(msg) = prog_receiver.recv().await {
                        p_lbl_c.set_label(&msg);
                    }
                });

                glib::MainContext::default().spawn_local(async move {
                    if let Ok(res) = done_receiver.recv().await {
                        s_btn_c.set_sensitive(true);
                        p_box_c.set_visible(false);

                        match res {
                            Ok(_) => {
                                state_c.refresh_all();
                                toast_c("Setup phpMyAdmin berhasil! Anda sekarang dapat menjalankan server.");
                            }
                            Err(e) => {
                                toast_c(&format!("Setup gagal: {}", e));
                            }
                        }
                    }
                });

                std::thread::spawn(move || {
                    let target_dir = PhpMyAdminDetector::default_local_pma_dir();
                    let sender_for_progress = prog_sender.clone();
                    let progress_cb = move |msg: &str| {
                        let _ = sender_for_progress.send_blocking(msg.to_string());
                    };

                    let res = PhpMyAdminManager::setup_phpmyadmin(&target_dir, progress_cb);
                    let _ = done_sender.send_blocking(res);
                });
            });
        }

        Self { container: scrolled }
    }
}
