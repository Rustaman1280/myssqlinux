use crate::app::AppState;
use crate::database::DatabaseConfig;
use crate::ui::components::UiComponents;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Orientation, ScrolledWindow};
use std::rc::Rc;

pub struct ServerPage {
    pub container: ScrolledWindow,
}

impl ServerPage {
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

        let db_info = app_state.db_info.lock().unwrap().clone();

        // Server Information Card
        let info_card = UiComponents::create_card();

        let header = Box::new(Orientation::Horizontal, 12);
        let title = Label::new(Some("Informasi Server & Konfigurasi"));
        title.add_css_class("big-title");
        title.set_halign(Align::Start);
        title.set_hexpand(true);
        header.append(&title);
        info_card.append(&header);

        let (type_row, _) = UiComponents::create_metric_row("Engine Database", db_info.db_type.as_str());
        let (ver_row, _) = UiComponents::create_metric_row("Versi Server", &db_info.version);
        let (exec_row, _) = UiComponents::create_metric_row("Executable Path", &db_info.executable_path);
        let (datadir_row, _) = UiComponents::create_metric_row("Data Directory", &db_info.data_dir);
        let (socket_row, _) = UiComponents::create_metric_row("UNIX Socket", &db_info.socket_path);
        let (port_row, _) = UiComponents::create_metric_row("Port Database", &db_info.port.to_string());
        let (bind_row, _) = UiComponents::create_metric_row("Bind Address", &db_info.bind_address);
        let (config_row, _) = UiComponents::create_metric_row("Berkas Konfigurasi", &db_info.config_file);

        info_card.append(&type_row);
        info_card.append(&ver_row);
        info_card.append(&exec_row);
        info_card.append(&datadir_row);
        info_card.append(&socket_row);
        info_card.append(&port_row);
        info_card.append(&bind_row);
        info_card.append(&config_row);

        // Open config button
        let config_btn = Button::with_label("Buka Berkas Konfigurasi di Text Editor");
        config_btn.add_css_class("suggested-action");
        config_btn.set_margin_top(8);
        {
            let cfg_path = db_info.config_file.clone();
            let toast = toast_cb.clone();
            config_btn.connect_clicked(move |_| {
                if let Err(e) = DatabaseConfig::open_in_editor(&cfg_path) {
                    toast(&format!("Gagal membuka editor: {}", e));
                }
            });
        }
        info_card.append(&config_btn);
        root.append(&info_card);

        // Connection Guide Card
        let guide_card = UiComponents::create_card();
        let guide_title = Label::new(Some("Panduan Koneksi Lokal"));
        guide_title.add_css_class("section-header");
        guide_title.set_halign(Align::Start);
        guide_card.append(&guide_title);

        let cli_help = format!(
            "Koneksi via Terminal CLI:\n  mariadb -h 127.0.0.1 -P {} -u root -p\natau via UNIX socket:\n  mariadb -S {}\n\nURL Koneksi Aplikasi (JDBC / PDO):\n  mysql://localhost:{}/your_database",
            db_info.port, db_info.socket_path, db_info.port
        );
        let guide_lbl = Label::new(Some(&cli_help));
        guide_lbl.set_halign(Align::Start);
        guide_lbl.set_selectable(true);
        guide_lbl.add_css_class("log-view");
        guide_card.append(&guide_lbl);

        root.append(&guide_card);

        Self { container: scrolled }
    }
}
