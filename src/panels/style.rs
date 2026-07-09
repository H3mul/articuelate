//! Shared helpers for the modern, card-based look used by every panel.
//!
//! The application is themed by `elegance::Theme::slate()`, which rewrites
//! `egui::Style` once per frame in `App::ui`. After that, the only thing
//! left to do is pick frames for grouping related controls together — that
//! is what this module provides.

use egui::{Color32, Frame, Margin, Shadow, Stroke, Ui};

/// Build a card frame used to wrap a logical group of controls.
pub fn card(ui: &Ui) -> Frame {
    let visuals = ui.style().visuals.clone();
    Frame {
        fill: visuals.widgets.noninteractive.bg_fill,
        stroke: Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color),
        inner_margin: Margin::same(10),
        corner_radius: egui::CornerRadius::same(8),
        shadow: Shadow {
            offset: [0, 1],
            blur: 4,
            spread: 0,
            color: Color32::from_black_alpha(40),
        },
        ..Default::default()
    }
}

/// Build a tighter, "inset" frame used for nested groups (e.g. a sub-form
/// inside a card).
pub fn card_inner(ui: &Ui) -> Frame {
    let visuals = ui.style().visuals.clone();
    Frame {
        fill: visuals.faint_bg_color,
        stroke: Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color),
        inner_margin: Margin::same(8),
        corner_radius: egui::CornerRadius::same(6),
        ..Default::default()
    }
}

/// Convenience wrapper: show a card with an optional heading label.
pub fn show_card<R>(
    ui: &mut Ui,
    heading: Option<&str>,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    card(ui).show(ui, |ui| {
        if let Some(h) = heading {
            ui.label(egui::RichText::new(h).strong());
            ui.add_space(4.0);
        }
        add_contents(ui)
    })
}
