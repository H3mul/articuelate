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

/// The kind of action a cue performs on its target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ActionKind {
    Play,
    Fade,
    Stop,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CueAction {
    pub kind: ActionKind,
    /// Short human label, e.g. "Wind_Loop.wav".
    pub target: String,
    /// Extra detail shown dimmed, e.g. "Vol: -12dB, Loop".
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cue {
    /// Display number, e.g. "1.0".
    pub number: String,
    pub name: String,
    pub follow: FollowMode,
    /// Indent depth for folded Auto-Continue / Auto-Follow children.
    pub depth: usize,
    pub actions: Vec<CueAction>,
}

/// Build a small sample show that resembles the docs/ui.md schematic.
pub fn sample_cues() -> Vec<Cue> {
    fn play(target: &str, detail: &str) -> CueAction {
        CueAction {
            kind: ActionKind::Play,
            target: target.to_string(),
            detail: detail.to_string(),
        }
    }
    fn fade(target: &str, detail: &str) -> CueAction {
        CueAction {
            kind: ActionKind::Fade,
            target: target.to_string(),
            detail: detail.to_string(),
        }
    }

    vec![
        Cue {
            number: "1.0".into(),
            name: "Storm Intro".into(),
            follow: FollowMode::Go,
            depth: 0,
            actions: vec![
                play("Wind_Loop.wav", "Vol: -12dB, Loop"),
                play("Rain_Heavy.wav", "Vol: -8dB, Loop"),
                fade("BGM", "to -24dB, Dur: 3.0s"),
            ],
        },
        Cue {
            number: "2.0".into(),
            name: "Thunder Strike".into(),
            follow: FollowMode::Go,
            depth: 0,
            actions: vec![play("Thunder_Crack.wav", "Vol: 0dB, One-shot")],
        },
        Cue {
            number: "3.0".into(),
            name: "Storm Outro".into(),
            follow: FollowMode::AutoFollow,
            depth: 0,
            actions: vec![
                fade("Wind_Loop.wav", "to -inf, Dur: 4.0s"),
                fade("Rain_Heavy.wav", "to -inf, Dur: 4.0s"),
            ],
        },
        Cue {
            number: "4.0".into(),
            name: "Crowd Murmur".into(),
            follow: FollowMode::AutoContinue,
            depth: 0,
            actions: vec![play("Crowd.wav", "Vol: -18dB, Loop")],
        },
        Cue {
            number: "5.0".into(),
            name: "Scene Change".into(),
            follow: FollowMode::Go,
            depth: 0,
            actions: vec![fade("Crowd.wav", "to -inf, Dur: 2.0s")],
        },
        Cue {
            number: "6.0".into(),
            name: "Door Creak".into(),
            follow: FollowMode::Go,
            depth: 0,
            actions: vec![play("Door_Creak.wav", "Vol: -6dB, One-shot")],
        },
    ]
}

/// Names of the currently-playing media layers shown in the right sidebar.
pub fn sample_active_media() -> Vec<Arc<str>> {
    vec!["CUE 1.0 (Wind_Loop)".into(), "CUE 1.0 (Rain)".into()]
}

/// A named cue-context placeholder.
///
/// This is intentionally unimplemented for now (the async actor / cue-context
/// machinery arrives in a later phase) — it only carries a `name` so the
/// `WorkspaceState` has a stable slot to grow into.
#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub struct CueContext {
    pub name: String,
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
    /// Show-level cue context (unimplemented beyond its name for now).
    #[allow(dead_code)]
    pub context: CueContext,
}

impl WorkspaceState {
    pub fn new(cues: Vec<Cue>) -> Self {
        Self {
            cues,
            context: CueContext {
                name: "default".to_string(),
            },
        }
    }
}

/// State the Execution Thread publishes back to the UI for ingestion.
///
/// Broadcast to the UI through a `tokio::sync::watch` channel; the UI mirrors
/// it into a Floem `RwSignal` so the rest of the reactive UI can react to it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExecutionState {
    /// Linear playhead: the index of the cue the next GO will fire (and that
    /// the most recent GO fired). Advanced by the Execution Thread.
    pub playhead: usize,
    /// Whether a cue sequence is currently running.
    pub running: bool,
}

