//! Shared Lapce-inspired dark theme palette and a couple of reusable style helpers.
//!
//! The palette is deliberately close to Lapce's dark theme: deep charcoal
//! backgrounds, crisp 1px borders, muted text, and a theatre-green accent for
//! the primary transport / selection states.

use floem::peniko::Color;

// --- Surfaces -------------------------------------------------------------
/// Base application background (#1E1E1E).
pub const BG: Color = Color::rgb8(0x1e, 0x1e, 0x1e);
/// Raised panel surface (toolbar, sidebars, status bar).
pub const PANEL: Color = Color::rgb8(0x25, 0x25, 0x28);
/// Slightly brighter alt surface used for nested rows / inputs.
pub const PANEL_ALT: Color = Color::rgb8(0x2d, 0x2d, 0x31);
/// Hairline border / divider color.
pub const BORDER: Color = Color::rgb8(0x3a, 0x3a, 0x3f);

// --- Text -----------------------------------------------------------------
pub const TEXT: Color = Color::rgb8(0xd4, 0xd4, 0xd4);
pub const TEXT_DIM: Color = Color::rgb8(0x8a, 0x8a, 0x8a);
pub const TEXT_FAINT: Color = Color::rgb8(0x5a, 0x5a, 0x60);

// --- Accents --------------------------------------------------------------
/// Theatre green - primary "GO" / active-selection accent.
pub const ACCENT: Color = Color::rgb8(0x3f, 0xb9, 0x50);
pub const ACCENT_DIM: Color = Color::rgb8(0x2f, 0x8a, 0x3b);
/// Lapce-style blue, used sparingly for informational highlights.
pub const LAPCE_BLUE: Color = Color::rgb8(0x18, 0x90, 0xff);
/// Panic / stop-all red.
pub const PANIC: Color = Color::rgb8(0xf1, 0x4c, 0x4c);
pub const PANIC_DIM: Color = Color::rgb8(0xb8, 0x30, 0x30);

/// Live meter green (used by the active-media meters).
pub const METER: Color = Color::rgb8(0x4e, 0xc9, 0x5a);

pub const MONO: &str = "monospace";
