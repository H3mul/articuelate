use egui::{ComboBox, RichText, Slider, Ui};

use crate::cue::{Cue, FadeCurve, Task};
use crate::panels::style;

/// Renders the context-dependent detail panel (bottom-left).
pub fn render(ui: &mut Ui, selected_cue: Option<&Cue>, selected_task: Option<&Task>) {
    match (selected_cue, selected_task) {
        (Some(cue), Some(task)) => {
            render_task_editor(ui, cue, task);
        }
        (Some(cue), None) => {
            render_cue_properties(ui, cue);
        }
        (None, _) => {
            render_global_settings(ui);
        }
    }
}

fn render_task_editor(ui: &mut Ui, cue: &Cue, task: &Task) {
    // Header card: shows which cue + task is being edited.
    style::show_card(ui, Some(&format!("Task – {}", task.target_name)), |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Cue:").weak());
            ui.strong(format!("{} – {}", cue.number, cue.name));
        });
    });
    ui.add_space(6.0);

    // Form card: target / property / output (read-only metadata).
    style::show_card(ui, Some("Target"), |ui| {
        ui.horizontal(|ui| {
            ui.label("Target:");
            ui.strong(&task.target_name);
        });
        ui.horizontal(|ui| {
            ui.label("Property:");
            ui.strong(&task.property);
        });
        ui.horizontal(|ui| {
            ui.label("Output:");
            ui.strong(&task.output.name);
        });
    });
    ui.add_space(6.0);

    // Parameters card: the editable controls.
    style::show_card(ui, Some("Parameters"), |ui| {
        ui.horizontal(|ui| {
            ui.label("Target Vol:");
            let mut v = task.target_value;
            ui.add(Slider::new(&mut v, -60.0..=12.0));
            ui.label("dB");
        });
        ui.horizontal(|ui| {
            ui.label("Duration:");
            let mut d = task.duration_secs;
            ui.add(Slider::new(&mut d, 0.1..=30.0));
            ui.label("s");
        });

        ui.horizontal(|ui| {
            ui.label("Curve:");
            let variants = ["Linear", "Logarithmic", "Exponential"];
            let mut idx = match task.curve {
                FadeCurve::Linear => 0,
                FadeCurve::Logarithmic => 1,
                FadeCurve::Exponential => 2,
            };
            ComboBox::from_id_salt("curve_combo")
                .selected_text(variants[idx])
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut idx, 0, "Linear");
                    ui.selectable_value(&mut idx, 1, "Logarithmic");
                    ui.selectable_value(&mut idx, 2, "Exponential");
                });
        });
    });
}

fn render_cue_properties(ui: &mut Ui, cue: &Cue) {
    // Header card
    style::show_card(
        ui,
        Some(&format!("Cue {} – {}", cue.number, cue.name)),
        |ui| {
            ui.label("Trigger constraints and base properties for the selected cue.");
        },
    );
    ui.add_space(6.0);

    // Trigger card: pre / post wait.
    style::show_card(ui, Some("Trigger"), |ui| {
        ui.horizontal(|ui| {
            ui.label("Pre-wait:");
            let mut v = cue.pre_wait_secs;
            ui.add(Slider::new(&mut v, 0.0..=30.0));
            ui.label("s");
        });
        ui.horizontal(|ui| {
            ui.label("Post-wait:");
            let mut v = cue.post_wait_secs;
            ui.add(Slider::new(&mut v, 0.0..=30.0));
            ui.label("s");
        });
    });
    ui.add_space(6.0);

    // Notes card.
    style::show_card(ui, Some("Designer Notes"), |ui| {
        let mut notes = cue.notes.clone();
        ui.add(
            egui::TextEdit::multiline(&mut notes)
                .desired_rows(4)
                .hint_text("Notes for the operator…"),
        );
    });
}

fn render_global_settings(ui: &mut Ui) {
    // Header card
    style::show_card(ui, Some("Global Show Settings"), |ui| {
        ui.label(
            RichText::new("Nothing selected — adjust the global show defaults.")
                .weak(),
        );
    });
    ui.add_space(6.0);

    style::show_card(ui, Some("Output"), |ui| {
        ui.horizontal(|ui| {
            ui.label("Master Show Volume");
            let mut master_vol = 0.0;
            ui.add(Slider::new(&mut master_vol, -60.0..=12.0));
            ui.label("dB");
        });
        ui.horizontal(|ui| {
            ui.label("Default Output Device:");
            ui.strong("Main L/R");
        });
    });
}
