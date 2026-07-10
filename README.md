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
| Desktop Shell | **Tauri v2** (OS-native WebView) |
| UI Framework | **React** + **Vite** |
| Component Library | **Mantine UI** (dark, dense, desktop-first) |
| Window Management | **Golden Layout** (dockable, splittable, draggable panels) |
| Serialization | **Serde** (JSON/YAML) |

## UI

A **dockable 3-Panel Workspace** managed by Golden Layout inside the Tauri WebView, with a sticky global toolbar and status bar. Panels can be split, dragged, and torn off (e.g. the Media Panel to a second monitor).

- **Toolbar (top):** Transport controls — Pause, large GO, Back — plus a global search/filter and a red Panic (Stop All) button.
- **Cuelist (top-left):** The flat execution chain, with visual folding (indent) for `Auto-Continue`/`Auto-Follow` children. Vanilla Mantine list for this proof-of-concept (virtualization via `@tanstack/react-virtual` lands later).
- **Detail Panel (bottom-left):** Context-sensitive inspector built from Mantine form controls (NumberInput, Select, Slider) that reacts to the current selection.
- **Active Media Panel (right):** Live engine telemetry — playback layers with progress and volume meters, animated for feel.
- **Status Bar (bottom):** Hardware/driver status and CPU/DSP headroom.

> **Status:** This is a layout proof-of-concept. The React frontend talks to the Rust core over Tauri IPC (`invoke`). When run under a plain Vite dev server (no Tauri), it falls back to static mock data so the layout can be iterated on without compiling Rust.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) (edition 2024) + the Tauri v2 system dependencies for your OS
- [Node.js](https://nodejs.org/) (v20+) and npm
- [Task](https://taskfile.dev/) — the build system

### Run the full Tauri app

```sh
# Install frontend deps (first time)
npm --prefix ui install

# Launch: compiles Rust, opens the WebView window
task tauri:dev

# Build a bundled binary
task tauri:build
```

### Iterate on layout only (no Rust compile)

```sh
task frontend:dev      # vite dev server at http://localhost:5173 (mock data)
```

### Rust core only

```sh
task check             # cargo check
```

## Project Layout

```
articuelate/                 Rust crate + Tauri config (root)
├── src/                     Rust core: cue model, engine, Tauri commands, state
├── capabilities/            Tauri v2 permission capabilities
├── icons/                   App icons (regenerate with `task icons`)
├── ui/                      React + Vite + Mantine frontend
│   └── src/
│       ├── components/      Toolbar, CueList, DetailPanel, MediaPanel, StatusBar
│       ├── layout/          Golden Layout workspace
│       ├── ipc.ts           Tauri invoke + mock fallback
│       ├── store.ts         Shared reactive store (useSyncExternalStore)
│       └── types.ts         Shared TypeScript types
├── scripts/gen_icons.mjs    Placeholder icon generator
├── tauri.conf.json          Tauri v2 configuration
└── Taskfile.yaml            Build tasks
```

## Development Roadmap

| Phase | Focus | Duration |
|-------|-------|----------|
| 1 | State machine & data models (`Vec<Cue>`, Inherit From) | ~1.5 weeks |
| 2 | Audio engine backend (Kira integration, state-squashing) | ~1.5 weeks |
| 3 | Tauri + React/Mantine GUI (Golden Layout, 3-pane workspace) | ~3 weeks |
| 4 | Serialization & hardware resilience (JSON, USB recovery) | ~1.5 weeks |

## License

MIT — see [LICENSE](LICENSE).

## Ethical & Legal

Articuelate is a **clean room** implementation. Built entirely from scratch in Rust/Tauri. No code decompilation, reverse engineering, or proprietary assets are used. The project brand and marketing maintain strict separation from proprietary trademarks.
