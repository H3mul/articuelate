# **UI Layout & Interaction Specification: Unified Two-Pane Workspace**

This document defines the interface architecture, visual layout, and the state-exchange mechanisms connecting the Floem UI to the Execution Engine.

## **1\. The Unified Two-Pane Layout Concept**

The workspace is divided into two primary logical panes, maintaining a strict "Source of Truth" separation where the UI simply renders the state provided by the Execution Engine.

\+-------------------------------------------------------------+-----------------------+  
| \[Search: "wind" \] \[ Panic \] | DETAIL PANEL |  
\+-------------------------------------------------------------+ |  
| \> CUE 1.0 \- Storm Intro (Go) \[0.0s\] \[10.0s\] | |  
| \- Play: Wind\_Loop.wav (Vol: \-12dB, Loop) | \[Context: Task 1.1\] |  
| \- Play: Rain\_Heavy.wav (Vol: \-8dB, Loop) | |  
| \* Fade: BGM to \-24dB (Dur: 3.0s) | Target: BGM |  
| v CUE 2.0 \- Thunder Strike (Go) \[1.5s\] \[ 0.0s\] | Property: Volume |  
| \> CUE 3.0 \- Storm Outro (Auto-Follow) \[0.0s\] \[ 5.0s\] | Target Vol: \[ \-24 \] |  
| | Duration: \[ 3.0 \] |  
\+-------------------------------------------------------------+-----------------------+  
| STATUS BAR: \[Output Levels (Active Media Telemetry)\] | Mode: Edit (Ctrl+E) |  
\+-------------------------------------------------------------+-----------------------+

## **2\. State Exchange Mechanisms**

To maintain thread safety and high performance, the UI interacts with the system using four distinct data conduits.

### **A. The Blueprint Update (UI Engine)**

- **Mechanism:** ArcSwap\<WorkspaceState\>
- **Flow:** When the user modifies a cue in the Detail Panel, the UI constructs a complete new WorkspaceState, wraps it in an Arc, and performs an atomic swap. This is the **only** way the Execution Engine receives data updates. The UI does not push partial updates; it pushes the full valid state of the show file.

### **B. Discrete Intents (UI Engine)**

- **Mechanism:** mpsc::Sender\<UserIntent\>
- **Flow:** High-level operator actions (GO, Panic, ScrubMedia, JumpPlayhead) are sent as discrete messages. These are processed asynchronously by the Execution Orchestrator, ensuring the UI remains perfectly responsive during show execution.

### **C. Execution State Watch (Engine UI)**

- **Mechanism:** tokio::sync::watch::Receiver\<Arc\<ExecutionState\>\>
- **Flow:** The Execution Engine pushes a new snapshot of execution progress (playhead position, running cue IDs, and timer values) whenever the state changes.
- **UI Ingestion:** The UI thread hosts a non-blocking background listener that updates a Floem RwSignal\<Arc\<ExecutionState\>\>. The cuelist reactively binds to this signal to highlight active rows, update progress bars, and move the playhead cursor.

### **D. Media Telemetry (DSP ![][image1] UI)**

- **Mechanism:** Lock-free Ringbuffer
- **Flow:** High-frequency playback metadata (playhead position per file, peak volume levels) bypasses the Execution Engine entirely. It is pushed directly from the DSP thread to a UI-side RwSignal\<HashMap\<CueId, PlaybackState\>\>. This ensures 60fps responsiveness for meters and progress bars without saturating the async runtime.

## **3\. UI Selection & Inspector Logic**

The Detail Panel is governed by the SelectedItem enum, acting as a router that determines which data from the ArcSwap snapshot to bind to the input widgets.

\#\[derive(Debug, Clone, PartialEq)\]  
pub enum SelectedItem {  
None,  
Cue { cue\_id: String },  
Task { cue\_id: String, task\_index: usize },  
}

- **Interaction Logic:** Selecting an item updates the selected\_item signal. The Detail Panel reactively re-renders to reflect the properties of the selected cue or task.
- **Commit Cycle:** Input widgets (sliders, text boxes) inside the Detail Panel perform local edits; upon the user finishing an interaction (e.g., on\_blur or on\_release), the UI triggers an ArcSwap commit of the updated workspace state.

## **4\. Usability Design Constraints**

1. **Virtualized Lists:** The main Cuelist uses Floem's virtual\_list to handle massive show files, ensuring that the Arc\<ExecutionState\> watch updates only cause the visible portion of the list to re-render.
2. **Keyboard-First Focus:** Navigation is mapped directly to the SelectedItem selection state. Moving focus with Up/Down arrows updates the selected item, instantly causing the Detail Panel to display the context of the new selection.
3. **Playback Lock:** During active show playback, the UI enforces a "Lock Edit" mode on structural cues to prevent race conditions in the ArcSwap commit cycle, while still allowing non-structural parameter tweaks via direct DSP-bound signals where safe.

[image1]: data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAdCAYAAACwuqxLAAABpklEQVR4Xp1VvU7DMBhMJCSQ+BESVBFKYielUh+gPAwTIwMTOyNiZEIsXRgYeQFejJ0vToL9+c5JxEmunbv7/mK1zfJM4D40Qsp7BlZvE1COPBkAxVKYFEeE3UGnYROpbINnlFM2wIyRy4xV5WNKj8TCQYR8JBGcOTHjV1h24QDodiHCuKZpjowx96vV5Ql4ABHLJ9QmecqttS9SZBey/2t+CPDBfSJjzU6KvMrxYLQ6sAKpTkMQJZdX9WiNuevOIP59DLuYz6WjKyOr2+PF+PX6Wt6S+ZDz22azOfPpIxRFcWyNfRLjXhLtu10vQ7h+SYFv2X9kPUuRQzXLOAbM56AvTY08oK7rG0n8VZZlReQ5aHscLEkvJPln07YWxBigA4GQ5A9yb7eeIUGEQhBTK180ubP3qqpKRxCPw8jPvWuN3rHdbk9lU87JsB7k94cQQA3wfOSArsNpAOlRJ/2oAkGQ9qACjH4VvuHJ8ThSXsjHCZIgIvwj+UMPvWlqOQG6A2d1Q7rRJJxEOk2BexYlWGRCOQwDMUuPGT6xOI05R1h+ySDEH+IXCpIokiS2F0IAAAAASUVORK5CYII=
