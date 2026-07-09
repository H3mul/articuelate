use egui::{Color32, Frame, Margin, RichText, Ui};

use crate::engine::AudioEngine;

/// Renders the status bar at the bottom of the window.
pub fn render(ui: &mut Ui, engine: &AudioEngine) {
    let frame = Frame {
        inner_margin: Margin::symmetric(8, 2),
        ..Default::default()
    };
    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            // Hardware status with a colored dot.
            let connected = engine.is_connected();
            let dot_color = if connected {
                Color32::from_rgb(80, 200, 120)
            } else {
                Color32::from_rgb(220, 80, 80)
            };
            ui.colored_label(dot_color, "●");
            ui.label(format!(
                "{} ({})",
                if connected { "Connected" } else { "Disconnected" },
                engine.audio_device_name(),
            ));

            // Performance metrics, right-aligned.
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("DSP: {:.0}%", engine.dsp_usage())).weak());
                ui.label(RichText::new(format!("CPU: {:.0}%", engine.cpu_usage())).weak());
            });
        });
    });
}
