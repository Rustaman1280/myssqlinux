use crate::app::AppState;
use crate::system::service::SystemdManager;
use gtk4::prelude::*;
use gtk4::{
    Box, Button, CheckButton, Entry, Orientation, PolicyType, ScrolledWindow, TextView,
};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct LogsPage {
    pub container: Box,
    pub raw_logs: Arc<std::sync::Mutex<String>>,
}

impl LogsPage {
    pub fn new(app_state: AppState, toast_cb: Rc<dyn Fn(&str)>) -> Self {
        let root = Box::new(Orientation::Vertical, 12);
        root.set_margin_start(16);
        root.set_margin_end(16);
        root.set_margin_top(16);
        root.set_margin_bottom(16);

        // Header and Toolbar
        let toolbar = Box::new(Orientation::Horizontal, 10);

        let search_entry = Entry::new();
        search_entry.set_placeholder_text(Some("Cari / filter kata kunci dalam log..."));
        search_entry.set_hexpand(true);

        let refresh_btn = Button::with_label("Refresh");
        let copy_btn = Button::with_label("Copy");
        let clear_btn = Button::with_label("Clear");

        let auto_scroll_check = CheckButton::with_label("Auto-scroll");
        auto_scroll_check.set_active(true);

        let auto_refresh_check = CheckButton::with_label("Auto-refresh");
        auto_refresh_check.set_active(true);

        toolbar.append(&search_entry);
        toolbar.append(&refresh_btn);
        toolbar.append(&copy_btn);
        toolbar.append(&clear_btn);
        toolbar.append(&auto_scroll_check);
        toolbar.append(&auto_refresh_check);

        root.append(&toolbar);

        // Text view for logs
        let text_view = TextView::new();
        text_view.set_editable(false);
        text_view.set_cursor_visible(false);
        text_view.set_monospace(true);
        text_view.set_wrap_mode(gtk4::WrapMode::WordChar);
        text_view.add_css_class("log-view");

        let buffer = text_view.buffer();
        buffer.set_text("Memuat log sistem database...\n");

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Automatic)
            .vscrollbar_policy(PolicyType::Automatic)
            .hexpand(true)
            .vexpand(true)
            .child(&text_view)
            .build();
        scrolled.add_css_class("card-frame");

        root.append(&scrolled);

        let raw_logs = Arc::new(std::sync::Mutex::new(String::new()));
        let is_loading = Arc::new(AtomicBool::new(false));

        // Function to refresh logs asynchronously using async-channel and spawn_local
        let fetch_logs = {
            let state = app_state.clone();
            let raw_c = raw_logs.clone();
            let buffer_c = buffer.clone();
            let search_c = search_entry.clone();
            let scrolled_c = scrolled.clone();
            let auto_scroll_c = auto_scroll_check.clone();
            let loading = is_loading.clone();

            Arc::new(move || {
                if loading.load(Ordering::SeqCst) {
                    return;
                }
                loading.store(true, Ordering::SeqCst);

                let svc_name = state.db_info.lock().unwrap().service_name.clone();
                let (sender, receiver) = async_channel::bounded::<String>(1);

                let raw_inner = raw_c.clone();
                let buf_inner = buffer_c.clone();
                let search_inner = search_c.clone();
                let scrolled_inner = scrolled_c.clone();
                let auto_s_inner = auto_scroll_c.clone();
                let loading_inner = loading.clone();

                glib::MainContext::default().spawn_local(async move {
                    if let Ok(log_text) = receiver.recv().await {
                        loading_inner.store(false, Ordering::SeqCst);
                        *raw_inner.lock().unwrap() = log_text.clone();

                        let query = search_inner.text().to_string();
                        let displayed = if query.trim().is_empty() {
                            log_text
                        } else {
                            log_text
                                .lines()
                                .filter(|l| l.to_lowercase().contains(&query.to_lowercase()))
                                .collect::<Vec<&str>>()
                                .join("\n")
                        };

                        buf_inner.set_text(&displayed);

                        if auto_s_inner.is_active() {
                            let vadj = scrolled_inner.vadjustment();
                            vadj.set_value(vadj.upper());
                        }
                    }
                });

                std::thread::spawn(move || {
                    let log_text = SystemdManager::get_journal_logs(&svc_name, 250)
                        .unwrap_or_else(|e| format!("Gagal memuat log: {}", e));
                    let _ = sender.send_blocking(log_text);
                });
            })
        };

        // Wire Refresh Button
        {
            let fetcher = fetch_logs.clone();
            refresh_btn.connect_clicked(move |_| {
                fetcher();
            });
        }

        // Wire Copy Button
        {
            let buffer_c = buffer.clone();
            let toast = toast_cb.clone();
            copy_btn.connect_clicked(move |btn| {
                let start = buffer_c.start_iter();
                let end = buffer_c.end_iter();
                let text = buffer_c.text(&start, &end, false);
                let display = btn.display();
                display.clipboard().set_text(&text);
                toast("Log berhasil disalin ke clipboard.");
            });
        }

        // Wire Clear Button
        {
            let buffer_c = buffer.clone();
            let toast = toast_cb;
            clear_btn.connect_clicked(move |_| {
                buffer_c.set_text("");
                toast("Tampilan log dibersihkan.");
            });
        }

        // Wire Search Filter
        {
            let raw_c = raw_logs.clone();
            let buffer_c = buffer;
            let scrolled_c = scrolled;
            let auto_scroll_c = auto_scroll_check.clone();

            search_entry.connect_changed(move |entry| {
                let query = entry.text().to_string();
                let raw = raw_c.lock().unwrap().clone();

                let displayed = if query.trim().is_empty() {
                    raw
                } else {
                    raw.lines()
                        .filter(|l| l.to_lowercase().contains(&query.to_lowercase()))
                        .collect::<Vec<&str>>()
                        .join("\n")
                };

                buffer_c.set_text(&displayed);

                if auto_scroll_c.is_active() {
                    let vadj = scrolled_c.vadjustment();
                    vadj.set_value(vadj.upper());
                }
            });
        }

        // Periodic auto-refresh timer (every 4 seconds if checked)
        {
            let fetcher = fetch_logs.clone();
            let auto_ref = auto_refresh_check;
            glib::timeout_add_seconds_local(4, move || {
                if auto_ref.is_active() {
                    fetcher();
                }
                glib::ControlFlow::Continue
            });
        }

        // Trigger initial fetch
        fetch_logs();

        Self {
            container: root,
            raw_logs,
        }
    }
}
