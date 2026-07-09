mod cue;
mod engine;

use std::cell::RefCell;
use std::rc::Rc;

use slint::{ModelRc, SharedString, VecModel};

use cue::{Cue, CueStatus, FollowMode};
use engine::{ActivePlayback, AudioEngine};

slint::include_modules!();

// ---------------------------------------------------------------------------
// Sample data (placeholder show — will later be loaded via Serde).

fn sample_cues() -> Vec<Cue> {
    vec![
        Cue {
            number: 1.0,
            name: "Storm Intro".into(),
            status: CueStatus::Ready,
            follow_mode: FollowMode::Manual,
            tasks: vec![
                cue::Task {
                    target_name: "BGM".into(),
                    property: "Volume".into(),
                    target_value: -24.0,
                    duration_secs: 3.0,
                    curve: cue::FadeCurve::Linear,
                    output: cue::OutputTarget {
                        name: "Main L/R".into(),
                    },
                },
                cue::Task {
                    target_name: "Player".into(),
                    property: "Play".into(),
                    target_value: 0.0,
                    duration_secs: 0.0,
                    curve: cue::FadeCurve::Linear,
                    output: cue::OutputTarget {
                        name: "Main L/R".into(),
                    },
                },
            ],
            indented: false,
            audio_file_name: Some("Wind_Loop.wav".into()),
            ..Default::default()
        },
        Cue {
            number: 2.0,
            name: "Thunder Strike".into(),
            status: CueStatus::Ready,
            follow_mode: FollowMode::Manual,
            tasks: vec![],
            indented: true,
            audio_file_name: Some("Thunder.wav".into()),
            ..Default::default()
        },
        Cue {
            number: 3.0,
            name: "Storm Outro".into(),
            status: CueStatus::Ready,
            follow_mode: FollowMode::AutoFollow,
            tasks: vec![],
            indented: true,
            ..Default::default()
        },
    ]
}

// ---------------------------------------------------------------------------
// Display helpers (flatten Rust data into the plain Slint structs).

fn status_icon(s: CueStatus) -> &'static str {
    match s {
        CueStatus::Playing => "▶",
        CueStatus::Paused => "⏸",
        CueStatus::Complete => "✓",
        CueStatus::Ready => "○",
    }
}

fn follow_text(f: FollowMode) -> &'static str {
    match f {
        FollowMode::Manual => "",
        FollowMode::AutoContinue => " (Auto-Continue)",
        FollowMode::AutoFollow => " (Auto-Follow)",
    }
}

fn to_cue_item(cue: &Cue, selected: bool) -> CueItem {
    CueItem {
        number: SharedString::from(format!("{:.1}", cue.number)),
        name: SharedString::from(cue.name.as_str()),
        status: SharedString::from(status_icon(cue.status)),
        follow: SharedString::from(follow_text(cue.follow_mode)),
        indented: cue.indented,
        selected,
    }
}

fn to_playback_item(pb: &ActivePlayback) -> PlaybackItem {
    PlaybackItem {
        cue_number: SharedString::from(format!("{:.1}", pb.cue_number)),
        label: SharedString::from(pb.label.as_str()),
        volume_db: SharedString::from(format!("{:.0} dB", pb.volume_db)),
        progress: pb.progress,
    }
}

