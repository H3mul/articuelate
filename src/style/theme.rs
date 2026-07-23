//! Theme file parsing and live-reload for Articuelate.
//!
//! All `.toml` files in `themes/` are merged in alphabetical order (later files
//! override earlier ones). At boot [`load_theme`] panics on failure so the app
//! never starts with a broken theme. During hot-reload [`try_load_theme`]
//! silently returns `None` so the old theme stays in place.
//!
//! Access resolved values through [`theme()`], e.g. `theme().color.bg_app`
//! or `theme().font.font_size`.

use std::path::Path;

use crossbeam_channel::Sender;
use floem::reactive::{RwSignal, SignalGet, use_context};
use notify::{RecursiveMode, Watcher};
use serde::Deserialize;

use crate::style::tokens::{ColorStyle, DimStyle, FontStyle};

/// Top-level theme, containing one sub-struct per toml section.
#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    pub color: ColorStyle,
    pub font: FontStyle,
    pub dim: DimStyle,
}

/// Parse a toml string into a `Theme` via serde.
fn parse_theme(toml_str: &str) -> Theme {
    toml::from_str(toml_str).expect("failed to parse theme toml")
}

// --- resolution -----------------------------------------------------------

type ThemeSignal = RwSignal<Theme>;

/// Fetch the current theme from Floem context.
///
/// Uses `get_untracked()` so call sites don't create individual reactive
/// dependencies — the top-level `dyn_container` in `app_view` is the single
/// dependency that triggers a full rebuild on theme change.
pub fn theme() -> Theme {
    let signal = use_context::<ThemeSignal>().expect("theme signal not provided");
    signal.get_untracked()
}

/// Collect all `.toml` file paths in `themes/`, sorted alphabetically.
///
/// Panics if the directory is missing or empty.
fn collect_theme_paths() -> Vec<std::path::PathBuf> {
    let mut paths: Vec<_> = std::fs::read_dir("themes")
        .expect("themes/ directory not found")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |e| e == "toml"))
        .collect();

    assert!(!paths.is_empty(), "no .toml files found in themes/");

    paths.sort();
    paths
}

/// Read and merge all `.toml` files in `themes/`. Later alphabetical files
/// override earlier ones. Returns `None` on any I/O or parse error.
///
/// This is the safe variant used during hot-reload — a bad save is silently
/// ignored so the old theme stays in place.
pub fn try_load_theme() -> Option<Theme> {
    let paths = collect_theme_paths();

    let mut merged = toml::Table::new();
    for path in &paths {
        let raw = std::fs::read_to_string(path).ok()?;
        let table: toml::Table = toml::from_str(&raw).ok()?;
        merge_tables(&mut merged, &table);
    }

    let merged_str = toml::to_string(&merged).ok()?;
    Some(parse_theme(&merged_str))
}

/// Same as [`try_load_theme`] but panics on failure — called once at boot so
/// the app never starts with a broken theme.
pub fn load_theme() -> Theme {
    try_load_theme().expect("failed to load theme at boot")
}

/// Recursively merge `overlay` into `base`. Nested tables are merged
/// recursively; all other values overwrite.
fn merge_tables(base: &mut toml::Table, overlay: &toml::Table) {
    for (key, value) in overlay {
        match value {
            toml::Value::Table(overlay_table) => {
                if let Some(toml::Value::Table(base_table)) = base.get_mut(key) {
                    merge_tables(base_table, overlay_table);
                } else {
                    base.insert(key.clone(), toml::Value::Table(overlay_table.clone()));
                }
            }
            _ => {
                base.insert(key.clone(), value.clone());
            }
        }
    }
}

// --- live reload ----------------------------------------------------------

/// Spawn an async file watcher task on the current tokio runtime that
/// monitors `themes/` for changes to `.toml` files and pushes reloaded
/// themes into `write`.
///
/// The watcher itself uses `notify` (synchronous) and bridges events to
/// an async channel. The watcher instance is intentionally leaked so it
/// lives for the lifetime of the program.
pub fn watch_theme_async(tx: Sender<Theme>) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send>> {
    let (async_tx, mut async_rx) = tokio::sync::mpsc::channel::<()>(10);

    let mut watcher = notify::RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                let is_toml = event
                    .paths
                    .iter()
                    .any(|p| p.extension().map_or(false, |e| e == "toml"));
                if is_toml {
                    let _ = async_tx.try_send(());
                }
            }
        },
        notify::Config::default(),
    )
    .expect("failed to create theme file watcher");

    // Start watching the themes/ directory.
    if watcher
        .watch(Path::new("themes"), RecursiveMode::NonRecursive)
        .is_err()
    {
        return Box::pin(async move {});
    }

    // Leak the watcher so it continues monitoring for the program's lifetime.
    std::mem::forget(watcher);

    Box::pin(async move {
        while let Some(_) = async_rx.recv().await {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                if async_rx.try_recv().is_err() {
                    break;
                }
            }
            let _ = try_load_theme().map(|t| tx.send(t));
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use floem::peniko::Color;
    use floem::text::Weight;

    #[test]
    fn parses_dark_toml_with_defaults() {
        let t = parse_theme(include_str!("../../themes/01-base.toml"));
        assert_eq!(t.color.bg_app, Color::rgb8(0x18, 0x19, 0x26));
        assert_eq!(t.color.text_primary, Color::rgb8(0xca, 0xd3, 0xf5));
        assert_eq!(t.dim.space_xs, 4.0);
        assert_eq!(t.dim.space_md, 12.0);
    }

    #[test]
    fn parses_font_style_from_toml() {
        let t = parse_theme(include_str!("../../themes/01-base.toml"));
        assert_eq!(t.font.mono_xl.family, "JetBrains Mono");
        assert_eq!(t.font.mono_xl.size, 24.0);
        assert_eq!(t.font.mono_xl.line_height, 30.0);
        assert_eq!(t.font.mono_xl.weight, Weight::BOLD);
        assert_eq!(t.font.mono_sm.family, "JetBrains Mono");
        assert_eq!(t.font.mono_sm.size, 11.0);
        assert_eq!(t.font.mono_sm.weight, Weight::NORMAL);
        assert_eq!(t.font.heading.family, "Segoe UI");
        assert_eq!(t.font.heading.weight, Weight::SEMIBOLD);
        assert_eq!(t.font.body_bold.weight, Weight::MEDIUM);
        assert_eq!(t.font.body.weight, Weight::NORMAL);
    }
}
