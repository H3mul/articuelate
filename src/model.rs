//! Data model for the Articuelate cue system.
//!
//! Mirrors the prototype's rich cue model: flat chain of cues with support for
//! music, control, OSC, group, and fade types. The UI reads from an
//! `Arc<Cuelist>` via reactive signals; the execution engine reads from an
//! `ArcSwap<WorkspaceState>`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

// ─── Core Enums ─────────────────────────────────────────────────────────

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
    Control {
        target: String,
        value: String,
    },
    Osc {
        task: String,
        host: String,
        port: u16,
    },
    Group,
    Fade {
        target: String,
        property: String,
        target_value: f64,
        duration_sec: f64,
    },
}

/// Visual state of a cue for the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CueState {
    #[default]
    Idle,
    Standby,
    Running,
}

/// How a cue advances to the next one in the chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TriggerCondition {
    /// Cue triggers when the playhead reaches it
    #[default]
    Playhead,
    /// Cue triggers together with a target cue
    WithCue,
    /// Cue triggers after the target cue (when it finishes)
    AfterCue,
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

/// Fade state for the active cue row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FadeState {
    #[default]
    None,
    In,
    Out,
}

// ─── Core Types ─────────────────────────────────────────────────────────

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

/// A single cue in the show file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cue {
    pub id: CueId,
    pub number: String,
    pub name: String,
    pub notes: String,
    /// Display string for the target/media summary in the list (e.g. "audio · file.wav")
    pub target: String,
    pub kind: CueKind,
    pub trigger_condition: TriggerCondition,
    /// Cue number this trigger references (for WithCue / AfterCue)
    pub trigger_target: Option<String>,
    pub pre_wait: String,
    pub duration: String,
    pub post_wait: String,
    /// Indentation depth for sub-cues (0 = top-level)
    pub depth: usize,
    pub state: CueState,
    pub color: CueColor,
    /// Media file path — only meaningful for Audio cues
    pub media_file: Option<String>,
    /// Target output volume 0-1 — only meaningful for Audio cues
    pub volume: f64,
    /// Progress 0-1, only relevant while running
    pub progress: Option<f64>,
    /// Pre-delay progress 0-1 while the cue is counting into playback
    pub pre_progress: Option<f64>,
    /// Post-delay progress 0-1 while the cue is winding down
    pub post_progress: Option<f64>,
}

impl Cue {
    pub fn new_music(
        number: impl Into<String>,
        name: impl Into<String>,
        notes: impl Into<String>,
        file_path: PathBuf,
        volume: f64,
        duration: impl Into<String>,
    ) -> Self {
        Self {
            id: CueId::new(),
            number: number.into(),
            name: name.into(),
            notes: notes.into(),
            target: format!("audio · {}", file_path.file_name().unwrap_or_default().to_string_lossy()),
            kind: CueKind::Audio {
                file_path,
                volume,
                looping: false,
                fade_in_sec: 0.0,
                fade_out_sec: 0.0,
            },
            trigger_condition: TriggerCondition::Playhead,
            trigger_target: None,
            pre_wait: "00:00".into(),
            duration: duration.into(),
            post_wait: "00:00".into(),
            depth: 0,
            state: CueState::Idle,
            color: CueColor::None,
            media_file: None,
            volume,
            progress: None,
            pre_progress: None,
            post_progress: None,
        }
    }
}

// ─── Cuelist ────────────────────────────────────────────────────────────

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

// ─── Workspace ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceState {
    pub cuelist: Arc<Cuelist>,
}

// ─── Execution State ────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Playhead {
    #[default]
    Stopped,
    Playing(CueId),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExecutionState {
    pub playhead: Playhead,
    /// The currently-running cue IDs (for the runtime sidebar)
    pub running_cues: Vec<CueId>,
}

// ─── Active Cue (Runtime Sidebar) ───────────────────────────────────────

