Product Definition Document (PDD)

Project: Open-Source Audio Cue System
Target Platforms: Windows & Linux (Desktop)

1. Executive Summary

This project aims to create a modern, cross-platform, open-source audio cue system designed specifically for professional live show control (theatre, immersive experiences, live events).

Unlike DAWs which rely on static timelines, this system is a deterministic, event-driven state machine. It provides a highly reliable, operator-centric runtime environment focused on safe audio execution, quick recovery, and intuitive show design.

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

3. Technology Stack (Tauri + Rust Backend)

The application strictly separates the high-performance audio engine from the flexible, component-rich UI.

3.1 Backend: Rust (Core Logic & DSP)

Audio Hardware: cpal used directly for low-latency driver access (ASIO/WASAPI/JACK).

Audio DSP Pipeline: Custom pipeline (via fundsp or manual slices) for crosspoint matrix mixing and sample-accurate triggers.

Thread Communication: Lock-free ringbuffers (ringbuf) connect the async IPC command handlers to the real-time cpal audio thread.

Serialization: Serde handles project file I/O.

3.2 Frontend: Tauri v2 + React (UI Layer)

Framework: React inside Tauri's OS-native WebView.

Component Library: Mantine UI. Provides accessible, dense, dark-themed inputs, sliders, and standard desktop UI paradigms out of the box.

Window Management: Golden Layout. Provides a robust, dockable, resizable paneling system for the multi-pane workspace.

Specialty Components: Leverage React ecosystem libraries (e.g., react-rotary-knob, wavesurfer.js) for audio-specific visualizations.

4. User Interface (UI) Specification

Rendered via React and managed by Golden Layout, the workspace provides a Unified Three-Pane Layout.

4.1 Pane 1: The Unified Cuelist (Left)

Renders the flat chain. Auto-Continue/Follow cues are visually indented under parent "GO" cues.

Utilizes a virtualized list (e.g., @tanstack/react-virtual) to handle thousands of cues without DOM bloat.

4.2 Pane 2: Contextual Detail Panel (Bottom Left)

Reacts to the current selection. Contains Mantine's robust numeric steppers, combo boxes, and custom matrix-routing grids.

4.3 Pane 3: Active Media Panel (Right)

Displays real-time engine telemetry.

IPC Strategy: To avoid IPC bottlenecking, peak meter telemetry is throttled to 30fps and batched before sending to the React frontend.

5. MVP Development Roadmap

Phase 1: Backend State & DSP (~3 Weeks)

Define Rust data models, establish cpal streams, build the matrix-mixing pipeline, and implement the lock-free ringbuffer bridge.

Phase 2: Tauri IPC & API Boundary (~1 Week)

Define the strict IPC payload structures. Build the command handlers that translate frontend requests into ringbuffer pushes.

Phase 3: React Frontend (~3 Weeks)

Initialize Tauri + React + Mantine. Implement Golden Layout. Build the virtualized cuelist, inspector panel, and active media panel.

Phase 4: Serialization & Polish (~1 Week)

Implement file saving/loading. Apply strict CSS (user-select: none) to ensure a native application feel.
