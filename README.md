# Articuelate

**A modern, open-source audio cue system for professional live show control.**

Articuelate is a cross-platform (Windows & Linux) alternative to QLab, purpose-built for theatre, immersive experiences, and live events. Unlike DAWs that rely on static timelines, Articuelate is a **deterministic, event-driven state machine** — a highly reliable, operator-centric runtime environment focused on safe, predictable audio execution and quick tech-rehearsal recovery.

## Philosophy

**Commands Over Time.** Every show is stored as a single, flat `Vec<Cue>` — a **Strict 1:1 Flat Chain** where one cue equals one action on one targetable object. Composition (simultaneous playback) is achieved via `Auto-Continue` or `Auto-Follow` triggers, and targeting is unambiguous: fades and stops always reference the exact cue that generated the audio layer.

### Key Concepts

- **Inherit From (Templating):** Cues can inherit data from a master cue, with property-level overrides via `Option<T>`. At runtime, each cue fully owns its execution state — no shared audio handles, no targeting conflicts.
- **State-Squashing (Rehearsal Mode):** Jumping out of sequence evaluates skipped cues instantly. Non-temporal tasks apply immediately; temporal constraints are squashed to `0.0s` duration. Active audio streams continue naturally.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | **Rust** (edition 2021) |
| GUI Framework | **Floem** (v0.2) — immediate-mode reactive widgets, native (no WebView) |
| Reactive Primitives | Floem `RwSignal` / `create_memo` / `create_effect` + `im` (immutable vectors) |
| Windowing | Floem's built-in Winit-based windowing |
| Audio Engine | *Planned* — **Kira** (v0.12) & **cpal** (not yet wired; placeholder data only) |
| Serialization | *Planned* — **Serde** (JSON/YAML) |

> The earlier Tauri v2 + React + Mantine + Golden Layout stack was replaced by a
> single native Rust crate using Floem. See `docs/pdd.md` for the rationale behind
> choosing a Rust-native GUI.

## UI

A **Lapce-inspired 3-Pane Workspace** rendered natively with Floem — a dark, dense,
editor-style layout with a sticky global toolbar and status bar. Panels collapse via
toggles (draggable splitters land when Floem ships them).

- **Toolbar (top):** Transport controls — Pause, large GO, Back — plus a global search/filter, panel toggles, a red Panic (Stop All) button, and a live cue readout (`CUE n/N`).
- **Cuelist (top-left):** The flat execution chain, virtualized via Floem `virtual_list`, with visual indenting for `Auto-Continue`/`Auto-Follow` children, active-cue highlight, and action glyphs (▶ play / ◢ fade / ■ stop). Click to select; GO advances the active cue.
- **Detail Panel (bottom-left):** Context-sensitive inspector with three tabs — **General**, **Audio Routing** (2×4 toggle matrix), and **Fades** (duration/volume sliders) — bound to the selected cue. Toggle with **Ctrl+J**.
- **Active Media Panel (right):** Live level meters with fine-grained per-channel `RwSignal`s animated by a timer (`exec_after`), repainting only the bound meter node. Collapsible.
- **Status Bar (bottom):** Static status strings.

> **Status:** This is a layout placeholder. All data is mocked in `src/model.rs`
> (`sample_cues`, `sample_active_media`); there is no real audio backend yet. The UI
> demonstrates the full three-pane skeleton and reactivity model from `docs/ui.md`.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) (edition 2021) — toolchain only; **no Node.js, npm, or Tauri required**.

### Build & run

```sh
# Compile and launch the native window
cargo run

# Build a release binary
cargo build --release

# Type/lint check without a full build
cargo check
```

### Keyboard shortcuts

- **Ctrl+J** — toggle the bottom Detail Panel.

## Project Layout

```
articuelate/                 Repo root = single binary crate
├── Cargo.toml               Package "articuelate" (floem, fastrand, im)
├── src/
│   ├── main.rs              App assembly, window config, Ctrl+J shortcut, status bar
│   ├── theme.rs             Lapce-style dark palette (Color constants) + MONO font
│   ├── model.rs             Placeholder data: Cue / CueAction / FollowMode + samples
│   ├── toolbar.rs           Top transport toolbar (Pause/GO/Back/Search/Panic/toggles)
│   ├── cuelist.rs           Virtualized cuelist (virtual_list) + cue/action rows
│   ├── detail.rs            Bottom inspector (General / Routing / Fades tabs)
│   └── media.rs             Right "Active Media" sidebar with animated level meters
├── docs/
│   ├── pdd.md               Product Design Document (architecture rationale)
│   └── ui.md                UI layout schematic (3-pane Lapce-style workspace)
└── AGENTS.md                Contributor notes
```

## Development Roadmap

| Phase | Focus | Duration |
|-------|-------|----------|
| 1 | State machine & data models (`Vec<Cue>`, Inherit From) | ~1.5 weeks |
| 2 | Audio engine backend (Kira integration, state-squashing) | ~1.5 weeks |
| 3 | Native Floem GUI (Lapce-style 3-pane workspace, draggable splitters) | ~3 weeks |
| 4 | Serialization & hardware resilience (JSON, USB recovery) | ~1.5 weeks |

## License

MIT — see [LICENSE](LICENSE).

## Ethical & Legal

Articuelate is a **clean room** implementation. Built entirely from scratch in Rust with the native Floem GUI (previously Tauri). No code decompilation, reverse engineering, or proprietary assets are used. The project brand and marketing maintain strict separation from proprietary trademarks.