/// A cue that has been triggered and is still playing back.
#[derive(Debug, Clone)]
pub struct ActiveCue {
    pub id: CueId,
    pub number: String,
    pub name: String,
    pub file: String,
    pub elapsed: f64,
    pub remaining: f64,
    pub duration: f64,
    pub progress: f64,
    pub color: CueColor,
    pub level: f64,
}

// ─── Sample Data ────────────────────────────────────────────────────────

/// Build a sample show matching the prototype's CUE_DATA.
pub fn sample_cues() -> Vec<Cue> {
    vec![
        Cue {
            id: CueId::new(),
            number: "1".into(),
            name: "Preshow Music".into(),
            notes: "House playlist — start 30 min before doors.".into(),
            target: "group · 2 cues".into(),
            kind: CueKind::Group,
            trigger_condition: TriggerCondition::Playhead,
            trigger_target: None,
            pre_wait: "00:00".into(),
            duration: "12:00".into(),
            post_wait: "00:00".into(),
            depth: 0,
            state: CueState::Idle,
            color: CueColor::Orange,
            media_file: None,
            volume: 0.7,
            progress: None,
            pre_progress: None,
            post_progress: None,
        },
        Cue {
            id: CueId::new(),
            number: "1.1".into(),
            name: "Volume Down".into(),
            notes: "Dim house to 50% as music settles.".into(),
            target: "universe 1 · ch 12".into(),
            kind: CueKind::Control {
                target: "universe 1".into(),
                value: "ch 12".into(),
            },
            trigger_condition: TriggerCondition::WithCue,
            trigger_target: Some("1".into()),
            pre_wait: "00:02".into(),
            duration: "00:03".into(),
            post_wait: "00:00".into(),
            depth: 1,
            state: CueState::Idle,
            color: CueColor::None,
            media_file: None,
            volume: 0.0,
            progress: None,
            pre_progress: None,
            post_progress: None,
        },
        Cue {
            id: CueId::new(),
            number: "1.2".into(),
            name: "Send OSC · Projector On".into(),
            notes: "/projector/power 1".into(),
            target: "osc · 10.0.0.42".into(),
            kind: CueKind::Osc {
                task: "/projector/power 1".into(),
                host: "10.0.0.42".into(),
                port: 3333,
            },
            trigger_condition: TriggerCondition::AfterCue,
            trigger_target: Some("1.1".into()),
            pre_wait: "00:00".into(),
            duration: "00:00".into(),
            post_wait: "00:00".into(),
            depth: 1,
            state: CueState::Idle,
            color: CueColor::None,
            media_file: None,
            volume: 0.0,
            progress: None,
            pre_progress: None,
            post_progress: None,
        },
        Cue {
            id: CueId::new(),
            number: "2".into(),
            name: "Act 1 Intro".into(),
            notes: "Fade in under the announcement.".into(),
            target: "audio · act1_intro.wav".into(),
            kind: CueKind::Audio {
                file_path: PathBuf::from("act1_intro.wav"),
                volume: 0.85,
                looping: false,
                fade_in_sec: 2.0,
                fade_out_sec: 3.0,
            },
            trigger_condition: TriggerCondition::Playhead,
            trigger_target: None,
            pre_wait: "00:00".into(),
            duration: "00:45".into(),
            post_wait: "00:02".into(),
            depth: 0,
            state: CueState::Idle,
            color: CueColor::Blue,
            media_file: Some("act1_intro.wav".into()),
            volume: 0.85,
            progress: None,
            pre_progress: None,
            post_progress: None,
        },
        Cue {
            id: CueId::new(),
            number: "3".into(),
            name: "Thunderclap".into(),
            notes: "Hard hit on the lightning flash.".into(),
            target: "audio · thunder_01.wav".into(),
            kind: CueKind::Audio {
                file_path: PathBuf::from("thunder_01.wav"),
                volume: 1.0,
                looping: false,
                fade_in_sec: 0.0,
                fade_out_sec: 0.0,
            },
            trigger_condition: TriggerCondition::Playhead,
            trigger_target: None,
            pre_wait: "00:03".into(),
            duration: "00:06".into(),
            post_wait: "00:00".into(),
            depth: 0,
            state: CueState::Standby,
            color: CueColor::None,
            media_file: Some("thunder_01.wav".into()),
            volume: 1.0,
            progress: None,
            pre_progress: Some(0.6),
            post_progress: None,
        },
        Cue {
            id: CueId::new(),
            number: "4".into(),
            name: "Rain Ambience".into(),
            notes: "Loop through the storm scene.".into(),
            target: "audio · rain_loop.wav".into(),
            kind: CueKind::Audio {
                file_path: PathBuf::from("rain_loop.wav"),
                volume: 0.62,
                looping: true,
                fade_in_sec: 0.0,
                fade_out_sec: 0.0,
            },
            trigger_condition: TriggerCondition::Playhead,
            trigger_target: None,
            pre_wait: "00:00".into(),
            duration: "04:30".into(),
            post_wait: "00:00".into(),
            depth: 0,
            state: CueState::Running,
            color: CueColor::Green,
            media_file: Some("rain_loop.wav".into()),
            volume: 0.62,
            progress: Some(0.42),
            pre_progress: None,
            post_progress: None,
        },
        Cue {
            id: CueId::new(),
            number: "4.1".into(),
            name: "Distant Rumble".into(),
            notes: "Under-bed rumble, triggered with rain.".into(),
            target: "audio · rumble_lo.wav".into(),
            kind: CueKind::Audio {
                file_path: PathBuf::from("rumble_lo.wav"),
                volume: 0.4,
                looping: false,
                fade_in_sec: 0.0,
                fade_out_sec: 0.0,
            },
            trigger_condition: TriggerCondition::WithCue,
            trigger_target: Some("4".into()),
            pre_wait: "00:00".into(),
            duration: "06:00".into(),
            post_wait: "00:00".into(),
            depth: 1,
            state: CueState::Running,
            color: CueColor::None,
            media_file: Some("rumble_lo.wav".into()),
            volume: 0.4,
            progress: Some(0.31),
            pre_progress: None,
            post_progress: None,
        },
    ]
}

