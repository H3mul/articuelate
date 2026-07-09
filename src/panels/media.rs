use egui::{Color32, RichText, Sense, Slider, Ui};

use crate::engine::ActivePlayback;
use crate::panels::style;

/// Renders the active-media (right) panel showing live playback telemetry.
pub fn render(ui: &mut Ui, playbacks: &[ActivePlayback]) {
    style::show_card(ui, Some("Active Media"), |ui| {
        if playbacks.is_empty() {
            ui.label(RichText::new("No active audio layers.").weak());
            return;
        }

        for pb in playbacks {
            render_playback_card(ui, pb);
            ui.add_space(6.0);
        }
    });
}

fn render_playback_card(ui: &mut Ui, pb: &ActivePlayback) {
    style::card_inner(ui).show(ui, |ui| {
        // Header row
        ui.horizontal(|ui| {
            ui.colored_label(
                Color32::from_rgb(80, 200, 120),
                RichText::new("▶").strong(),
            );
            ui.strong(format!("CUE {:.1}", pb.cue_number));
            ui.label(&pb.label);
        });
        ui.add_space(4.0);

        // Volume meter
        let normalized_vol = ((pb.volume_db + 60.0) / 72.0).clamp(0.0, 1.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Vol").weak());
            let meter_width = ui.available_width() - 60.0;
            let (_, rect) = ui.allocate_space(egui::vec2(meter_width.max(40.0), 12.0));
            ui.painter().rect_filled(rect, 2.0, egui::Color32::DARK_GRAY);
            if normalized_vol > 0.0 {
                let fill_rect = egui::Rect::from_min_size(
                    rect.min,
                    egui::vec2(rect.width() * normalized_vol, rect.height()),
                );
                let color = if normalized_vol > 0.85 {
                    Color32::RED
                } else {
                    Color32::from_rgb(80, 200, 120)
                };
                ui.painter().rect_filled(fill_rect, 2.0, color);
            }
            ui.label(format!("{:.0} dB", pb.volume_db));
        });

        // Progress bar (playhead scrubbing placeholder)
        ui.horizontal(|ui| {
            ui.label(RichText::new("Progress").weak());
            let mut p = pb.progress;
            let response = ui.add(
                Slider::new(&mut p, 0.0..=1.0)
                    .show_value(false),
            );
            let _ = response; // would scrub the audio playhead in a real app
            ui.label(format!("{:.0}%", pb.progress * 100.0));
        });

        // Manual override — wraps the slider in an inset card.
        ui.horizontal(|ui| {
            ui.label(RichText::new("Manual Override").weak());
            let mut override_db = 0.0f32;
            ui.add(Slider::new(&mut override_db, -24.0..=24.0));
            ui.label("dB");
        });

        // Click-to-focus the whole card.
        let _ = ui.interact(ui.min_rect(), ui.id().with("media").with(pb.label.clone()), Sense::click());
    });
}
