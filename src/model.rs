//! Placeholder data model for the flat `Vec<Cue>` execution chain.
//!
//! This mirrors the PDD's "Strict 1:1 Flat Chain": one cue = one action on one
//! targetable object. Composition (simultaneous playback) is achieved via
//! `Auto-Continue` / `Auto-Follow` triggers, which for the placeholder simply
//! change the displayed follow badge.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

/// How a cue advances to the next one in the chain.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerMode {
    /// Cue triggers when the playhead reaches it
    #[default]
    Playhead,
    /// Cue triggers together with a target cue
    WithCue,
    /// Cue triggers after the target cue (when it finishes)
    AfterCue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trigger {
    pub mode: TriggerMode,
    pub target: Option<CueId>,
}

impl Trigger {
    pub fn default() -> Self {
        Self {
            mode: TriggerMode::default(),
            target: None,
        }
    }
    pub fn badge(self) -> &'static str {
        match self.mode {
            TriggerMode::Playhead => "PLAYHEAD",
            TriggerMode::WithCue => "WITH-CUE",
            TriggerMode::AfterCue => "AFTER-CUE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CueId(Uuid);

impl CueId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cue {
    pub id: CueId,
    pub name: String,
    pub trigger: Trigger,
}

impl Cue {
    pub fn new(name: impl Into<String>) -> Self {
        return Self::with_trigger(name, Trigger::default());
    }
    pub fn with_trigger(name: impl Into<String>, trigger: Trigger) -> Self {
        Self {
            id: CueId::new(),
            name: name.into(),
            trigger: trigger,
        }
    }
}

/// Build a small sample show that resembles the docs/ui.md schematic.
pub fn sample_cues() -> Vec<Cue> {
    vec![
        Cue::new("Storm Intro"),
        Cue::with_trigger(
            "Thunder Strike",
            Trigger {
                mode: TriggerMode::WithCue,
                target: None,
            },
        ),
        Cue::with_trigger(
            "Storm Outro",
            Trigger {
                mode: TriggerMode::AfterCue,
                target: None,
            },
        ),
        Cue::new("Storm Heavy"),
    ]
}

/// Names of the currently-playing media layers shown in the right sidebar.
pub fn sample_active_media() -> Vec<Arc<str>> {
    vec!["CUE 1.0 (Wind_Loop)".into(), "CUE 1.0 (Rain)".into()]
}

/// The shared, read-only workspace snapshot the Execution Thread queries.
///
/// Held behind an `Arc` so the UI can cheaply clone the handle into the
/// Execution Thread, which reads it lock-free on every event (e.g. GO) without
/// ever taking a lock or copying the cue list.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Cuelist {
    /// The flat cue chain. For now a simple `Vec`; the execution thread reads
    /// `cues` directly to resolve the current playhead target.
    order: Vec<CueId>,
    /// The mapping of cue IDs to cue data.
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

    /// Returns an iterator over all cues in the cue list.
    pub fn iter(&self) -> impl Iterator<Item = &Arc<Cue>> {
        self.order.iter().filter_map(|id| self.cues.get(id))
    }

    /// Returns an iterator over the cues after the given cue ID, if it exists.
    pub fn iter_after(&self, id: CueId) -> Option<impl Iterator<Item = &Arc<Cue>>> {
        self.order.iter().position(|&x| x == id).map(|index| {
            let start_index = index + 1; // Return next element

            self.order[start_index..]
                .iter()
                .filter_map(|id| self.cues.get(id))
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceState {
    pub cuelist: Arc<Cuelist>,
}

impl WorkspaceState {
    pub fn sample() -> Self {
        Self {
            cuelist: Arc::new(Cuelist::new(sample_cues())),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Playhead {
    #[default]
    Stopped,
    Playing(CueId),
}

/// State the Execution Thread publishes back to the UI for ingestion.
///
/// Broadcast to the UI through a `tokio::sync::watch` channel; the UI mirrors
/// it into a Floem `RwSignal` so the rest of the reactive UI can react to it.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExecutionState {
    /// Linear playhead: the index of the cue the next GO will fire (and that
    /// the most recent GO fired). Advanced by the Execution Thread.
    pub playhead: Playhead,
    pub active_cues: HashSet<CueId>,
}