/// Sample active cues for the runtime sidebar.
pub fn sample_active_cues() -> Vec<ActiveCue> {
    vec![
        ActiveCue {
            id: CueId::new(),
            number: "4".into(),
            name: "Rain Ambience".into(),
            file: "rain_loop.wav".into(),
            elapsed: 113.0,
            remaining: 157.0,
            duration: 270.0,
            progress: 0.42,
            color: CueColor::Green,
            level: 0.54,
        },
        ActiveCue {
            id: CueId::new(),
            number: "4.1".into(),
            name: "Distant Rumble".into(),
            file: "rumble_lo.wav".into(),
            elapsed: 111.0,
            remaining: 249.0,
            duration: 360.0,
            progress: 0.31,
            color: CueColor::None,
            level: 0.33,
        },
        ActiveCue {
            id: CueId::new(),
            number: "2".into(),
            name: "Act 1 Intro".into(),
            file: "act1_intro.wav".into(),
            elapsed: 38.0,
            remaining: 7.0,
            duration: 45.0,
            progress: 0.84,
            color: CueColor::Blue,
            level: 0.78,
        },
    ]
}

/// The cue currently on the playhead and the one queued next.
pub struct ConductorCues {
    pub current_number: &'static str,
    pub current_name: &'static str,
    pub next_number: &'static str,
    pub next_name: &'static str,
    pub next_notes: &'static str,
}

pub const CONDUCTOR_CUES: ConductorCues = ConductorCues {
    current_number: "3",
    current_name: "Thunderclap",
    next_number: "4",
    next_name: "Rain Ambience",
    next_notes: "Loop through the storm scene.",
};