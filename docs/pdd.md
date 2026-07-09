Product Definition Document (PDD)

Project: Open-Source Audio Cue System
Target Platforms: Windows & Linux (Desktop)

1. Executive Summary

This project aims to create a modern, cross-platform, open-source audio cue system designed specifically for professional live show control (theatre, immersive experiences, live events).

Unlike DAWs (Digital Audio Workstations) which rely on static timelines, this system is a deterministic, event-driven state machine. It provides a highly reliable, operator-centric runtime environment ("Show Mode") focused on safe, predictable audio execution, quick tech-rehearsal recovery, and intuitive show design.

2. Core Philosophy & Data Architecture

The system embraces a Commands Over Time philosophy, optimized into a Strict 1:1 Flat Chain data model to maximize execution safety and minimize programming complexity.

2.1 The Strict 1:1 Data Model

1 Cue = 1 Action = 1 Targetable Object.

A Cue is an enum of exactly one action (e.g., PlayAudio, Fade, Stop).

Under the hood, the entire show is stored and executed as a single, flat Vec<Cue>.

Composition (simultaneous playback) is achieved by chaining cues using Auto-Continue or Auto-Follow triggers.

Targeting is unambiguous: Fades and Stops always target the explicit ID of the Cue that generated the audio layer.

2.2 The "Inherit From" (Templating) Pattern

To solve cue duplication and bulk-editing (the "DRY" principle), the system implements a pure Data-Layer Inheritance pattern.

A cue can be designated to "Inherit From" a master cue (e.g., Cue 10 inherits from Cue 1).

Data Overlay: Any property modified in Cue 10 is overlaid on top of Cue 1's data (using Rust's Option<T> for overrides).

Runtime Isolation: When triggered, the engine resolves the overlaid data. Cue 10 fully owns its execution state and custom audio pipeline. It is completely decoupled from Cue 1 at runtime.

2.3 State-Squashing (Rehearsal Mode)

When jumping out of sequence (e.g., operator skips from Cue 1 to Cue 4):

The engine evaluates the skipped cues instantly.

Non-temporal tasks (absolute volume targets, explicit playhead seeks) are "squashed" and applied immediately via lock-free messages to the audio thread.

Temporal constraints (durations, fades) are executed with a 0.0s duration.

Active, uninterrupted audio streams continue playing naturally.

3. Technology Stack

The application is strictly partitioned into a declarative frontend and a low-latency, real-time backend, guaranteeing that UI rendering never blocks audio DSP processing.

Audio Hardware Interface: cpal

Used directly to initialize and lock hardware audio streams, providing critical access to professional low-latency drivers (ASIO on Windows, WASAPI Exclusive, JACK/ALSA on Linux).

Audio DSP Engine: Custom Pipeline (fundsp / Slice Manipulation)

Replaces rigid game-audio abstractions to allow for true crosspoint matrix mixing (arbitrary N-in to N-out channel routing), sample-accurate triggers, and professional-grade dynamic time stretching and pitch shifting.

Thread Communication: Lock-Free Ringbuffers (ringbuf or rtrb)

Serves as the critical safety bridge. State changes (play, pause, volume tweaks) and telemetry (peak meter data) are passed between the Slint UI thread and the cpal real-time audio thread without using mutexes, eliminating the risk of audio dropouts or stutters.

UI Framework: Slint

A component-driven, declarative GUI framework using .slint markup. Provides the sleek, high-contrast, dark-themed "Pro Audio" aesthetic without boilerplate Rust styling code. UI logic reacts to property bindings updated by the backend.

Serialization: Serde

Handles JSON/YAML project serialization and "Inherit From" data overlays.

4. User Interface (UI) Specification

The system replaces the traditional, cluttered, multi-tab layout with a highly focused Unified Two-Pane Workspace rendered in Slint.

4.1 Pane 1: The Unified Cuelist (Left)

Displays the flat Vec<Cue> execution chain using Slint's high-performance ListView for massive datasets.

Visual Folding: Cues linked via Auto-Continue or Auto-Follow are visually indented and grouped under their parent "GO" cue. A toggle allows the operator to collapse these chains, hiding the "tail" and keeping the UI clean during live runs.

4.2 Pane 2: Contextual Detail Panel (Right/Bottom)

Reactively updates based on the selection state passed to the Slint model.

Displays explicit overrides, base inherited values, and multi-channel routing matrices.

Acts as the sole editing location, keeping the operator's physical focus locked to one area.

4.3 Live "Show Mode"

A runtime workspace state that locks destructive edits.

Features a massive GO button, BACK button, Panic (Stop All), and an Active Playback Panel listing all currently active audio layers with live output meters driven by ringbuffer telemetry.

5. Ethical & Legal Framework

To ensure the project remains legally defensible as an open-source alternative to proprietary software (like QLab):

Clean Room Development: Built entirely from scratch in Rust/Slint. No code decompilation or reverse engineering is utilized.

Original Assets: All UI paradigms rely on custom .slint styles or standard open-source icon sets. No proprietary icons or graphic assets are cloned.

IP Compliance: The visual folding of flat cue chains and generic theatrical control methods (Auto-Follow, Cue Targeting) fall under unprotected "methods of operation." The project brand and marketing will maintain strict separation from proprietary trademarks.

6. MVP Development Roadmap

Phase 1: State Machine & Data Models (~1.5 Weeks)

Define the Vec<Cue>, pure flat-chain structs, and Inherit From data overlay logic.

Phase 2: Custom Audio DSP Engine (~3 Weeks)

Initialize cpal streams (ASIO/WASAPI). Build the low-level playback and matrix-mixing pipeline (using fundsp or manual slices). Establish the ringbuf communication channels.

Phase 3: Slint GUI Integration (~2.5 Weeks)

Design the .slint declarative frontend. Build the Rust data-bridge to bind the Phase 1 state machine to Slint's reactive properties. Implement the Three-Panel layout and visual cue folding.

Phase 4: Serialization & Hardware Resilience (~1.5 Weeks)

Implement JSON saving/loading via Serde (managing relative file paths). Ensure graceful fallback and recovery if hardware interfaces are disconnected.
