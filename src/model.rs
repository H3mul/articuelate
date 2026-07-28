//! Data model for the Articuelate cue system.
//!
//! Mirrors the prototype's rich cue model: flat chain of cues with support for
//! music, control, OSC, group, and fade types. The UI reads from an
//! `Arc<Cuelist>` via reactive signals; the execution engine reads from an
//! `ArcSwap<WorkspaceState>`.

use floem::reactive::{Memo, RwSignal, SignalGet, create_memo};
use serde::{Deserialize, Serialize};
use serde_with::{DurationMicroSeconds, serde_as};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::audio::{AtomicCueMetrics, AudioTelemetry};

// ─── Foundation Types ───────────────────────────────────────────────────
// Lowest-level types with no (or minimal) local dependencies.

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CueId(Uuid);

impl CueId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl std::fmt::Display for CueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Highlight colour swatch for a cue row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CueColor {
    #[default]
    None,
    Red,
    Orange,
    Green,
    Blue,
    Purple,
}

impl CueColor {
    /// Human-readable label matching the prototype.
    pub fn as_str(&self) -> &'static str {
        match self {
            CueColor::None => "none",
            CueColor::Red => "red",
            CueColor::Orange => "orange",
            CueColor::Green => "green",
            CueColor::Blue => "blue",
            CueColor::Purple => "purple",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TriggerCondition {
    /// Cue triggers when the playhead reaches it
    #[default]
    Playhead,
    /// Cue triggers together with a target cue
    WithCue { target: CueId },
    /// Cue triggers after the target cue (when it finishes)
    AfterCue { target: CueId },
}

impl TriggerCondition {
    /// Discriminant tag for UI selectors that don't need the target.
    pub fn discriminant_tag(&self) -> u8 {
        match self {
            TriggerCondition::Playhead => 0,
            TriggerCondition::WithCue { .. } => 1,
            TriggerCondition::AfterCue { .. } => 2,
        }
    }
}

// ─── Core Cue Model ─────────────────────────────────────────────────────
// The cue itself and the collection that holds cues.

/// Kind of cue — determines which tab/target fields are relevant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CueKind {
    Audio {
        file_path: PathBuf,
        volume: f64,
        looping: bool,
        fade_in_sec: f64,
        fade_out_sec: f64,
    },
    Group,
    Control {
        target: String,
        value: String,
    },
    Osc {
        task: String,
        host: String,
        port: u16,
    },
    Fade {
        target: String,
        property: String,
        target_value: f64,
        duration_sec: f64,
    },
}

impl CueKind {
    /// Human-readable target label for the cuelist "TARGET" column.
    pub fn target_label(&self) -> String {
        match self {
            CueKind::Audio { file_path, .. } => {
                format!(
                    "audio · {}",
                    file_path.file_name().unwrap_or_default().to_string_lossy()
                )
            }
            CueKind::Group => "group".to_string(),
            CueKind::Control { target, value } => format!("control · {}:{}", target, value),
            CueKind::Osc { host, port, .. } => format!("osc · {}:{}", host, port),
            CueKind::Fade {
                target, property, ..
            } => format!("fade · {}:{}", target, property),
        }
    }

    /// The media file path, if this is an audio cue.
    pub fn media_file(&self) -> Option<&PathBuf> {
        match self {
            CueKind::Audio { file_path, .. } => Some(file_path),
            _ => None,
        }
    }
}

/// A single cue in the show file.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cue {
    pub id: CueId,
    pub name: String,
    pub notes: String,
    pub kind: CueKind,
    pub trigger_condition: TriggerCondition,
    #[serde_as(as = "DurationMicroSeconds<u64>")]
    pub pre_wait: Duration,
    #[serde_as(as = "DurationMicroSeconds<u64>")]
    pub post_wait: Duration,
    pub color: CueColor,
}

/// The flat cue chain — a strict ordering of cues plus a key-value map.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Cuelist {
    order: Vec<CueId>,
    cues: HashMap<CueId, Arc<Cue>>,
}

