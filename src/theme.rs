//! Theming for Articuelate.
//!
//! Each section struct mirrors a `[section]` table in the `.toml` theme file
//! and derives `Deserialize`. Adding a new themable attribute is a single
//! two-line edit: add the field to the struct here, then add its value to
//! the theme file. No boilerplate needs updating.
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
use floem::{
    peniko::Color,
    reactive::{RwSignal, SignalGet, use_context},
    style::Style,
    text::Weight,
    views::scroll::{ScrollClass, ScrollCustomStyle},
};
use notify::{RecursiveMode, Watcher};
use serde::Deserialize;
use serde_with::DeserializeAs;

// --- custom deserializer helpers ------------------------------------------

fn de_weight<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Weight, D::Error> {
    let s = String::deserialize(d)?;
    let weight = match s.to_lowercase().as_str() {
        "thin" => Weight::THIN,
        "extralight" | "extra light" => Weight::EXTRA_LIGHT,
        "light" => Weight::LIGHT,
        "normal" | "regular" => Weight::NORMAL,
        "medium" => Weight::MEDIUM,
        "semibold" | "semi bold" => Weight::SEMIBOLD,
        "bold" => Weight::BOLD,
        "extrabold" | "extra bold" => Weight::EXTRA_BOLD,
        "black" => Weight::BLACK,
        _ => {
            return Err(serde::de::Error::custom(format!(
                "unknown font weight `{s}`"
            )));
        }
    };
    Ok(weight)
}

pub struct ColorParser;

impl<'de> DeserializeAs<'de, Color> for ColorParser {
    fn deserialize_as<D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Call your exact original custom deserializer helper here
        de_color(deserializer)
    }
}

fn de_color<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Color, D::Error> {
    let s = String::deserialize(d)?;
    hex_to_color(&s).map_err(serde::de::Error::custom)
}

fn hex_to_color(s: &str) -> Result<Color, String> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return Err(format!("invalid hex colour `{s}`"));
    }
    let r = u8::from_str_radix(&s[0..2], 16).map_err(|_| format!("invalid hex colour `{s}`"))?;
    let g = u8::from_str_radix(&s[2..4], 16).map_err(|_| format!("invalid hex colour `{s}`"))?;
    let b = u8::from_str_radix(&s[4..6], 16).map_err(|_| format!("invalid hex colour `{s}`"))?;
    Ok(Color::rgb8(r, g, b))
}

// --- font style helper type -----------------------------------------------

/// A bundle of typography attributes for a single text role.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct FontStyle {
    pub family: String,
    pub size: f64,
    pub line_height: f64,
    #[serde(deserialize_with = "de_weight")]
    pub weight: Weight,
}

// --- theme section structs ------------------------------------------------

/// Colour attributes.

#[serde_with::apply(
    Color => #[serde_as(as = "ColorParser")],
)]
#[serde_with::serde_as]
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ColorTheme {
    pub bg_app: Color,
    pub bg_surface: Color,
    pub bg_surface_odd: Color,
    pub bg_hover: Color,
    pub bg_overlay: Color,
    pub border_subtle: Color,
    pub border_focus: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    pub status_active: Color,
    pub status_status_running: Color,
    pub status_wait: Color,
    pub status_error: Color,
    pub status_standby: Color,
}

/// Font / typography attributes.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct FontTheme {
    pub mono_sm: FontStyle,
    pub mono_xl: FontStyle,
    pub heading: FontStyle,
    pub body_bold: FontStyle,
    pub body: FontStyle,
}

/// Dimension / spacing attributes.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct DimTheme {
    pub space_xs: f64,
    pub space_sm: f64,
    pub space_md: f64,
    pub space_lg: f64,
    pub space_xl: f64,
    pub height_cue_row: f64,
    pub min_panel_size: f64,
}

/// Top-level theme, containing one sub-struct per toml section.
#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    pub color: ColorTheme,
    pub font: FontTheme,
    pub dim: DimTheme,
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

pub fn global_stylesheet(s: Style) -> Style {
    s.class(ScrollClass, |s| {
        s.size_full()
            .min_size(0.0, 0.0)
            .apply_custom(ScrollCustomStyle::new().handle_thickness(theme().dim.space_xs))
    })
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

    #[test]
    fn parses_dark_toml_with_defaults() {
        let t = parse_theme(include_str!("../themes/01-base.toml"));
        assert_eq!(t.color.bg_app, Color::rgb8(0x18, 0x19, 0x26));
        assert_eq!(t.color.text_primary, Color::rgb8(0xca, 0xd3, 0xf5));
        assert_eq!(t.dim.space_xs, 4.0);
        assert_eq!(t.dim.space_md, 12.0);
    }

    #[test]
    fn parses_font_style_from_toml() {
        let t = parse_theme(include_str!("../themes/01-base.toml"));
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
