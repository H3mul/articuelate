//! Styling subsystem for Articuelate.
//!
//! Broken into three sub-modules:
//!
//! - **tokens** — colour, font, and dimension data types (parsed from toml)
//! - **theme**  — file loading, merging, live-reload, and the reactive `theme()` accessor
//! - **style**  — the global `Style` stylesheet applied to the root view

pub mod style;
pub mod theme;
pub mod tokens;

pub use style::global_stylesheet;
pub use theme::*;
