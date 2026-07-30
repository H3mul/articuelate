# **Product Definition Document (PDD)**

**Project:** Open-Source Audio Cue System

**Target Platforms:** Windows, macOS, & Linux (Native Desktop)

## **1\. Executive Summary**

This project aims to create a modern, cross-platform, open-source audio cue system designed specifically for professional live show control.

Built purely in Rust, it delivers a deterministic, event-driven state machine with the ultra-low memory footprint and extreme speed of a native application. It achieves this using a dedicated real-time DSP engine and a native reactive frontend, coordinated by a dynamic, asynchronous execution orchestrator.

## **2\. Core Philosophy & Data Architecture**

The system utilizes an **Async Actor Model** for execution, meaning cues operate as independent async tasks that dynamically read from a live "Source of Truth" blueprint. This guarantees that UI edits take effect instantly, even while a sequence is running.

### **2.1 Cues (User Intent)**

- The show is stored as a flat Vec\<Cue\>.
- A Cue is a data container holding user settings (Volume, File Path, Pre-Waits, Post-Waits).
- **Trigger Relationships:** Cues define their relationship to other cues via Playhead (Manual GO), With (Target Cue), or After (Target Cue).
- **Inherit From:** A cue can inherit data from a master template. Overrides are stored via Option\<T\> and resolved dynamically during the cue's execution.

### **2.2 The Async Actor Model (Runtime Execution)**

- **The Orchestrator:** Instead of pre-compiling a rigid sequence of tasks, an Event Orchestrator routes lifecycle events. When "GO" is pressed, the Orchestrator spawns a lightweight async task (an "Actor") for the target cue.
- **Dynamic Hydration:** During its lifecycle (e.g., before and after a pre-wait), the Actor queries an atomic ArcSwap reference to the show file. This means any UI edits made during a wait timer are instantly applied when the timer finishes.
- **Cascading Events:** When an Actor begins or finishes its primary action, it emits an event (ActionStarted or ActionFinished). The Orchestrator hears this and instantly spawns new Actors for any cues configured to trigger With or After that cue.

### **2.3 The Linear Playhead & "Early GO" Preemption**

- The Playhead is a strict, linear cursor. When "GO" is pressed, the playhead fires the selected cue and immediately jumps to the next _independent_ cue (skipping over With cues, and stopping on After or Manual cues).
- **Preemption:** If the operator hits "GO" early while the playhead is sitting on a pending After cue, the Orchestrator acts as an override, instantly aborting the wait listeners and firing the cue.

### **2.4 State-Squashing (Rehearsal Mode)**

- Skipping cues during rehearsal is achieved by fast-forwarding the Async Actors: tokio::time::sleep calls are bypassed, and DSPCommand::Play instructions are suppressed until the engine's internal time catches up to the target timestamp, snapping the DSP to the correct state.

## **3\. Technology Stack & Threading Model**

The application utilizes a strict **Tri-Thread Architecture** to guarantee microsecond audio stability while maintaining fluid UI and complex logic evaluation.

### **3.1 Frontend & State (The UI Thread)**

- **Framework:** **Floem**. A native, retained-mode, reactive GUI framework using fine-grained reactive signals (RwSignal).
- **Role:** Acts as the "Dumb View". It handles user input and visual layout.
- **Data Flow:** When the user edits a cue, the UI publishes a new Arc\<WorkspaceState\> and atomically swaps the global ArcSwap pointer. It pushes discrete events (like Intent::GoPressed) to the Execution Engine via lock-free channels.

### **3.2 Execution Engine (Background Async Runtime)**

- **Framework:** **Tokio** (or async-std).
- **Role:** The "Controller". It runs entirely off the main UI thread to avoid OS window-interaction freezing. It owns the Playhead logic and orchestrates the Async Cue Actors.
- **Data Flow:** Ingests UI intents, reads the ArcSwap blueprint, and pushes raw DSP commands (Play(Node), SetVolume(Node, Vol)) to the audio thread. It broadcasts its current state (playhead position, active timers) back to the UI via a tokio::sync::watch channel.

### **3.3 Core Audio Backend (Real-Time DSP Thread)**

- **Hardware:** cpal directly locks low-latency drivers (ASIO/WASAPI/CoreAudio).
- **Role:** The "Muscle". A custom node-based pipeline handling crosspoint matrix mixing. It has zero knowledge of "Cues" or "Playheads".
- **Data Flow:** Communicates with the Execution Engine _exclusively_ via lock-free ringbuffers (ringbuf), ensuring the audio deadline (e.g., 2.6ms) is never compromised.
- **Direct Telemetry:** High-frequency media telemetry (playhead progress, peak volume meters) is pushed directly from the DSP thread back to the UI thread via a return ringbuffer, completely bypassing the Execution Engine.

## **4\. User Interface (UI) Specification**

The workspace provides a **Unified Three-Pane Layout** utilizing a dockable, collapsible panel system modeled after the **Lapce** code editor.

### **4.1 Pane 1: The Unified Cuelist (Main View)**

- Renders the flat chain using Floem's virtualized lists. Linked With/After cues are visually folded under their parent triggers.

### **4.2 Pane 2: Contextual Detail Panel (Bottom Panel)**

- Collapsible panel containing flat sliders, number inputs, and routing toggle-grids. Modifying these instantly updates the global ArcSwap state.

### **4.3 Pane 3: Active Media Panel (Right Sidebar)**

- Displays real-time engine telemetry (progress bars and meters). High-frequency updates bypass full UI redraws via Floem's surgical signal bindings, driven by the DSP return-ringbuffer.

## **5\. MVP Development Roadmap**

- **Phase 1: DSP Engine & Ringbuffers (\~2.5 Weeks)**
    - Establish cpal streams, build the matrix-mixing DSP pipeline, and implement the lock-free ringbuffer command receiver and telemetry publisher.
- **Phase 2: Event Orchestrator & Async Actors (\~2 Weeks)**
    - Define the Cue structs and global ArcSwap state. Build the Tokio orchestrator and the execute\_cue\_lifecycle async actor loops. Implement the linear Playhead and "Early GO" logic.
- **Phase 3: Floem GUI Implementation (\~2.5 Weeks)**
    - Bridge the Phase 2 watch channels to Floem RwSignals. Build the Lapce-style layout, virtualized Cuelist, and inspector forms.
- **Phase 4: Serialization & Polish (\~1 Week)**
    - Implement Serde file I/O for the workspace and finalize UI keyboard shortcuts.
