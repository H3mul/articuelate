//! Theming for Articuelate.
//!
//! Every themable attribute is declared in exactly one place — the
//! [`define_theme!`] invocation at the bottom of this file. That single dict is
//! the source of truth for each attribute's name, type and default, and it
//! generates the `Theme` struct (with one sub-struct per *section*), the
//! baked-in `Default` values, the toml override parser, and the `theme()`
//! accessor. Adding a new themable attribute is therefore a single edit: add
//! one line to the dict.
//!
//! A `.toml` file in `themes/` acts as a styling layer *on top of* those
//! defaults: at boot we load the first `.toml` we find and, for each
//! attribute, override the default only when the file provides a value that
//! parses into the expected type — otherwise the baked-in default stands. This
//! makes parsing safe: the app always boots.
//!
//! Access resolved values through [`theme()`], whose sections mirror the toml
//! file and the declaration dict, e.g. `theme().color.bg` or
//! `theme().font.font_size`.

use floem::{
    peniko::Color,
    reactive::{ReadSignal, SignalGet, WriteSignal, use_context},
    style::Style,
    text::Weight,
    views::scroll::{ScrollClass, ScrollCustomStyle},
};
use toml::Value;

// --- toml value extraction ------------------------------------------------

/// Extract a typed value from a `toml::Value`, returning `None` when the value
/// is missing or the wrong type. This is what makes parsing safe: a bad value
/// is ignored and the baked-in default is kept.
trait FromTomlValue: Sized {
    fn from_value(v: &Value) -> Option<Self>;
}

impl FromTomlValue for Color {
    fn from_value(v: &Value) -> Option<Color> {
        v.as_str().and_then(|s| hex_to_color(s).ok())
    }
}

impl FromTomlValue for f64 {
    fn from_value(v: &Value) -> Option<f64> {
        v.as_float().or_else(|| v.as_integer().map(|i| i as f64))
    }
}

impl FromTomlValue for String {
    fn from_value(v: &Value) -> Option<String> {
        v.as_str().map(str::to_string)
    }
}

impl FromTomlValue for Weight {
    fn from_value(v: &Value) -> Option<Weight> {
        let s = v.as_str()?;
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
                let n: u16 = s.parse().ok()?;
                Weight(n)
            }
        };
        Some(weight)
    }
}

