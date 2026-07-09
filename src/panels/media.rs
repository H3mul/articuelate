use egui::Ui;

use crate::engine::ActivePlayback;

/// Renders the active-media (right) panel showing live playback telemetry.
pub fn render(ui: &mut Ui, playbacks: &[ActivePlayback]) {
    ui.heading("Active Media");
    ui.separator();

    if playbacks.is_empty() {
        ui.label("No active audio layers.");
        return;
    }

    for pb in playbacks {
        render_playback_row(ui, pb);
        ui.separator();
    }
}

fn render_playback_row(ui: &mut Ui, pb: &ActivePlayback) {
    ui.label(format!("> CUE {:.1} ({})", pb.cue_number, pb.label));

    // Volume meter (simplified horizontal bar)
    let normalized_vol = ((pb.volume_db + 60.0) / 72.0).clamp(0.0, 1.0);
    ui.horizontal(|ui| {
        ui.label("Vol:");
        let meter_width = ui.available_width() - 60.0;
        let (_, rect) = ui.allocate_space(egui::vec2(meter_width.max(40.0), 12.0));
        ui.painter().rect_filled(rect, 0.0, egui::Color32::DARK_GRAY);
        if normalized_vol > 0.0 {
            let fill_rect = egui::Rect::from_min_size(
                rect.min,
                egui::vec2(rect.width() * normalized_vol, rect.height()),
            );
            let color = if normalized_vol > 0.85 {
                egui::Color32::RED
            } else {
                egui::Color32::GREEN
            };
            ui.painter().rect_filled(fill_rect, 0.0, color);
        }
        ui.label(format!("{:.0} dB", pb.volume_db));
    });

    // Progress bar (playhead scrubbing placeholder)
    ui.horizontal(|ui| {
        ui.label("Progress:");
        let mut p = pb.progress;
        let response = ui.add(
            egui::Slider::new(&mut p, 0.0..=1.0)
                .text("")
                .show_value(false),
        );
        // In a real implementation, dragging would scrub the audio playhead.
        let _ = response;
        ui.label(format!("{:.0}%", pb.progress * 100.0));
    });

    // Placeholder for manual volume override (future: interactive knob / drag)
    ui.horizontal(|ui| {
        ui.label("Manual Override:");
        let mut override_db = 0.0f32;
        ui.add(egui::Slider::new(&mut override_db, -24.0..=24.0).text("dB"));
    });
}