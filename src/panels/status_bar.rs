use egui::Ui;

use crate::engine::AudioEngine;

/// Renders the status bar at the bottom of the window.
pub fn render(ui: &mut Ui, engine: &AudioEngine) {
    ui.horizontal(|ui| {
        // Hardware status
        let connected = engine.is_connected();
        let color = if connected {
            egui::Color32::GREEN
        } else {
            egui::Color32::RED
        };
        ui.colored_label(
            color,
            format!(
                "STATUS: {} ({})",
                if connected { "Connected" } else { "Disconnected" },
                engine.audio_device_name(),
            ),
        );

        // Push performance metrics to the right
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!("DSP: {:.0}%", engine.dsp_usage()));
            ui.label(format!("CPU: {:.0}%", engine.cpu_usage()));
        });
    });
}