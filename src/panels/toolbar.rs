use egui::{Color32, Frame, Margin, RichText, TextEdit, Ui};

use crate::engine::AudioEngine;

/// Renders the toolbar at the top of the window.
pub fn render(ui: &mut Ui, search_query: &mut String, engine: &mut AudioEngine) {
    let frame = Frame {
        inner_margin: Margin::symmetric(8, 4),
        ..Default::default()
    };
    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            // Transport controls wrapped in a small inset card.
            let transport = Frame {
                fill: ui.style().visuals.widgets.noninteractive.bg_fill,
                stroke: ui.style().visuals.widgets.noninteractive.bg_stroke,
                inner_margin: Margin::symmetric(4, 2),
                corner_radius: egui::CornerRadius::same(6),
                ..Default::default()
            };
            transport.show(ui, |ui| {
                if ui.button("⏸  PAUSE").clicked() {
                    // pause not yet implemented
                }
                if ui.button("▶  GO").clicked() {
                    engine.fire_next();
                }
                if ui.button("◀  BACK").clicked() {
                    // back not yet implemented
                }
            });

            ui.add_space(8.0);

            // Search / Filter wrapped in an inset card.
            let search = Frame {
                fill: ui.style().visuals.widgets.noninteractive.bg_fill,
                stroke: ui.style().visuals.widgets.noninteractive.bg_stroke,
                inner_margin: Margin::symmetric(6, 2),
                corner_radius: egui::CornerRadius::same(6),
                ..Default::default()
            };
            search.show(ui, |ui| {
                ui.label(RichText::new("🔍").weak());
                ui.add(
                    TextEdit::singleline(search_query)
                        .hint_text("Search cues…")
                        .desired_width(220.0)
                        .frame(egui::Frame::NONE),
                );
            });

            // Push the PANIC button to the right.
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(RichText::new("⛔ PANIC").color(Color32::WHITE).strong())
                            .fill(Color32::from_rgb(200, 60, 60))
                            .min_size(egui::vec2(110.0, 32.0))
                            .corner_radius(egui::CornerRadius::same(6)),
                    )
                    .clicked()
                {
                    engine.stop_all();
                }
            });
        });
    });
}