impl FromTomlValue for f32 {
    fn from_value(v: &Value) -> Option<f32> {
        v.as_float()
            .or_else(|| v.as_integer().map(|i| i as f64))
            .map(|f| f as f32)
    }
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

/// A bundle of typography attributes for a single text role. Each entry in the
/// `[font]` section of the toml file (e.g. `font.mono_xl`) maps to one of
/// these, making it easy to apply in Floem:
///
/// ```ignore
/// label(|| "Hello")
///     .style(|s| {
///         s.font_family(theme().font.mono_xl.family.clone())
///          .font_size(theme().font.mono_xl.size)
///          .line_height(theme().font.mono_xl.line_height)
///          .font_weight(theme().font.mono_xl.weight)
///     });
/// ```
#[derive(Debug, Clone)]
pub struct FontStyle {
    pub family: String,
    pub size: f32,
    pub line_height: f32,
    pub weight: Weight,
}

impl FromTomlValue for FontStyle {
    fn from_value(v: &Value) -> Option<FontStyle> {
        let t = v.as_table()?;
        Some(FontStyle {
            family: t.get("font_family").and_then(FromTomlValue::from_value)?,
            size: t
                .get("font_size")
                .and_then(FromTomlValue::from_value)
                .unwrap_or(14.0),
            line_height: t
                .get("line_height")
                .and_then(FromTomlValue::from_value)
                .unwrap_or(20.0),
            weight: t
                .get("font_weight")
                .and_then(FromTomlValue::from_value)
                .unwrap_or(Weight::NORMAL),
        })
    }
}

// --- theme definition macro ----------------------------------------------

/// Generates the `Theme` struct (with one sub-struct per *section*), their
/// `Default` impls (the baked-in defaults), the toml override parser, and the
/// [`theme()`] accessor — all from a single dict:
///
/// ```text
/// define_theme! {
///     <section>: {
///         <field>: (<Type>, <default literal>),
///         ...
///     },
///     ...
/// }
/// ```
///
/// `<section>` becomes a `[section]` table in the toml file *and* a `pub`
/// sub-struct field on `Theme` (e.g. `theme().color.bg`). `<field>` is the
/// struct field, the toml key, and the accessor path component — all
/// identical.
macro_rules! define_theme {
    (
        $(
            $section:ident : {
                $(
                    $field:ident : ( $ty:ty, $default:expr )
                ),+ $(,)?
            }
        ),+ $(,)?
    ) => {
        $(
            #[derive(Debug, Clone)]
            #[allow(non_camel_case_types)]
            pub struct $section {
                $(
                    pub $field: $ty,
                )+
            }
        )+

        #[derive(Debug, Clone)]
        pub struct Theme {
            $(
                pub $section: $section,
            )+
        }

        $(
            impl Default for $section {
                fn default() -> Self {
                    Self {
                        $(
                            $field: $default,
                        )+
                    }
                }
            }
        )+

        impl Default for Theme {
            fn default() -> Self {
                Self {
                    $(
                        $section: $section::default(),
                    )+
                }
            }
        }

        /// Parse a toml string into a `Theme`, layering any provided values
        /// over the baked-in defaults. A malformed *file* is a hard error; a
        /// missing or unparseable individual value keeps its default.
        fn parse_theme(toml_str: &str) -> Theme {
            let doc: Value = toml::from_str(toml_str).expect("failed to parse theme toml");
            Theme {
                $(
                    $section: $section {
                        $(
                            $field: doc
                                .get(stringify!($section))
                                .and_then(|s| s.get(stringify!($field)))
                                .and_then(<$ty>::from_value)
                                .unwrap_or_else(|| $default),
                        )+
                    },
                )+
            }
        }
    };
}

define_theme! {
    color: {
        // Deepest base canvas background (window background).
        bg_app: (Color, Color::rgb8(0x18, 0x19, 0x26)),
        // Primary pane panels (Cuelist background, Inspector background).
        bg_surface: (Color, Color::rgb8(0x1e, 0x20, 0x30)),
        // Primary pane panel alternation (Cuelist background rows)
        bg_surface_odd: (Color, Color::rgb8(0x24, 0x27, 0x3a)),
        // Hover states, active tab backgrounds, secondary buttons.
        bg_hover: (Color, Color::rgb8(0x36, 0x3a, 0x4f)),
        // Input controls, table headers, inactive rows, cards.
        bg_overlay: (Color, Color::rgb8(0x49, 0x4d, 0x64)),

        // Standard divider for table rows and pane borders.
        border_subtle: (Color, Color::rgb8(0x6e, 0x73, 0x8d)),
        // Focused input borders, active tab indicators.
        border_focus: (Color, Color::rgb8(0x80, 0x87, 0xa2)),

        // High-contrast body, cue titles, times. Primary legibility.
        text_primary: (Color, Color::rgb8(0xca, 0xd3, 0xf5)),
        // Column headers, cue numbers, inactive tabs, unit labels (sec, dB).
        text_secondary: (Color, Color::rgb8(0xb8, 0xc0, 0xe0)),
        // Disabled cues, muted parameters.
        text_disabled: (Color, Color::rgb8(0xa5, 0xad, 0xcb)),

        // Active Playhead / Selected Target (Cyan Blue).
        status_active: (Color, Color::rgb8(0x8a, 0xad, 0xf4)),
        // Active Playing Cue (Green glow/bar).
        status_status_running: (Color, Color::rgb8(0xa6, 0xda, 0x95)),
        // Pre-Wait / Post-Wait Active (Amber).
        status_wait: (Color, Color::rgb8(0xee, 0xd4, 0x9f)),
        // Broken Cue / Panic Trigger (Red).
        status_error: (Color, Color::rgb8(0xee, 0x99, 0xa0)),
        // Armed / Standby State (Purple).
        status_standby: (Color, Color::rgb8(0xc6, 0xa0, 0xf6)),
    },
    font: {
        mono: (String, "monospace".to_string()),
        font_size: (f32, 13.0),
        mono_sm: (FontStyle, FontStyle {
            family: "JetBrains Mono".to_string(),
            size: 11.0,
            line_height: 14.0,
            weight: Weight::NORMAL,
        }),
        mono_xl: (FontStyle, FontStyle {
            family: "JetBrains Mono".to_string(),
            size: 24.0,
            line_height: 30.0,
            weight: Weight::BOLD,
        }),
        heading: (FontStyle, FontStyle {
            family: "Segoe UI".to_string(),
            size: 15.0,
            line_height: 20.0,
            weight: Weight::SEMIBOLD,
        }),
        body_bold: (FontStyle, FontStyle {
            family: "Segoe UI".to_string(),
            size: 13.0,
            line_height: 18.0,
            weight: Weight::MEDIUM,
        }),
        body: (FontStyle, FontStyle {
            family: "Segoe UI".to_string(),
            size: 13.0,
            line_height: 18.0,
            weight: Weight::NORMAL,
        }),
    },
    dim: {
        space_xs: (f32, 4.0),
        space_sm: (f32, 8.0),
        space_md: (f32, 12.0),
        space_lg: (f32, 16.0),
        space_xl: (f32, 24.0),
        height_cue_row: (f32, 24.0),
    },
}

// --- resolution -----------------------------------------------------------

/// Type alias for the theme signal tuple passed through Floem context.
pub type ThemeSignal = (ReadSignal<Theme>, WriteSignal<Theme>);

/// Fetch the current theme from Floem context.
///
/// Uses `get_untracked()` so call sites don't create individual reactive
/// dependencies — the top-level `move ||` in [`app_view`] is the single
/// dependency that triggers a full rebuild on theme change.
pub fn theme() -> Theme {
    let (rx, _) = use_context::<ThemeSignal>().expect("theme signal not provided");
    rx.get_untracked()
}

/// Grab the first `.toml` file in `themes/` and parse it as an override layer
/// on top of the baked-in defaults. Falls back to the compiled-in default so
/// the app always boots even if the directory is missing or empty.
pub fn load_theme() -> Theme {
    let first = std::fs::read_dir("themes").ok().and_then(|entries| {
        entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .find(|p| p.extension().map_or(false, |e| e == "toml"))
    });

    match first {
        Some(path) => {
            let raw = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read theme file {:?}: {e}", path));
            parse_theme(&raw)
        }
        None => parse_theme(include_str!("../themes/dark.toml")),
    }
}

pub fn global_stylesheet(s: Style) -> Style {
    s.class(ScrollClass, |s| {
        s.size_full()
            .min_size(0.0, 0.0)
            .apply_custom(ScrollCustomStyle::new().handle_thickness(theme().dim.space_xs))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dark_toml_with_defaults() {
        let t = parse_theme(include_str!("../themes/dark.toml"));
        assert_eq!(t.font.mono, "monospace");
        assert_eq!(t.color.bg_app, Color::rgb8(0x18, 0x19, 0x26));
        assert_eq!(t.color.text_primary, Color::rgb8(0xca, 0xd3, 0xf5));
        assert_eq!(t.dim.space_xs, 4.0);
        assert_eq!(t.dim.space_md, 12.0);
    }

    #[test]
    fn parses_font_style_from_toml() {
        let t = parse_theme(include_str!("../themes/dark.toml"));
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
        assert_eq!(t.font.font_size, 13.0);
    }

    #[test]
    fn missing_values_fall_back_to_defaults() {
        let t = parse_theme("[color]\n# empty\n");
        assert_eq!(t.color.bg_app, Theme::default().color.bg_app);
        assert_eq!(t.color.text_primary, Theme::default().color.text_primary);
        assert_eq!(t.font.mono, Theme::default().font.mono);
    }

    #[test]
    fn font_style_falls_back_to_defaults() {
        let t = parse_theme("[font]\n# empty\n");
        assert_eq!(t.font.mono_xl.family, Theme::default().font.mono_xl.family);
        assert_eq!(t.font.mono_xl.size, Theme::default().font.mono_xl.size);
        assert_eq!(
            t.font.mono_xl.line_height,
            Theme::default().font.mono_xl.line_height
        );
        assert_eq!(t.font.mono_xl.weight, Theme::default().font.mono_xl.weight);
    }

    #[test]
    fn unparseable_color_falls_back_to_default() {
        let t = parse_theme("[color]\nbg_app = \"not-a-colour\"\n");
        assert_eq!(t.color.bg_app, Theme::default().color.bg_app);
    }

    #[test]
    fn unparseable_font_style_falls_back_to_default() {
        let t = parse_theme("[font.mono_xl]\nfont_family = 42\n");
        assert_eq!(t.font.mono_xl.family, Theme::default().font.mono_xl.family);
        assert_eq!(t.font.mono_xl.size, Theme::default().font.mono_xl.size);
    }

    #[test]
    fn font_style_defaults_from_missing_toml_table() {
        // When the [font.mono_xl] table is entirely absent, the default is
        // used because none of the table-access path produces a Value.
        let t = parse_theme("[font]\nmono = \"foo\"\n");
        assert_eq!(t.font.mono_xl.family, Theme::default().font.mono_xl.family);
    }
}
