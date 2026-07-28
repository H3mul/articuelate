Next.js to Floem Migration: Agent Workflow Guide

You are an expert systems and UI engineer tasked with migrating a Next.js React prototype into a native Rust desktop application using the Floem GUI framework.

You have access to the local codebase (including a mockup/ directory with the Next.js prototype) and a Chrome MCP server attached to the running web app.

Instead of guessing the layout, you will dynamically introspect the web prototype and systematically translate it to Floem using the two-phase workflow below.

Phase 1: Analysis & Mapping

Do NOT write any Rust UI code in this phase. Your sole objective is to inspect, analyze, and map the existing web application state into a structured Floem component blueprint.

1. Web Prototype Introspection

Analyze DOM & React Codebase: Use Chrome MCP tools to inspect the active DOM structure of the running Next.js app. Correlate DOM nodes with React components in mockup/src/app/.

Measure Proportions & Layout: Inspect computed styles to record exact flexbox ratios, fixed pixel dimensions (e.g., status bar height, column widths, row heights), and split-pane constraints.

Inspect Interactive States & Telemetry: Inspect how hover, selection, focus, and running cue states are styled in Next.js, including telemetry meters and icon usages.

2. Architectural Blueprint Generation

Produce a Component Mapping Report containing:

Root Shell Structure: High-level flex/grid hierarchy (e.g., Vertical Split -> Top Workspace / Bottom Inspector; Status Bar at bottom).

Module Breakdown: List of required Rust UI modules to map to src/ui/ (e.g., status_bar.rs, cuelist.rs, media.rs, detail.rs, tabbed.rs, icons.rs).

State Inventory: Map React useState hooks to Floem RwSignal instances (e.g., selected cue ID, active cuelist vector, tab selection, meter telemetry values).

Icon Inventory: Catalog all Lucide icons used across the prototype to map to lucide-floem.

MANDATORY GATE: Output the Component Mapping Report for user review and approval before proceeding to Phase 2.

Phase 2: Implementation & Assembly

Once the analysis report is approved, execute the structural translation and Rust implementation using the guidelines below.

1. Structural Translation Rules (React/Tailwind ➔ Floem/Taffy)

A. Container Layouts

Floem relies on the Taffy layout engine. Map HTML/Tailwind flexbox directly to Floem primitives:

<div className="flex flex-col"> ➔ v_stack((child1, child2))

<div className="flex flex-row"> ➔ h_stack((child1, child2))

<div className="grid"> ➔ Use container() or h_stack() with .style(|s| s.display(Display::Grid).grid_template_columns(...))

Scrollable Views: <div className="overflow-y-auto"> ➔ scroll(v_stack(...)) or virtual_list(...) for the cuelist.

Resizable Panes: React <ResizablePanelGroup> ➔ Floem resizable_panel(...) (or custom split container).

B. Icon System (lucide-floem)

Central Encapsulation: Encapsulate lucide_floem::Icon calls inside ui/icons.rs.

Token Dimensions: Bind sizes to --spacing-icon-sm (14px) or --spacing-icon-md (18px) from ThemeDimensions.

Example Wrapper (ui/icons.rs):

use lucide_floem::{Icon, LucideIcon};
use floem::views::Decorate;

pub fn app_icon(icon: LucideIcon, size: f32) -> impl Decorate {
Icon::new(icon)
.style(move |s| s.width(size).height(size).flex_shrink(0.0))
}

C. Styling & Classes

Apply pre-registered CSS classes via .class(ClassName) on views (e.g., PanelSurface, BtnPunchDown, CueRowGrid).

NO LITERALS: Never hardcode hex colors or inline pixel dimensions. Use ThemeColors and ThemeDimensions.

D. Taffy Layout Safety (CRITICAL)

In Floem/Taffy, dynamic text labels in flex/grid tracks will blow out container bounds unless strictly constrained.

Rule: Any dynamic text label inside a grid or flex row (e.g., Cue Names, File Paths) MUST have text ellipsis and a min-width applied:
label(|| text).style(|s| s.text_ellipsis().min_width(0.0))

E. Interactions & Events

onClick={...} ➔ .on_click_stop(move |_| { ... }) or .action(...) for Floem button primitives.

useState ➔ RwSignal::new(...). Read/track signals in view closures using .get() or .with().

2. Step-by-Step Assembly Execution

Build Layout Shell (src/app.rs & src/ui/mod.rs):

Assemble the main application shell using v_stack and h_stack based on Phase 1 mapping.

Use placeholder containers (container(label(|| "Placeholder"))) for inner panes.

Verify shell flex proportions and fixed status bar height.

Implement Components Iteratively (src/ui/):

Migrate one module at a time (status_bar.rs ➔ detail.rs ➔ media.rs ➔ cuelist.rs).

Use Chrome MCP as needed to double-check computed styles or padding of specific widgets.

Inject established CSS classes via .class(...) and lucide-floem icons via src/ui/icons.rs.

Wire Dynamic State Signals:

Define top-level RwSignal states for the cuelist, active selection, and active running cues.

Bind signals to virtual_list for the cuelist and telemetry bars for active media.

Execution Command

"Begin Phase 1 (Analysis & Mapping). Introspect the Chrome mockup via MCP, inspect the codebase, and present the Component Mapping Report for approval. Do not write Rust code until Phase 1 is approved."
