## Documentation References

Read through the `/docs/*.md` files to understand the project's design and UI layout.

## Tools

When running Bash tools in Yolo mode, always wrap long-running or network-dependent commands using timeout 30s <command>. If the command exits with status 124 (timeout), automatically log the failure, kill any remaining leaked processes, and retry the command exactly once before asking for help.

Abstain from broad filesystem searches - if file searching is required, search for specific files or directories instead of using broad wildcards or root directories. The only acceptable search locations are the project's source directory and any explicitly specified directories, eg cargo crate directories.

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
