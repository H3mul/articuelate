use serde::{Deserialize, Serialize};

/// How a cue transitions after completing its tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FollowMode {
    /// Waits for a manual GO press.
    Manual,
    /// Automatically continues after the current tasks finish, with no delay.
    AutoContinue,
    /// Automatically fires after a specified duration.
    AutoFollow,
}

/// The overall state of a cue in the playback lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CueStatus {
    /// Loaded and ready, awaiting the standby / fire signal.
    Ready,
    /// Playback is in progress.
    Playing,
    /// Paused mid-playback.
    Paused,
    /// Finished or released.
    Complete,
}

/// A fade curve applied to a volume or parameter ramp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FadeCurve {
    Linear,
    Logarithmic,
    Exponential,
}

/// A routing target for audio output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputTarget {
    pub name: String,
}

/// A single parameter instruction inside a cue — e.g. "fade BGM volume to -24 dB over 3s".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub target_name: String,
    pub property: String,
    pub target_value: f32,
    pub duration_secs: f32,
    pub curve: FadeCurve,
    pub output: OutputTarget,
}

/// A single cue in the flat chain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cue {
    pub number: f64,
    pub name: String,
    pub tasks: Vec<Task>,
    pub status: CueStatus,
    pub follow_mode: FollowMode,
    pub pre_wait_secs: f32,
    pub post_wait_secs: f32,
    pub notes: String,
    /// Whether this cue is logically grouped under its parent (Auto-Follow / Auto-Continue).
    pub indented: bool,
    pub audio_file_name: Option<String>,
}

// ---------------------------------------------------------------------------
// Defaults / helpers

impl Default for Cue {
    fn default() -> Self {
        Self {
            number: 1.0,
            name: "New Cue".into(),
            tasks: Vec::new(),
            status: CueStatus::Ready,
            follow_mode: FollowMode::Manual,
            pre_wait_secs: 0.0,
            post_wait_secs: 0.0,
            notes: String::new(),
            indented: false,
            audio_file_name: None,
        }
    }
}

impl Default for Task {
    fn default() -> Self {
        Self {
            target_name: "BGM".into(),
            property: "Volume".into(),
            target_value: -24.0,
            duration_secs: 3.0,
            curve: FadeCurve::Linear,
            output: OutputTarget {
                name: "Main L/R".into(),
            },
        }
    }
}
