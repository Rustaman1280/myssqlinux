use crate::app::AppState;
use crate::ui::components::UiComponents;
use adw::ColorScheme;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, DropDown, Entry, Label, Orientation, ScrolledWindow, SpinButton};
use std::rc::Rc;

pub struct SettingsPage {
    pub container: ScrolledWindow,
}

impl SettingsPage {
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

        let settings = app_state.settings.lock().unwrap().clone();

        // 1. Appearance / Theme Card
        let theme_card = UiComponents::create_card();
        let theme_title = Label::new(Some("Tampilan & Antarmuka"));
        theme_title.add_css_class("big-title");
        theme_title.set_halign(Align::Start);
        theme_card.append(&theme_title);

        let theme_row = Box::new(Orientation::Horizontal, 12);
        let theme_lbl = Label::new(Some("Pilihan Tema"));
        theme_lbl.set_halign(Align::Start);
        theme_lbl.set_hexpand(true);

        let theme_options = gtk4::StringList::new(&["System Default", "Dark Mode", "Light Mode"]);
        let theme_dropdown = DropDown::new(Some(theme_options), None::<gtk4::Expression>);

        let initial_theme_idx = match settings.theme.as_str() {
            "dark" => 1,
            "light" => 2,
            _ => 0,
        };
        theme_dropdown.set_selected(initial_theme_idx);

        theme_row.append(&theme_lbl);
        theme_row.append(&theme_dropdown);
        theme_card.append(&theme_row);
        root.append(&theme_card);

        // 2. Database & Port Card
        let net_card = UiComponents::create_card();
        let net_title = Label::new(Some("Port & Jaringan"));
        net_title.add_css_class("big-title");
        net_title.set_halign(Align::Start);
        net_card.append(&net_title);

        // DB Port Row
        let db_port_row = Box::new(Orientation::Horizontal, 12);
        let db_port_lbl = Label::new(Some("Port Database Server (MySQL/MariaDB)"));
        db_port_lbl.set_halign(Align::Start);
        db_port_lbl.set_hexpand(true);
        let db_port_spin = SpinButton::with_range(1.0, 65535.0, 1.0);
        db_port_spin.set_value(settings.db_port as f64);
        db_port_row.append(&db_port_lbl);
        db_port_row.append(&db_port_spin);
        net_card.append(&db_port_row);

        // PMA Port Row
        let pma_port_row = Box::new(Orientation::Horizontal, 12);
        let pma_port_lbl = Label::new(Some("Port phpMyAdmin Lokal"));
        pma_port_lbl.set_halign(Align::Start);
        pma_port_lbl.set_hexpand(true);
        let pma_port_spin = SpinButton::with_range(1024.0, 65535.0, 1.0);
        pma_port_spin.set_value(settings.phpmyadmin_port as f64);
        pma_port_row.append(&pma_port_lbl);
        pma_port_row.append(&pma_port_spin);
        net_card.append(&pma_port_row);

        // PMA Host Row
        let pma_host_row = Box::new(Orientation::Horizontal, 12);
        let pma_host_lbl = Label::new(Some("Host Binding phpMyAdmin (Aman: 127.0.0.1)"));
        pma_host_lbl.set_halign(Align::Start);
        pma_host_lbl.set_hexpand(true);
        let pma_host_entry = Entry::new();
        pma_host_entry.set_text(&settings.phpmyadmin_host);
        pma_host_row.append(&pma_host_lbl);
        pma_host_row.append(&pma_host_entry);
        net_card.append(&pma_host_row);

        root.append(&net_card);

        // 3. Application Preferences Card
        let app_card = UiComponents::create_card();
        let app_title = Label::new(Some("Preferensi Aplikasi"));
        app_title.add_css_class("big-title");
        app_title.set_halign(Align::Start);
        app_card.append(&app_title);

        let refresh_row = Box::new(Orientation::Horizontal, 12);
        let refresh_lbl = Label::new(Some("Interval Auto-Refresh Status (detik)"));
        refresh_lbl.set_halign(Align::Start);
        refresh_lbl.set_hexpand(true);
        let refresh_spin = SpinButton::with_range(1.0, 60.0, 1.0);
        refresh_spin.set_value(settings.auto_refresh_interval_secs as f64);
        refresh_row.append(&refresh_lbl);
        refresh_row.append(&refresh_spin);
        app_card.append(&refresh_row);

        // Custom PHP Path
        let php_path_row = Box::new(Orientation::Horizontal, 12);
        let php_path_lbl = Label::new(Some("Jalur Kustom Binary PHP (opsional)"));
        php_path_lbl.set_halign(Align::Start);
        php_path_lbl.set_hexpand(true);
        let php_path_entry = Entry::new();
        if let Some(ref p) = settings.custom_php_path {
            php_path_entry.set_text(p);
        }
        php_path_entry.set_placeholder_text(Some("/usr/bin/php"));
        php_path_row.append(&php_path_lbl);
        php_path_row.append(&php_path_entry);
        app_card.append(&php_path_row);

        root.append(&app_card);

        // Save Button
        let save_btn = Button::with_label("Simpan Pengaturan");
        save_btn.add_css_class("suggested-action");
        save_btn.set_halign(Align::End);

        {
            let state = app_state.clone();
            let toast = toast_cb.clone();
            let th_dd = theme_dropdown.clone();
            let db_spin = db_port_spin.clone();
            let pma_spin = pma_port_spin.clone();
            let host_ent = pma_host_entry.clone();
            let ref_spin = refresh_spin.clone();
            let php_ent = php_path_entry.clone();

            save_btn.connect_clicked(move |_| {
                let theme_str = match th_dd.selected() {
                    1 => "dark",
                    2 => "light",
                    _ => "system",
                };

                // Apply theme immediately
                let style_manager = adw::StyleManager::default();
                match theme_str {
                    "dark" => style_manager.set_color_scheme(ColorScheme::ForceDark),
                    "light" => style_manager.set_color_scheme(ColorScheme::ForceLight),
                    _ => style_manager.set_color_scheme(ColorScheme::Default),
                }

                let mut current = state.settings.lock().unwrap();
                current.theme = theme_str.to_string();
                current.db_port = db_spin.value() as u16;
                current.phpmyadmin_port = pma_spin.value() as u16;
                current.phpmyadmin_host = host_ent.text().to_string();
                current.auto_refresh_interval_secs = ref_spin.value() as u64;

                let php_val = php_ent.text().to_string();
                current.custom_php_path = if php_val.trim().is_empty() {
                    None
                } else {
                    Some(php_val.trim().to_string())
                };

                match current.save() {
                    Ok(_) => {
                        toast("Pengaturan berhasil disimpan.");
                    }
                    Err(e) => {
                        toast(&format!("Gagal menyimpan pengaturan: {}", e));
                    }
                }
            });
        }

        root.append(&save_btn);

        Self { container: scrolled }
    }
}
