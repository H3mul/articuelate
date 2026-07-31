# Agent Directive: Advanced Framework Introspection & Context Mastery

## 🎯 Primary Objective

You are tasked with writing, refactoring, and maintaining production Rust code utilizing the Floem GUI framework. Because Floem is actively developed, rapidly evolving, and heavily macro-driven, your internal training weights regarding its API are outdated and highly prone to hallucination. You MUST rely on the live local source code repository for architectural truth.

---

## 📂 Source Code Location & Mapping

When analyzing Floem's API, signatures, layout paradigms, and reactive state management, you must introspect the following local directory path:

- **Target Repository Path:** `../floem`

### 🔍 Context Priority Hierachy

When pulling context from the Floem repository, prioritize your analysis in this exact order:

1.  **`floem/examples/` (The Ultimate Truth):** Analyze these files first. They contain the most accurate, compiler-verified implementations of modern UI views, layout macros, and reactivity state flows.
2.  **`floem/src/` (Core Definitions):** Use this to resolve raw struct layouts, trait bounds, and `rustc` compiler error mismatches.
3.  **`floem/Cargo.toml` (Dependency Tree):** Use this to verify active feature flags and sub-crate workspaces (e.g., reactive primitives vs. rendering loops).

---

## 🛠️ How to Use This Source Code Context

### 1. Zero-Hallucination Syntax Mapping

- Do NOT invent macro names or assume compatibility with older versions (e.g., `0.2.0`).
- Cross-reference the project's reactive state engine directly with Floem's internal `create_rw_signal` or signal-tracking implementations found in the `src/` directory before proposing view tree changes.

### 2. Multi-File "Graph" Problem Solving

- If a local compilation error (`cargo check`) indicates a missing trait bound or a lifetime mismatch, do not just patch the immediate line.
- Look inside `floem/src/` to see how that specific trait is defined. Synthesize how the type system interacts across files before outputting code.

### 3. Idiomatic Layout Pattern Matching

- Before building new custom widgets or data-grid components, parse the most structurally similar file inside the `examples/` directory.
- Mirror Floem's unique macro-driven styling layouts exactly as demonstrated in the examples, rather than borrowing paradigms from other Rust GUI crates like `iced` or `druid`.

---

## 🚫 Context Guardrails (Noise Control)

To maximize token efficiency and preserve your attention mechanism across deep prompts:

- **IGNORE** the `floem/tests/` directory unless debugging a highly isolated unit-test failure.
- **IGNORE** any target build artifacts, benchmark files (`benches/`), or continuous integration files (`.github/`).
- **DO NOT** copy-paste or read massive blocks of internal layout engine implementation details unless a cryptic compiler error specifically points to a broken macro expansion inside them. Focus entirely on the public API layer and example usages.

# **Floem & Taffy Layout Guide for Autonomous AI Agents**

> **CRITICAL DIRECTIVE FOR LLMs / AGENTS:** > > > > > \> Floem wraps the **Taffy** flexbox engine, but its styling API (Style) differs from CSS and web Tailwind paradigms. Follow these strict rules to avoid common compilation errors, redundant styles, and Taffy layout blowouts.

## **1\. Flexbox Anti-Patterns & Hallucinations**

### **❌ DO NOT use flex\_grow, flex\_shrink, or flex\_basis for standard sizing**

- **Hallucination:** LLMs habitually inject .flex\_grow(1.0).flex\_shrink(1.0).flex\_basis(0.0) into views to make them expand.
- **Reality:** In Floem, views expand naturally to fill available space when given **width\_full()** or **height\_full()**.
- **Rule:** Reserved flex\_grow() strictly for **disproportionate weighted scaling** (e.g., Column A gets flex\_grow(2.0), Column B gets flex\_grow(1.0)). Otherwise, **never use it**.

### **❌ DO NOT use flex\_col() or flex\_row() on Stack primitives**

- **Hallucination:** Adding .flex\_col() to Stack::vertical(...) or .flex\_row() to Stack::horizontal(...).
- **Reality:** Stack::vertical and Stack::horizontal already set the Taffy FlexDirection internally. Calling .flex\_col() or .flex\_row() is completely redundant and clutters the styling chain.
- **Rule:** Only use flex\_col() / flex\_row() if you are constructing any container that doesn't have an initial orientation like a Stack.

## **2\. Dynamic Sizing & Layout Expansion Rules**

### **✅ Use width\_full() and height\_full() to fill parent bounds**

To make a view stretch across its parent container on the main or cross axis:

// CORRECT  
Stack::vertical((top\_row, bottom\_row))  
.style(|s| s.width\_full().height\_full())

