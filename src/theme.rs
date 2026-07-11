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

use std::sync::OnceLock;

use floem::peniko::Color;
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
        bg: (Color, Color::rgb8(0x1e, 0x1e, 0x1e)),
        panel: (Color, Color::rgb8(0x25, 0x25, 0x28)),
        panel_alt: (Color, Color::rgb8(0x2d, 0x2d, 0x31)),
        border: (Color, Color::rgb8(0x3a, 0x3a, 0x3f)),
        fg: (Color, Color::rgb8(0xd4, 0xd4, 0xd4)),
        text_dim: (Color, Color::rgb8(0x8a, 0x8a, 0x8a)),
        text_faint: (Color, Color::rgb8(0x5a, 0x5a, 0x60)),
        accent: (Color, Color::rgb8(0x3f, 0xb9, 0x50)),
        accent_dim: (Color, Color::rgb8(0x2f, 0x8a, 0x3b)),
        lapce_blue: (Color, Color::rgb8(0x18, 0x90, 0xff)),
        panic: (Color, Color::rgb8(0xf1, 0x4c, 0x4c)),
        panic_dim: (Color, Color::rgb8(0xb8, 0x30, 0x30)),
        meter: (Color, Color::rgb8(0x4e, 0xc9, 0x5a)),
    },
    font: {
        mono: (String, "monospace".to_string()),
        // font_size: (f64, 13.0),
    },
    panel: {
        default_left_size: (f64, 220.0),
        default_right_size: (f64, 260.0),
        default_bottom_size: (f64, 240.0),
        handle_width: (f64, 4.0),
        min_left_size: (f64, 140.0),
        min_right_size: (f64, 180.0),
        min_bottom_size: (f64, 120.0),
        border_width: (f64, 1.0),
        scroll_bar_width: (f64, 4.0),
    },
}

// --- resolution -----------------------------------------------------------

static THEME: OnceLock<Theme> = OnceLock::new();

/// The resolved, immutable theme. Bootstrapped on first use from the first
/// `.toml` file found in `themes/` (theme switching arrives later). Sections
/// mirror the toml file and the declaration dict, e.g. `theme().color.bg`.
pub fn theme() -> &'static Theme {
    THEME.get_or_init(load_theme)
}

/// Grab the first `.toml` file in `themes/` and parse it as an override layer
/// on top of the baked-in defaults. Falls back to the compiled-in default so
/// the app always boots even if the directory is missing or empty.
fn load_theme() -> Theme {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dark_toml_with_defaults() {
        let t = parse_theme(include_str!("../themes/dark.toml"));
        assert_eq!(t.font.mono, "monospace");
        assert_eq!(t.color.bg, Color::rgb8(0x1e, 0x1e, 0x1e));
        assert_eq!(t.color.accent, Color::rgb8(0x3f, 0xb9, 0x50));
        assert_eq!(t.panel.default_left_size, 220.0);
        assert_eq!(t.panel.default_right_size, 260.0);
        assert_eq!(t.panel.default_bottom_size, 240.0);
        assert_eq!(t.panel.handle_width, 4.0);
        assert_eq!(t.panel.min_left_size, 140.0);
        assert_eq!(t.panel.min_right_size, 180.0);
        assert_eq!(t.panel.min_bottom_size, 120.0);
        assert_eq!(t.panel.border_width, 1.0);
    }

    #[test]
    fn missing_values_fall_back_to_defaults() {
        let t = parse_theme("[color]\n# empty\n");
        assert_eq!(t.color.bg, Theme::default().color.bg);
        assert_eq!(t.color.accent, Theme::default().color.accent);
        assert_eq!(t.font.mono, Theme::default().font.mono);
        assert_eq!(
            t.panel.default_left_size,
            Theme::default().panel.default_left_size
        );
    }

    #[test]
    fn unparseable_color_falls_back_to_default() {
        let t = parse_theme("[color]\nbg = \"not-a-colour\"\n");
        assert_eq!(t.color.bg, Theme::default().color.bg);
    }

    #[test]
    fn panel_values_parsed_from_toml() {
        let t = parse_theme("[panel]\nhandle_width = 8\nmin_left_size = 200\nborder_width = 2\n");
        assert_eq!(t.panel.handle_width, 8.0);
        assert_eq!(t.panel.min_left_size, 200.0);
        assert_eq!(t.panel.border_width, 2.0);
        // defaults kept for unset fields
        assert_eq!(
            t.panel.default_left_size,
            Theme::default().panel.default_left_size
        );
    }
}
