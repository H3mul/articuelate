use egui::Ui;

use crate::engine::AudioEngine;

/// Renders the toolbar at the top of the window.
/// Returns `true` if the engine was mutated.
pub fn render(ui: &mut Ui, search_query: &mut String, engine: &mut AudioEngine) {
    ui.horizontal(|ui| {
        // Transport: Pause, GO, Back
        if ui.button("⏸  PAUSE").clicked() {
            // pause not yet implemented
        }
        if ui.button("▶  GO").clicked() {
            engine.fire_next();
        }
        if ui.button("◀  BACK").clicked() {
            // back not yet implemented
        }

        ui.separator();

        // Search / Filter
        ui.label("Search:");
        ui.add(
            egui::TextEdit::singleline(search_query)
                .hint_text("wind")
                .desired_width(200.0),
        );

        // Push the PANIC button to the right
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(
                    egui::Button::new("⛔ PANIC")
                        .fill(egui::Color32::from_rgb(180, 40, 40))
                        .min_size(egui::vec2(100.0, 32.0)),
                )
                .clicked()
            {
                engine.stop_all();
            }
        });
    });
}