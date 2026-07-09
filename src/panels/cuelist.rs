use egui::Ui;

use crate::cue::{Cue, CueStatus, FollowMode};

/// Renders the main cuelist panel — the top-left data grid.
pub fn render(ui: &mut Ui, cues: &[Cue], selected_index: &mut usize) {
    egui_extras::StripBuilder::new(ui)
        .sizes(egui_extras::Size::remainder(), cues.len().max(1))
        .vertical(|mut strip| {
            if cues.is_empty() {
                strip.cell(|ui| {
                    ui.label("No cues loaded. Add a cue to get started.");
                });
                return;
            }

            for (i, cue) in cues.iter().enumerate() {
                strip.cell(|ui| {
                    let indent = if cue.indented { 20.0 } else { 0.0 };

                    let selected = i == *selected_index;
                    let bg = if selected {
                        ui.style().visuals.selection.bg_fill
                    } else if i % 2 == 0 {
                        ui.style().visuals.panel_fill
                    } else {
                        ui.style().visuals.faint_bg_color
                    };

                    let frame = egui::Frame {
                        fill: bg,
                        inner_margin: egui::Margin::symmetric((4.0 + indent) as i8, 2),
                        ..Default::default()
                    };

                    frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Status icon
                            let icon = match cue.status {
                                CueStatus::Playing => "▶",
                                CueStatus::Paused => "⏸",
                                CueStatus::Complete => "✓",
                                CueStatus::Ready => "○",
                            };
                            ui.label(icon);

                            // Number + Name
                            ui.strong(format!("CUE {}", cue.number));
                            ui.label(&cue.name);

                            // Follow-mode badge
                            let badge = match cue.follow_mode {
                                FollowMode::Manual => "",
                                FollowMode::AutoContinue => " (Auto-Continue)",
                                FollowMode::AutoFollow => " (Auto-Follow)",
                            };
                            if !badge.is_empty() {
                                ui.colored_label(egui::Color32::LIGHT_BLUE, badge);
                            }

                            // Click to select
                            let response = ui.interact(
                                ui.min_rect(),
                                ui.id().with("cue").with(i),
                                egui::Sense::click(),
                            );
                            if response.clicked() {
                                *selected_index = i;
                            }
                        });
                    });
                });
            }
        });
}

/// Renders the cuelist panel header row (used inside a containing panel).
pub fn render_header(ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("MAIN CUELIST").strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!("{} cues", 0));
        });
    });
    ui.separator();
}