impl Cuelist {
    pub fn new(cues: Vec<Cue>) -> Self {
        let mut list = Self::default();
        list.add_cues(cues.into_iter());
        list
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    pub fn add_cue(&mut self, cue: Cue) {
        let id = cue.id;
        self.cues.insert(id, Arc::new(cue));
        self.order.push(id);
    }

    pub fn add_cues(&mut self, cues: impl Iterator<Item = Cue>) {
        for cue in cues {
            self.add_cue(cue);
        }
    }

    pub fn get_cue(&self, id: CueId) -> Option<&Arc<Cue>> {
        self.cues.get(&id)
    }

    /// Returns an iterator over all cues in order.
    pub fn iter(&self) -> impl Iterator<Item = &Arc<Cue>> {
        self.order.iter().filter_map(|id| self.cues.get(id))
    }

    /// Returns an iterator over the cues after the given cue ID, if it exists.
    pub fn iter_after(&self, id: CueId) -> Option<impl Iterator<Item = &Arc<Cue>>> {
        self.order.iter().position(|&x| x == id).map(|index| {
            self.order[index + 1..]
                .iter()
                .filter_map(|id| self.cues.get(id))
        })
    }

    /// Find the position (1-based) of a cue by ID.
    pub fn position_of(&self, id: CueId) -> Option<usize> {
        self.order.iter().position(|&x| x == id).map(|i| i + 1)
    }
}

// ─── Runtime / Execution ────────────────────────────────────────────────
// Ephemeral playback state that lives outside the persisted show file.

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PlaybackStatus {
    #[default]
    Idle,
    Standby,
    Playing,
    Paused,
    Error,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CueExecutionState {
    pub status: PlaybackStatus,
    pub pre_wait_elapsed: Duration,
    pub post_wait_elapsed: Duration,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Playhead {
    #[default]
    Stopped,
    Playing(CueId),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExecutionState {
    pub playhead: Playhead,
    pub cue_execution_state: im::HashMap<CueId, CueExecutionState>,
}

// ─── Application State ──────────────────────────────────────────────────
// Persistent + transient state wired together for the UI and engine.

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceState {
    pub cuelist: Arc<Cuelist>,
}

#[derive(Clone)]
pub struct AppState {
    // The source of truth for order and configuration
    pub workspace: RwSignal<WorkspaceState>,

    // The ephemeral runtime data
    pub execution: RwSignal<ExecutionState>,

    // Audio telemetry for monitoring audio levels and playback status
    pub audio_telemetry: Arc<AudioTelemetry>,

    // UI-specific selections
    pub selected_cue: RwSignal<Option<CueId>>,
}

impl AppState {
    /// Convenience helper to assemble a transient view model for a cue
    pub fn cue_state(&self, id: CueId) -> Option<TransientCueState> {
        TransientCueState::new(id, self.clone())
    }
}

/// A transient type to join workspace and runtime state for a cue for UI rendering.
#[derive(Clone)]
pub struct TransientCueState {
    pub id: CueId,
    pub workspace: Memo<Arc<Cue>>,
    pub execution: Memo<CueExecutionState>,
    pub audio_telemetry: Arc<AudioTelemetry>,
}

impl PartialEq for TransientCueState {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl TransientCueState {
    pub fn new(id: CueId, app_state: AppState) -> Option<Self> {
        let initial_cue = app_state.workspace.get().cuelist.get_cue(id)?.clone();

        Some(Self {
            id,
            workspace: create_memo(move |_| {
                app_state
                    .workspace
                    .get()
                    .cuelist
                    .get_cue(id)
                    .cloned()
                    .unwrap_or_else(|| initial_cue.clone())
            }),
            execution: create_memo(move |_| {
                app_state
                    .execution
                    .get()
                    .cue_execution_state
                    .get(&id)
                    .cloned()
                    .unwrap_or_default()
            }),
            audio_telemetry: app_state.audio_telemetry.clone(),
        })
    }

    pub fn read_audio_telemetry(&self) -> Arc<AtomicCueMetrics> {
        self.audio_telemetry.clone().metrics_for(self.id)
    }
}

// ─── Sample Data ────────────────────────────────────────────────────────

/// Build a sample show file with a variety of cue kinds.
pub fn sample_cues() -> Vec<Cue> {
    vec![
        Cue {
            id: CueId::new(),
            name: "Preshow Music".into(),
            notes: "House playlist — start 30 min before doors.".into(),
            kind: CueKind::Audio {
                file_path: PathBuf::from("preshow_loop.wav"),
                volume: 0.7,
                looping: true,
                fade_in_sec: 3.0,
                fade_out_sec: 5.0,
            },
            trigger_condition: TriggerCondition::Playhead,
            pre_wait: Duration::ZERO,
            post_wait: Duration::ZERO,
            color: CueColor::Orange,
        },
        Cue {
            id: CueId::new(),
            name: "Act 1 Intro".into(),
            notes: "Fade in under the announcement.".into(),
            kind: CueKind::Audio {
                file_path: PathBuf::from("act1_intro.wav"),
                volume: 0.85,
                looping: false,
                fade_in_sec: 2.0,
                fade_out_sec: 3.0,
            },
            trigger_condition: TriggerCondition::Playhead,
            pre_wait: Duration::ZERO,
            post_wait: Duration::from_secs(2),
            color: CueColor::Blue,
        },
        Cue {
            id: CueId::new(),
            name: "Thunderclap".into(),
            notes: "Hard hit on the lightning flash.".into(),
            kind: CueKind::Audio {
                file_path: PathBuf::from("thunder_01.wav"),
                volume: 1.0,
                looping: false,
                fade_in_sec: 0.0,
                fade_out_sec: 0.0,
            },
            trigger_condition: TriggerCondition::Playhead,
            pre_wait: Duration::from_secs(3),
            post_wait: Duration::ZERO,
            color: CueColor::None,
        },
        Cue {
            id: CueId::new(),
            name: "Rain Ambience".into(),
            notes: "Loop through the storm scene.".into(),
            kind: CueKind::Audio {
                file_path: PathBuf::from("rain_loop.wav"),
                volume: 0.62,
                looping: true,
                fade_in_sec: 0.0,
                fade_out_sec: 0.0,
            },
            trigger_condition: TriggerCondition::Playhead,
            pre_wait: Duration::ZERO,
            post_wait: Duration::ZERO,
            color: CueColor::Green,
        },
        Cue {
            id: CueId::new(),
            name: "Distant Rumble".into(),
            notes: "Under-bed rumble, triggered with rain.".into(),
            kind: CueKind::Audio {
                file_path: PathBuf::from("rumble_lo.wav"),
                volume: 0.4,
                looping: false,
                fade_in_sec: 0.0,
                fade_out_sec: 0.0,
            },
            trigger_condition: TriggerCondition::Playhead,
            pre_wait: Duration::ZERO,
            post_wait: Duration::ZERO,
            color: CueColor::None,
        },
        Cue {
            id: CueId::new(),
            name: "Curtain Call".into(),
            notes: "Final bows and exit music.".into(),
            kind: CueKind::Audio {
                file_path: PathBuf::from("curtain_call.wav"),
                volume: 0.9,
                looping: false,
                fade_in_sec: 1.0,
                fade_out_sec: 4.0,
            },
            trigger_condition: TriggerCondition::Playhead,
            pre_wait: Duration::from_secs(5),
            post_wait: Duration::ZERO,
            color: CueColor::Purple,
        },
    ]
}

/// Build a sample execution state with some cues marked as playing / standby.
pub fn sample_execution_state(cuelist: &Cuelist) -> ExecutionState {
    let mut cue_execution_state: im::HashMap<CueId, CueExecutionState> = im::HashMap::new();

    // Mark the first few cues with interesting states
    for (i, cue) in cuelist.iter().enumerate() {
        let status = match i {
            0 => PlaybackStatus::Playing, // Preshow Music — running
            1 => PlaybackStatus::Standby, // Act 1 Intro — standby
            2 => PlaybackStatus::Idle,    // Thunderclap — idle
            3 => PlaybackStatus::Playing, // Rain Ambience — running
            4 => PlaybackStatus::Playing, // Distant Rumble — running
            _ => PlaybackStatus::Idle,
        };
        cue_execution_state.insert(
            cue.id,
            CueExecutionState {
                status,
                pre_wait_elapsed: Duration::ZERO,
                post_wait_elapsed: Duration::ZERO,
            },
        );
    }

    // Playhead on Thunderclap (index 2)
    let playhead = cuelist
        .iter()
        .nth(2)
        .map(|cue| Playhead::Playing(cue.id))
        .unwrap_or(Playhead::Stopped);

    ExecutionState {
        playhead,
        cue_execution_state,
    }
}
