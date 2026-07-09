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
| Language | **Rust** (edition 2024) |
| Audio Engine | **Kira** (v0.12) & **cpal** |
| UI Framework | **Iced** (v0.14) — Elm-inspired reactive UI |
| Serialization | **Serde** (JSON/YAML) |

## UI

A **Unified Two-Pane Workspace** replaces the traditional multi-tab layout:

- **Left — Unified Cuelist:** The flat execution chain, with visual folding for `Auto-Continue`/`Auto-Follow` groups. Collapse the "tail" to keep the UI clean during live runs.
- **Right — Contextual Detail Panel:** Reactively updates based on selection. Shows explicit overrides and inherited values. The sole editing location keeps physical focus locked to one area.
- **Show Mode:** Locked runtime workspace with a massive GO button, BACK, Panic (Stop All), and an Active Playback Panel with live output meters.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) (edition 2024)
- [Task](https://taskfile.dev/) — the build system

### Build & Run

```sh
# Build for the current platform (auto-detects OS)
task build

# Build with optimizations
task build:release

# Run directly
cargo run
```

### Cross-Compilation

```sh
# Build for Linux (from Linux)
task build:linux

# Build for Windows (cross-compile from Linux)
task build:windows:cross
```

## Development Roadmap

| Phase | Focus | Duration |
|-------|-------|----------|
| 1 | State machine & data models (`Vec<Cue>`, Inherit From) | ~1.5 weeks |
| 2 | Audio engine backend (Kira integration, state-squashing) | ~1.5 weeks |
| 3 | Unified Iced GUI (two-pane layout, visual folding) | ~3 weeks |
| 4 | Serialization & hardware resilience (JSON, USB recovery) | ~1.5 weeks |

## License

MIT — see [LICENSE](LICENSE).

## Ethical & Legal

Articuelate is a **clean room** implementation. Built entirely from scratch in Rust/Iced. No code decompilation, reverse engineering, or proprietary assets are used. All UI paradigms rely on Material design and custom native widgets. The project brand and marketing maintain strict separation from proprietary trademarks.