UI Layout & Interface Specification

This document defines the graphical interface architecture for our audio cue system. The application utilizes a robust 3-Panel Layout, ensuring operators have simultaneous access to the cue sequence, deep-edit parameters, and live playback telemetry without needing to switch tabs or open floating windows.

1. Global Application Layout

The window is divided into five main regions: a global toolbar, three primary interaction panels (separated by a main vertical split and a secondary horizontal split on the left), and a global status bar.

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


2. Component Breakdown

A. The Toolbar (Top)

The global control strip.

Transport Controls: The massive, misclick-proof GO button, along with BACK and Pause.

Search / Filter: A global text input that instantly filters the Main Cuelist for specific cue names, audio file names, or notes.

Safety Controls: The Panic (Stop All) button.

B. The Main Cuelist (Top-Left Panel)

The primary sequence view, occupying the top 2/3 of the left-hand split.

Flat-Chain Visual Folding: Displays the strict 1:1 Vec<Cue> flat chain. Cues chained via Auto-Continue or Auto-Follow are visually indented beneath their parent "GO" cue to create logical groupings without recursive data structures.

Rapid Navigation: Optimized for keyboard-centric navigation (Up/Down arrows to traverse the list, Enter to fire the standby cue).

C. Context-Dependent Detail Panel (Bottom-Left Panel)

The inspector view, occupying the bottom 1/3 of the left-hand split. It reactively updates based on what is selected in the Main Cuelist.

Cue Selected: Displays trigger constraints (Pre-wait, Post-wait), designer notes, and base "Inherit From" target data.

Task Selected: Displays explicit parameter editing controls (e.g., volume sliders, custom fade curves, and advanced multi-channel matrix routing).

Nothing Selected: Reverts to Global Show settings (e.g., master show volume, default ASIO/WASAPI devices).

D. Currently Playing Media Side Panel (Right Panel)

A dedicated, persistent view of the audio engine's live state.

Active Layer Telemetry: Iterates over active engine states to display currently playing cues.

Live Scrubbing: Provides visual progress bars for temporal audio playback, allowing the operator to click and drag to scrub the playhead dynamically.

Live Meters: Real-time volume meters for the specific playing assets, operating at high refresh rates.

Manual Override: Allows operators to manually intervene (e.g., dragging down the volume of a specific layer on the fly).

E. Status Bar (Bottom)

System health and environment telemetry.

Hardware Status: Displays the active low-latency driver.

Performance Metrics: CPU and DSP thread usage, essential for monitoring custom audio pipeline headroom.

3. UI Framework & Data Binding Notes (Slint & Lock-Free Ringbuffers)

With the pivot to Slint for the frontend and a custom cpal backend, the UI layer interacts with the audio engine through strict boundaries:

Declarative Layouts: The 3-panel layout is defined purely in .slint markup (using VerticalBox, HorizontalBox, and styling properties). This abstracts all CSS-like design logic away from the Rust execution code.

Asynchronous Command Pushing: When an operator clicks "GO" or drags a volume slider in the Slint UI, the Rust backend captures the callback and pushes a struct into a ringbuf::Producer. The cpal audio thread instantly consumes this from the ringbuf::Consumer without acquiring locks.

Reactive Telemetry (The Media Panel): To display smooth 60fps audio meters without bogging down the UI, the audio thread pushes peak volume data into a return-path ringbuffer. The Slint frontend reads this buffer via a lightweight Timer polling mechanism and updates bound reactive properties, which naturally redraws the UI components in isolation.
