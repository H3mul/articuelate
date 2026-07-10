# UI Layout & Interface Specification (Lapce-Inspired)

This document defines the graphical interface architecture for our audio cue system. The application utilizes a Lapce-inspired modular pane layout. This ensures a native, hyper-responsive feel while keeping complex data cleanly organized.

1. Global Application Layout

The window is structured like a modern code editor, maximizing screen real estate for the operator while keeping deep-dive parameters accessible in collapsible docks.

ASCII Layout Schematic

+-----------------------------------------------------------------------------+
| [||] (Pause) [>] (GO) [<] (BACK) [Search: "wind" ] [ PANIC ] | <- TITLE / TOOLBAR
+----------------------------------------------------+------------------------+
| | |
| > CUE 1.0 - Storm Intro (Go) | ACTIVE MEDIA | <- RIGHT SIDEBAR
| - Play: Wind_Loop.wav (Vol: -12dB, Loop) | | (Collapsible)
| - Play: Rain_Heavy.wav (Vol: -8dB, Loop) | > CUE 1.0 (Wind_Loop) |  
| * Fade: BGM to -24dB (Dur: 3.0s) | [======-----] -12dB |
| v CUE 2.0 - Thunder Strike (Go) | |
| > CUE 3.0 - Storm Outro (Auto-Follow) | > CUE 1.0 (Rain) |
| | [========---] -8dB |
| | |
| MAIN CUELIST | |
| (Virtualized List) | |
| | |
+----------------------------------------------------+ |
| | |
| [Context: Task 1.1 - Wind_Loop] | |
| Target: BGM Property: Volume | |
| Target Vol: [-24] Duration: [3.0] | |
| Matrix: [In L -> Out 1, 2] [In R -> Out 3, 4] | |
| | |
| BOTTOM PANEL | |
| (Collapsible) | |
+----------------------------------------------------+------------------------+
| STATUS: Connected (ASIO: Focusrite USB) | CPU: 4% DSP: 12% | <- STATUS BAR
+-----------------------------------------------------------------------------+

2. Component Breakdown & Floem Logic

A. The Toolbar (Top)

Design: Flat, borderless icon buttons.

Logic: Clicking "GO" triggers a signal that sends a lock-free message to the audio thread to evaluate the next cue index.

B. The Main Cuelist (Center / Main View)

Design: High-contrast text on a dark charcoal background. Selection states are highlighted with a subtle accent color (e.g., Lapce Blue or Theatre Green).

Floem Implementation: Uses Floem's virtual_list. The flat Vec<Cue> is mapped to a signal. Expanding/collapsing Auto-Continue chains dynamically updates the virtual list's viewport calculation.

C. Context-Dependent Detail Panel (Bottom Panel)

Design: Divided into logical tabs (e.g., "General", "Audio Routing", "Fades").

Lapce UX: Can be toggled open or closed with a keyboard shortcut (e.g., Ctrl+J) or dragging the splitter down to the bottom.

Component Strategy: No need for complex visual knobs. We use sleek, horizontal number sliders (similar to dragging a number value in Blender or Unity) and toggle-button grids for the audio routing matrix.

D. Currently Playing Media (Right Sidebar)

Design: A dedicated monitor for live telemetry.

Floem Implementation: This is where Floem shines. A timer running at 30/60fps pulls telemetry from the audio return-ringbuffer and updates a specific RwSignal<Vec<MeterData>>. Only the UI nodes bound to this signal (the green meter bars) update, leaving the rest of the UI untouched and saving CPU.

E. Status Bar (Bottom)

Design: Minimalist footer displaying active drivers, sample rate, and DSP load.

# Deep Dive: Floem & Lapce-Inspired UI Architecture

This document evaluates the pivot to Floem for the GUI framework, using the Lapce code editor as the primary reference for styling, window management, and component architecture.

1. Why Floem? (The Native Reactive Advantage)

Floem sits in a unique "sweet spot" in the Rust GUI ecosystem.

Iced requires diffing the entire UI view tree on every state change.

Egui requires redrawing the entire UI on every frame (Immediate Mode).

Floem uses Fine-Grained Reactive Signals. You wrap your state in signals (e.g., RwSignal<f32>). When a signal changes, Floem updates only the specific DOM node bound to that signal.

Audio Meter Performance

This reactive model is the holy grail for audio software. A high-frequency ringbuffer from the cpal audio thread can push peak volume values to a Floem RwSignal. The UI will update only the green meter bars at 60fps, utilizing virtually zero CPU, while the rest of the application remains entirely asleep.

2. Emulating Lapce (The Aesthetic & Layout Paradigm)

Lapce is widely praised for feeling incredibly snappy, native, and visually crisp. By studying Lapce's source code, we gain a massive head start on building our Audio Cue System.

A. The Dockable Panel System

Lapce uses a highly modular panel architecture (Left Sidebar, Right Sidebar, Bottom Panel, Main Editor area) with draggable splitters.

Our Implementation: We map our 3-panel layout directly into a Lapce-style shell.

Main Cuelist: Acts as the central "Editor" area.

Detail Inspector: Sits in the collapsible "Bottom Panel".

Active Media / Meters: Sits in the collapsible "Right Sidebar".

Operators can hide the Detail Panel during a live show using a single keystroke (like Cmd/Ctrl + J in VS Code/Lapce), leaving only the Cuelist and Meters.

B. Theming & Styling

Floem uses a Rust-based, CSS-like styling API. We will lift Lapce's dark-mode aesthetic:

Deep charcoal backgrounds (#1E1E1E or Lapce's specific themes).

Monospace font integration for timers, cue numbers, and routing grids.

Crisp, 1px borders and subtle hover states without heavy web-like drop shadows.

C. The Virtualized List

Lapce handles massive code files using virtualized (lazy) rendering. We will use Floem's virtual_list component to render the Vec<Cue> flat-chain. A show file with 5,000 cues will render instantly and consume negligible memory.

3. Development Trade-offs

The Challenge: Floem is young. You won't have a library of pre-built "Audio Knobs" like you would in React or egui.

The Solution: For the MVP, we rely on standard UI paradigms (horizontal sliders, number inputs, dropdowns, routing matrices built from toggle buttons) rather than skeuomorphic knobs. This perfectly matches the sleek, flat, developer-centric aesthetic of Lapce anyway.
