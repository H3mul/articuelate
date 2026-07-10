use std::sync::Mutex;

use crate::cue::{Cue, CueStatus, FollowMode};
use crate::engine::AudioEngine;

/// Shared, process-wide application state.
///
/// For this proof-of-concept the `Vec<Cue>` flat chain and the audio engine
/// live here in Rust (the single source of truth). The React/Tauri frontend
/// reads snapshots via IPC commands and dispatches transport actions back.
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
                crate::cue::Task {
                    target_name: "BGM".into(),
                    property: "Volume".into(),
                    target_value: -24.0,
                    duration_secs: 3.0,
                    curve: crate::cue::FadeCurve::Linear,
                    output: crate::cue::OutputTarget {
                        name: "Main L/R".into(),
                    },
                },
                crate::cue::Task {
                    target_name: "Player".into(),
                    property: "Play".into(),
                    target_value: 0.0,
                    duration_secs: 0.0,
                    curve: crate::cue::FadeCurve::Linear,
                    output: crate::cue::OutputTarget {
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
