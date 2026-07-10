use std::sync::Mutex;

use app_backend::cue::{Cue, CueStatus, FollowMode};
use app_backend::engine::AudioEngine;

/// Shared, process-wide application state managed by Tauri.
pub struct AppState {
    pub cues: Mutex<Vec<Cue>>,
    pub engine: Mutex<AudioEngine>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            cues: Mutex::new(sample_cues()),
            engine: Mutex::new(AudioEngine::new()),
        }
    }
}

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
                app_backend::cue::Task {
                    target_name: "BGM".into(),
                    property: "Volume".into(),
                    target_value: -24.0,
                    duration_secs: 3.0,
                    curve: app_backend::cue::FadeCurve::Linear,
                    output: app_backend::cue::OutputTarget {
                        name: "Main L/R".into(),
                    },
                },
                app_backend::cue::Task {
                    target_name: "Player".into(),
                    property: "Play".into(),
                    target_value: 0.0,
                    duration_secs: 0.0,
                    curve: app_backend::cue::FadeCurve::Linear,
                    output: app_backend::cue::OutputTarget {
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
