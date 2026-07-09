use egui::Ui;

use crate::cue::{Cue, FadeCurve, Task};

/// Renders the context-dependent detail panel (bottom-left).
pub fn render(ui: &mut Ui, selected_cue: Option<&Cue>, selected_task: Option<&Task>) {
    ui.heading("Detail Panel");
    ui.separator();

    match (selected_cue, selected_task) {
        (Some(cue), Some(task)) => {
            // Task editing mode
            render_task_editor(ui, cue, task);
        }
        (Some(cue), None) => {
            // Cue properties mode
            render_cue_properties(ui, cue);
        }
        (None, _) => {
            // Global show settings
            render_global_settings(ui);
        }
    }
}

fn render_task_editor(ui: &mut Ui, cue: &Cue, task: &Task) {
    ui.label(format!("Context: Task – {}", task.target_name));
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Target:");
        ui.label(&task.target_name);
    });

    ui.horizontal(|ui| {
        ui.label("Property:");
        ui.label(&task.property);
    });

    ui.horizontal(|ui| {
        ui.label("Target Value:");
        ui.add(egui::Slider::new(
            &mut task.target_value.clone(),
            -60.0..=12.0,
        ));
        ui.label("dB");
    });

    ui.horizontal(|ui| {
        ui.label("Duration:");
        ui.add(egui::Slider::new(
            &mut task.duration_secs.clone(),
            0.1..=30.0,
        ));
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
        egui::ComboBox::from_id_salt("curve_combo")
            .selected_text(variants[idx])
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut idx, 0, "Linear");
                ui.selectable_value(&mut idx, 1, "Logarithmic");
                ui.selectable_value(&mut idx, 2, "Exponential");
            });
        let curve = match idx {
            0 => FadeCurve::Linear,
            1 => FadeCurve::Logarithmic,
            _ => FadeCurve::Exponential,
        };
        let _ = curve; // would write back in real app
    });

    ui.horizontal(|ui| {
        ui.label("Output:");
        ui.label(&task.output.name);
    });

    ui.label(format!("Cue: {} – {}", cue.number, cue.name));
}

fn render_cue_properties(ui: &mut Ui, cue: &Cue) {
    ui.label(format!("Context: Cue {} – {}", cue.number, cue.name));
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Pre-wait:");
        ui.add(egui::Slider::new(
            &mut cue.pre_wait_secs.clone(),
            0.0..=30.0,
        ));
        ui.label("s");
    });

    ui.horizontal(|ui| {
        ui.label("Post-wait:");
        ui.add(egui::Slider::new(
            &mut cue.post_wait_secs.clone(),
            0.0..=30.0,
        ));
        ui.label("s");
    });

    ui.label("Notes:");
    ui.text_edit_multiline(&mut cue.notes.clone());
}

fn render_global_settings(ui: &mut Ui) {
    ui.label("Context: Global Show Settings");
    ui.separator();

    ui.label("Master Show Volume");
    let mut master_vol = 0.0;
    ui.add(egui::Slider::new(&mut master_vol, -60.0..=12.0).text("dB"));

    ui.separator();
    ui.label("Default Output Device:");
    ui.label("Main L/R");
}