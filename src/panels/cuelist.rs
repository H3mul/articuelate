use egui::{Color32, Frame, Margin, RichText, Sense, Ui};

use crate::cue::{Cue, CueStatus, FollowMode};
use crate::panels::style;

/// Renders the main cuelist panel — the top-left data grid.
pub fn render(ui: &mut Ui, cues: &[Cue], selected_index: &mut usize) {
    if cues.is_empty() {
        style::show_card(ui, None, |ui| {
            ui.label(
                RichText::new("No cues loaded. Add a cue to get started.")
                    .weak(),
            );
        });
        return;
    }

    for (i, cue) in cues.iter().enumerate() {
        render_cue_card(ui, cue, i, selected_index);
    }
}

/// Renders a single cue as a card. Selected cues get a tinted fill and a
/// stronger outline, indented cues get a left margin.
fn render_cue_card(ui: &mut Ui, cue: &Cue, index: usize, selected_index: &mut usize) {
    let selected = index == *selected_index;
    let visuals = ui.style().visuals.clone();

    let fill = if selected {
        visuals.selection.bg_fill
    } else {
        visuals.widgets.noninteractive.bg_fill
    };

    let stroke_color = if selected {
        visuals.selection.stroke.color
    } else {
        visuals.widgets.noninteractive.bg_stroke.color
    };

    let indent = if cue.indented { 20.0 } else { 0.0 };

    let frame = Frame {
        fill,
        stroke: egui::Stroke::new(if selected { 1.5 } else { 1.0 }, stroke_color),
        inner_margin: Margin::symmetric((4.0 + indent) as i8, 6),
        corner_radius: egui::CornerRadius::same(6),
        ..Default::default()
    };

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            // Status icon
            let (icon, color) = match cue.status {
                CueStatus::Playing => ("▶", Color32::from_rgb(80, 200, 120)),
                CueStatus::Paused => ("⏸", Color32::from_rgb(230, 200, 80)),
                CueStatus::Complete => ("✓", Color32::from_rgb(120, 120, 120)),
                CueStatus::Ready => ("○", visuals.text_color()),
            };
            ui.colored_label(color, icon);

            // Number + Name
            ui.strong(format!("CUE {}", cue.number));
            ui.label(&cue.name);

            // Follow-mode badge
            let (badge, badge_color) = match cue.follow_mode {
                FollowMode::Manual => (None, Color32::PLACEHOLDER),
                FollowMode::AutoContinue => (
                    Some(" (Auto-Continue)"),
                    Color32::from_rgb(120, 180, 230),
                ),
                FollowMode::AutoFollow => (
                    Some(" (Auto-Follow)"),
                    Color32::from_rgb(200, 140, 230),
                ),
            };
            if let Some(b) = badge {
                ui.colored_label(badge_color, b);
            }

            // Click anywhere on the card to select.
            let response = ui.interact(
                ui.min_rect(),
                ui.id().with("cue").with(index),
                Sense::click(),
            );
            if response.clicked() {
                *selected_index = index;
            }
        });
    });
}

/// Renders the cuelist panel header row (used inside a containing panel).
pub fn render_header(ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("MAIN CUELIST").strong().size(14.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new("0 cues").weak());
        });
    });
    ui.add_space(6.0);
}