/// Builds a barebones text description of the current selection for the
/// detail inspector panel.
fn describe(cues: &[Cue], idx: usize) -> String {
    match cues.get(idx) {
        Some(cue) => {
            let mut s = String::new();
            s.push_str(&format!("Cue {:.1} – {}\n", cue.number, cue.name));
            s.push_str(&format!("Status: {:?}\n", cue.status));
            s.push_str(&format!("Follow: {:?}\n", cue.follow_mode));
            s.push_str(&format!(
                "Pre-wait: {:.1}s   Post-wait: {:.1}s\n",
                cue.pre_wait_secs, cue.post_wait_secs
            ));
            if !cue.notes.is_empty() {
                s.push_str(&format!("Notes: {}\n", cue.notes));
            }
            if !cue.tasks.is_empty() {
                s.push_str("Tasks:\n");
                for t in &cue.tasks {
                    s.push_str(&format!(
                        "  - {} {} -> {:.1} ({:.1}s, {:?})\n",
                        t.target_name, t.property, t.target_value, t.duration_secs, t.curve
                    ));
                }
            }
            s
        }
        None => "Nothing selected — adjust the global show defaults.".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Shared mutable application state.

struct AppState {
    cues: Vec<Cue>,
    selected_index: usize,
    search: String,
    engine: AudioEngine,
}

/// Push the current `AppState` into the Slint reactive properties.
fn refresh(
    ui: &MainWindow,
    state: &Rc<RefCell<AppState>>,
    cues_model: &Rc<VecModel<CueItem>>,
    playbacks_model: &Rc<VecModel<PlaybackItem>>,
) {
    let s = state.borrow();

    // Cuelist (filtered by the search query).
    let q = s.search.to_lowercase();
    let filtered: Vec<CueItem> = s
        .cues
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            q.is_empty()
                || c.name.to_lowercase().contains(&q)
                || c.audio_file_name
                    .as_ref()
                    .map_or(false, |f| f.to_lowercase().contains(&q))
        })
        .map(|(i, c)| to_cue_item(c, i == s.selected_index))
        .collect();
    cues_model.set_vec(filtered);

    // Detail inspector.
    ui.set_detail_text(SharedString::from(describe(&s.cues, s.selected_index)));

    // Active media telemetry.
    let pbs: Vec<PlaybackItem> = s.engine.active_playbacks().iter().map(to_playback_item).collect();
    playbacks_model.set_vec(pbs);

    // Status bar.
    ui.set_status_text(SharedString::from(format!(
        "{} ({})    CPU: {:.0}%   DSP: {:.0}%",
        if s.engine.is_connected() {
            "Connected"
        } else {
            "Disconnected"
        },
        s.engine.audio_device_name(),
        s.engine.cpu_usage(),
        s.engine.dsp_usage(),
    )));
}

// ---------------------------------------------------------------------------
// Entry point.

fn main() -> Result<(), slint::PlatformError> {
    let state = Rc::new(RefCell::new(AppState {
        cues: sample_cues(),
        selected_index: 0,
        search: String::new(),
        engine: AudioEngine::new(),
    }));

    let ui = MainWindow::new()?;

    let cues_model = Rc::new(VecModel::<CueItem>::default());
    let playbacks_model = Rc::new(VecModel::<PlaybackItem>::default());
    ui.set_cues(ModelRc::new(cues_model.clone()));
    ui.set_playbacks(ModelRc::new(playbacks_model.clone()));

    refresh(&ui, &state, &cues_model, &playbacks_model);

    // --- Wire callbacks ---
    let st = state.clone();
    let ui_w = ui.as_weak();
    let cm = cues_model.clone();
    let pm = playbacks_model.clone();
    ui.on_cue_selected(move |idx| {
        st.borrow_mut().selected_index = idx as usize;
        let ui = ui_w.upgrade().unwrap();
        refresh(&ui, &st, &cm, &pm);
    });

    let st = state.clone();
    let ui_w = ui.as_weak();
    let cm = cues_model.clone();
    let pm = playbacks_model.clone();
    ui.on_search_changed(move |text| {
        st.borrow_mut().search = text.to_string();
        let ui = ui_w.upgrade().unwrap();
        refresh(&ui, &st, &cm, &pm);
    });

    let st = state.clone();
    let ui_w = ui.as_weak();
    let cm = cues_model.clone();
    let pm = playbacks_model.clone();
    ui.on_go_clicked(move || {
        st.borrow_mut().engine.fire_next();
        let ui = ui_w.upgrade().unwrap();
        refresh(&ui, &st, &cm, &pm);
    });

    let st = state.clone();
    let ui_w = ui.as_weak();
    let cm = cues_model.clone();
    let pm = playbacks_model.clone();
    ui.on_back_clicked(move || {
        // TODO: step back to the previous cue.
        let ui = ui_w.upgrade().unwrap();
        refresh(&ui, &st, &cm, &pm);
    });

    let st = state.clone();
    let ui_w = ui.as_weak();
    let cm = cues_model.clone();
    let pm = playbacks_model.clone();
    ui.on_pause_clicked(move || {
        // TODO: pause the active playback.
        let ui = ui_w.upgrade().unwrap();
        refresh(&ui, &st, &cm, &pm);
    });

    let st = state.clone();
    let ui_w = ui.as_weak();
    let cm = cues_model.clone();
    let pm = playbacks_model.clone();
    ui.on_panic_clicked(move || {
        st.borrow_mut().engine.stop_all();
        let ui = ui_w.upgrade().unwrap();
        refresh(&ui, &st, &cm, &pm);
    });

    ui.run()
}
