Product Definition Document (PDD)

Project: Open-Source Audio Cue System
Target Platforms: Windows, macOS, & Linux (Native Desktop)

1. Executive Summary

This project aims to create a modern, cross-platform, open-source audio cue system designed specifically for professional live show control (theatre, immersive experiences, live events).

Built purely in Rust, it delivers a deterministic, event-driven state machine with the ultra-low memory footprint and extreme speed of a native application.

2. Core Philosophy & Data Architecture

The system embraces a Commands Over Time philosophy, utilizing a Strict 1:1 Flat Chain data model to maximize execution safety.

2.1 The Strict 1:1 Data Model

1 Cue = 1 Action = 1 Targetable Object.

The show is stored and executed as a single, flat Vec<Cue>.

Composition is achieved by chaining cues using Auto-Continue/Auto-Follow triggers.

Targeting: Fades and Stops unambiguously target the explicit ID of the Cue that generated the audio layer.

2.2 The "Inherit From" (Templating) Pattern

Solves cue duplication. Cue 10 can inherit from Cue 1.

Any property modified in Cue 10 is overlaid on top of Cue 1's data via Rust's Option<T>.

At runtime, Cue 10 fully owns its execution state, isolated from the master.

2.3 State-Squashing (Rehearsal Mode)

When jumping out of sequence, skipped cues are evaluated instantly.

Non-temporal tasks are "squashed" and applied immediately via lock-free messages to the audio thread. Temporal constraints are bypassed.

3. Technology Stack (Pure Native Rust)

The application unifies the frontend and backend in a single, lightweight Rust binary, connected by reactive signals and lock-free buffers.

3.1 Core Audio Backend (Real-Time DSP)

Audio Hardware: cpal used directly for low-latency driver access (ASIO/WASAPI/JACK/CoreAudio).

Audio DSP Pipeline: Custom pipeline (fundsp or manual slices) for crosspoint matrix mixing, panning, and sample-accurate triggers.

Thread Communication: Lock-free ringbuffers (ringbuf or rtrb) connect the UI thread to the real-time cpal audio thread.

Serialization: Serde handles project file I/O (JSON/YAML).

3.2 Frontend (Floem)

Framework: Floem. A native, retained-mode, reactive GUI framework.

State Management: Fine-grained reactive signals (RwSignal). UI components surgically update only when their bound data changes.

Aesthetic Reference: Modeled aggressively after the Lapce editor. Features a dark-mode, modular pane system, crisp typography, and virtualized list rendering.

4. User Interface (UI) Specification

The workspace provides a Unified Three-Pane Layout utilizing a dockable, collapsible panel system.

4.1 Pane 1: The Unified Cuelist (Main View)

Renders the flat chain using Floem's virtual_list. Auto-Continue/Follow cues are visually indented under parent "GO" cues.

4.2 Pane 2: Contextual Detail Panel (Bottom Panel)

Reacts to the currently selected cue/task signal. Contains clean, Lapce-style flat sliders, text inputs, and matrix routing toggle-grids. Can be quickly collapsed/hidden.

4.3 Pane 3: Active Media Panel (Right Sidebar)

Displays real-time engine telemetry. High-frequency updates bypass full UI redraws via Floem's surgical signal updates.

5. MVP Development Roadmap

Phase 1: Backend State & DSP (~3 Weeks)

Define Rust data models, establish cpal streams, build the matrix-mixing pipeline, and implement the lock-free ringbuffer bridge.

Phase 2: Floem State & Routing (~1.5 Weeks)

Wrap the Phase 1 structs in Floem RwSignals. Build the application shell and Lapce-style layout system (Splitters, Sidebars).

Phase 3: GUI Implementation (~2.5 Weeks)

Build the virtualized Cuelist, the inspector detail forms, and the active media meters.

Phase 4: Serialization & Polish (~1 Week)

Implement file saving/loading via Serde. Polish custom styling and keyboard shortcuts.