### **✅ Use justify\_between() for Left-Right / Top-Bottom splits**

- **Anti-Pattern:** Inserting an empty() view or container with width\_full() or flex\_grow() as a spacer between two elements.
- **Correct Pattern:** Set .justify\_between() on the parent row or column.

// CORRECT: No dummy spacer views needed  
h\_stack((  
label(|| "Left Title"),  
label(|| "Right Value"),  
))  
.style(|s| s.width\_full().justify\_between().items\_center())

### **✅ Use margin\_left\_auto() for asymmetric grouping**

When you have 3+ elements in a row and want some pinned left and one pinned right:

// CORRECT: Pushes the third element to the far right  
h\_stack((  
icon(Music),  
label(|| "Track Name"),  
button(|| "Delete").style(|s| s.margin\_left\_auto()),  
))  
.style(|s| s.width\_full())

## **3\. Taffy Layout Safety (Preventing Container Blowouts)**

### **⚠️ MANDATORY: Dynamic Labels MUST have min\_width(0.0) and text\_ellipsis()**

In Taffy / Floem flexbox, dynamic text labels (cue names, file paths, dynamic values) report their full unclipped intrinsic width to the parent. Without explicit constraints, **dynamic text will push adjacent buttons/meters completely offscreen**.

// ❌ WRONG: Will blow out the row layout if cue.name is long  
label(move || cue.name.clone())

// ✅ CORRECT: Safely truncates with ellipsis inside flex containers  
label(move || cue.name.clone())  
.style(|s| s.min\_width(0.0).text\_ellipsis())

### **⚠️ Scroll & List Containers MUST set min\_size(0.0, 0.0)**

Flex children inside scrollable containers or nested stacks will fail to shrink properly if their minimum size is left unconstrained.

// ✅ ALWAYS constrain nested scroll/virtual lists  
scroll(cuelist\_view)  
.style(|s| s.width\_full().height\_full().min\_size(0.0, 0.0))

## **4\. Borders & Rounded Corners**

### **❌ Non-uniform borders break border\_radius**

- **Limitation:** Setting asymmetric borders (e.g., border\_left(4.0) with border(1.0)) alongside border\_radius() breaks Vello's curve rendering, turning corners sharp/straight.
- **Solution:** For flush status accent bars on cards/rows, use a **Position::Absolute overlay** pinned to the left edge:

// ✅ CORRECT: Smooth rounded corners with an accent bar  
Stack::horizontal((  
// 1\. Absolute Accent Bar Overlay  
empty().style(move |s| {  
s.position(Position::Absolute)  
.inset\_top(0.0)  
.inset\_bottom(0.0)  
.inset\_left(0.0)  
.width(4.0)  
.background(accent\_color)  
.border\_top\_left\_radius(5.0)  
.border\_bottom\_left\_radius(5.0)  
}),  
// 2\. Main Card Content  
card\_content.style(move |s| s.padding\_left(12.0)),  
))  
.style(move |s| {  
s.width\_full()  
.background(theme().color.surface)  
.border(1.0)  
.border\_color(theme().color.border)  
.border\_radius(5.0)  
})

## **5\. Style Token & Color Hygiene**

### **❌ NEVER use raw Hex strings or literal pixels inside .style()**

- **Wrong:** .background("\#181926"), .padding(8.0)
- **Right:** Use ThemeColors and ThemeDimensions context tokens or pre-defined CSS class names:

// ✅ CORRECT: Fully tokenized  
.style(move |s| {  
s.background(theme().color.element\_bg)  
.padding(theme().dim.space\_sm)  
.border\_radius(theme().dim.radius\_md)  
})

## **6\. Agent Code Style Cheat Sheet**

| Intent                  | ❌ LLM Hallucination              | ✅ Correct Floem Style              |
| :---------------------- | :-------------------------------- | :---------------------------------- |
| **Fill Parent Width**   | .flex\_grow(1.0).flex\_basis(0.0) | .width\_full()                      |
| **Fill Parent Height**  | .flex\_grow(1.0).height(0.0)      | .height\_full()                     |
| **Left / Right Split**  | Inserting empty().style(          | s                                   | s.width\_full()) | .justify\_between() on parent row   |
| **Push 1 item right**   | Adding spacer views               | .margin\_left\_auto() on right item |
| **Text Truncation**     | Plain label(...)                  | label(...).style(                   | s                | s.min\_width(0.0).text\_ellipsis()) |
| **Vertically Center**   | .align\_items\_center()           | .items\_center()                    |
| **Horizontally Center** | .justify\_content\_center()       | .justify\_center()                  |
