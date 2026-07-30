## Documentation References

Read through the `/docs/project/*.md` files to understand the project's design and UI layout.

## Tools

When running Bash tools in Yolo mode, always wrap long-running or network-dependent commands using timeout 30s <command>. If the command exits with status 124 (timeout), automatically log the failure, kill any remaining leaked processes, and retry the command exactly once before asking for help.

Abstain from broad filesystem searches - if file searching is required, search for specific files or directories instead of using broad wildcards or root directories. The only acceptable search locations are the project's source directory and any explicitly specified directories, eg cargo crate directories.

## 🎨 Floem & Taffy UI Rules

> **MANDATORY DIRECTIVE:** When writing, refactoring, or reviewing any Rust UI code utilizing the **Floem** framework, you MUST strictly adhere to the guidelines in [`docs\code-style\floem-style-guide.md`](docs\code-style\floem-style-guide.md).

### Quick Guardrails for Floem Layouts:

1. **No Flex Sizing Hacks:** Never use `.flex_grow()`, `.flex_shrink()`, or `.flex_basis()` for expanding views—use `.width_full()` or `.height_full()`.
2. **No Redundant Stack Styles:** Do NOT add `.flex_col()` or `.flex_row()` to `Stack::vertical` or `Stack::horizontal`.
3. **Prevent Taffy Blowouts:** Every dynamic `label(...)` inside flex/grid tracks MUST be constrained with `.min_width(0.0).text_ellipsis()`.
4. **Use Design Tokens:** Never hardcode hex strings or literal pixel dimensions; always use `theme().color` and `theme().dim`.
