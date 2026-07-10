UI Layout & Interface Specification

This document defines the graphical interface architecture for our audio cue system, utilizing the Tauri, React, and Golden Layout stack. The application provides a robust, dockable 3-Panel Layout, ensuring operators have simultaneous access to the cue sequence, deep-edit parameters, and live playback telemetry without sacrificing performance.

1. Global Application Layout

The window is managed by Golden Layout within a Tauri WebView. This allows for a highly customizable, drag-and-drop paneling system standard in professional audio software (meaning the operator can tear off the Media Panel to a second monitor if needed). It is divided into a global toolbar, three primary interaction panels, and a status bar.

ASCII Layout Schematic

+-----------------------------------------------------------------------------+
|  [||] (Pause)   [>] (GO)   [<] (BACK)   [Search: "wind" ]      [ PANIC ]    | <- TOOLBAR
+----------------------------------------------------+------------------------+
|                                                    |                        |
|  > CUE 1.0 - Storm Intro (Go)                      |  ACTIVE MEDIA          | <- MEDIA PANEL
|      - Play: Wind_Loop.wav (Vol: -12dB, Loop)      |                        |    (Right side,
|      - Play: Rain_Heavy.wav (Vol: -8dB, Loop)      |  > CUE 1.0 (Wind_Loop) |     Vertical split)
|      * Fade: BGM to -24dB (Dur: 3.0s)              |    [======-----] -12dB |
|  v CUE 2.0 - Thunder Strike (Go)                   |                        |
|  > CUE 3.0 - Storm Outro (Auto-Follow)             |  > CUE 1.0 (Rain)      |
|                                                    |    [========---]  -8dB |
|                                                    |                        |
|                MAIN CUELIST                        |                        |
|             (Top 2/3 of Left Pane)                 |                        |
|                                                    |                        |
+----------------------------------------------------+                        |
|                                                    |                        |
|  [Context: Task 1.1 - Wind_Loop]                   |                        |
|  Target: BGM         Property: Volume              |                        |
|  Target Vol: [-24]   Duration: [3.0]               |                        |
|  Matrix: [In L -> Out 1, 2] [In R -> Out 3, 4]     |                        |
|                                                    |                        |
|              DETAIL PANEL                          |                        |
|         (Bottom 1/3 of Left Pane)                  |                        |
+----------------------------------------------------+------------------------+
|  STATUS: Connected (ASIO: Focusrite USB)           |  CPU: 4%   DSP: 12%    | <- STATUS BAR
+-----------------------------------------------------------------------------+


2. Component Breakdown (React & Mantine UI)

A. The Toolbar (Top)

The global control strip rendered as a sticky header outside of the Golden Layout grid.

Transport Controls: Massive, misclick-proof GO and BACK buttons utilizing Mantine's robust Button components with custom CSS sizing and high-contrast theming.

Search / Filter: A global text input that instantly filters the Main Cuelist state.

Safety Controls: The Panic (Stop All) button, distinctively colored (e.g., Mantine's red palette).

B. The Main Cuelist (Top-Left Panel)

The primary sequence view, occupying the top 2/3 of the left-hand split.

Virtualized List: Built using @tanstack/react-virtual to handle thousands of cues smoothly without bloating the DOM.

Flat-Chain Visual Folding: Displays the strict 1:1 Vec<Cue> flat chain stored in Rust. Cues chained via Auto-Continue or Auto-Follow are visually indented beneath their parent "GO" cue using simple CSS padding.

Keyboard Navigation: Uses global DOM event listeners to capture Up/Down and Enter keys for rapid cue traversal and firing, ensuring focus isn't accidentally trapped inside a specific Mantine input.

C. Context-Dependent Detail Panel (Bottom-Left Panel)

The inspector view, built with Mantine UI form controls. Reactively updates based on the active selection state.

Inputs: Utilizes Mantine's NumberInput (for precise durations/fades), Select (for routing dropdowns), and Slider (for volumes).

Cue Selected: Displays trigger constraints (Pre-wait, Post-wait), designer notes, and base "Inherit From" target data.

Task Selected: Displays explicit parameter editing controls (e.g., volume sliders, custom fade curves, and advanced multi-channel matrix routing grids).

D. Currently Playing Media Side Panel (Right Panel)

A dedicated, persistent view of the audio engine's live state.

High-Performance Meters: To avoid React re-render bottlenecks, volume meters are drawn using direct DOM Refs (updating div height/width) or HTML5 <canvas>, bypassing the standard React state lifecycle.

Live Scrubbing: Provides visual progress bars for temporal audio playback.

Manual Override: Exposes quick-access volume sliders linked directly to active audio threads for live mixing adjustments.

E. Status Bar (Bottom)

System health and environment telemetry.

Hardware Status: Displays the active low-latency driver reported by the Rust backend.

Performance Metrics: CPU and DSP thread usage, essential for monitoring the Rust audio pipeline's real-time headroom.

3. UI Framework & IPC Boundaries (Tauri)

With the React/Tauri architecture, this layout interacts with the Rust audio engine through strict boundaries:

State as Truth: The Vec<Cue> lives in Rust. When a user changes a volume slider in the Detail Panel, React fires a Tauri invoke('update_property', payload) command. Rust updates the state and emits a state_changed event back to React to trigger a UI re-render.

Telemetry Throttling: The Active Media Panel requires 30-60fps updates. The Rust audio thread batches peak volume data into a small payload and pushes it over the Tauri IPC via emit. A dedicated useAudioTelemetry() hook in React catches this event and updates the meter DOM nodes directly.

Native Application Feel: CSS rules (user-select: none, custom scrollbars, disabled overscroll bounce) are applied globally in the React root to ensure the WebView behaves identically to a compiled native application, hiding any standard browser behaviors.
