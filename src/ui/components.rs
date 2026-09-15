use crate::system::service::ServiceState;
use gtk4::prelude::*;
use gtk4::{Align, Box, Label, Orientation};

pub struct UiComponents;

impl UiComponents {
    /// Create a styled badge indicating the service state
    pub fn create_status_badge(state: ServiceState) -> Label {
        let label = Label::new(Some(&format!("● {}", state.as_str())));
        label.add_css_class("status-badge");
        label.add_css_class(state.badge_class());
        label.set_valign(Align::Center);
        label
    }

    /// Create a card container box
    pub fn create_card() -> Box {
        let card = Box::new(Orientation::Vertical, 12);
        card.add_css_class("card-frame");
        card
    }

    /// Create a clean metric row (Key: Value)
    pub fn create_metric_row(label_text: &str, value_text: &str) -> (Box, Label) {
        let row = Box::new(Orientation::Horizontal, 12);
        row.set_hexpand(true);

        let lbl = Label::new(Some(label_text));
        lbl.add_css_class("metric-label");
        lbl.set_halign(Align::Start);
        lbl.set_hexpand(true);

        let val = Label::new(Some(value_text));
        val.add_css_class("metric-value");
        val.set_halign(Align::End);
        val.set_selectable(true);

        row.append(&lbl);
        row.append(&val);

        (row, val)
    }

    /// Create a check/cross detection badge (e.g. "✓ MariaDB detected")
    pub fn create_detection_badge(text: &str, detected: bool) -> Box {
        let row = Box::new(Orientation::Horizontal, 8);
        row.set_margin_top(2);
        row.set_margin_bottom(2);

        let icon = if detected { "✓" } else { "✕" };
        let icon_lbl = Label::new(Some(icon));
        if detected {
            icon_lbl.add_css_class("status-running");
        } else {
            icon_lbl.add_css_class("status-error");
        }
        icon_lbl.add_css_class("status-badge");

        let text_lbl = Label::new(Some(text));
        text_lbl.set_halign(Align::Start);

        row.append(&icon_lbl);
        row.append(&text_lbl);
        row
    }
}
