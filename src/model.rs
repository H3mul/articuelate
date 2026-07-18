//! Placeholder data model for the flat `Vec<Cue>` execution chain.
//!
//! This mirrors the PDD's "Strict 1:1 Flat Chain": one cue = one action on one
//! targetable object. Composition (simultaneous playback) is achieved via
//! `Auto-Continue` / `Auto-Follow` triggers, which for the placeholder simply
//! change the displayed follow badge.

use std::sync::Arc;

/// How a cue advances to the next one in the chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FollowMode {
    /// Operator must press GO.
    Go,
    /// Advances automatically when the previous cue's audio finishes.
    AutoFollow,
    /// Advances immediately after the previous cue fires.
    AutoContinue,
}

impl FollowMode {
    pub fn badge(self) -> &'static str {
        match self {
            FollowMode::Go => "GO",
            FollowMode::AutoFollow => "AUTO-FOLLOW",
            FollowMode::AutoContinue => "AUTO-CONTINUE",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cue {
    pub name: String,
    pub follow: FollowMode,
}

/// Build a small sample show that resembles the docs/ui.md schematic.
pub fn sample_cues() -> Vec<Cue> {
    vec![
        Cue {
            name: "Storm Intro".into(),
            follow: FollowMode::Go,
        },
        Cue {
            name: "Thunder Strike".into(),
            follow: FollowMode::AutoContinue,
        },
        Cue {
            name: "Storm Outro".into(),
            follow: FollowMode::AutoFollow,
        },
        Cue {
            name: "Storm Heavy".into(),
            follow: FollowMode::Go,
        },
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
#[derive(Clone, Debug, Default)]
pub struct WorkspaceState {
    /// The flat cue chain. For now a simple `Vec`; the execution thread reads
    /// `cues` directly to resolve the current playhead target.
    pub cues: Vec<Cue>,
}

impl WorkspaceState {
    pub fn new(cues: Vec<Cue>) -> Self {
        Self { cues }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlayheadState {
    #[default]
    Stopped,
    Playing(usize),
}

/// State the Execution Thread publishes back to the UI for ingestion.
///
/// Broadcast to the UI through a `tokio::sync::watch` channel; the UI mirrors
/// it into a Floem `RwSignal` so the rest of the reactive UI can react to it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExecutionState {
    /// Linear playhead: the index of the cue the next GO will fire (and that
    /// the most recent GO fired). Advanced by the Execution Thread.
    pub playhead: PlayheadState,
}
