//! Style token definitions for Articuelate.
//!
//! These types mirror the `[section]` tables in the `.toml` theme file.
//! Renamed with a `Style` suffix to distinguish them from the theme-loading
//! infrastructure.

use floem::peniko::Color;
use floem::text::Weight;
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

// --- style structs --------------------------------------------------------

/// Colour attributes.
#[serde_with::apply(
    Color => #[serde_as(as = "ColorParser")],
)]
#[serde_with::serde_as]
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ColorStyle {
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

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct FontRole {
    pub family: String,
    pub size: f64,
    pub line_height: f64,
    #[serde(deserialize_with = "de_weight")]
    pub weight: Weight,
}

/// Font / typography attributes.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct FontStyle {
    pub mono_sm: FontRole,
    pub mono_xl: FontRole,
    pub heading: FontRole,
    pub body_bold: FontRole,
    pub body: FontRole,
}

/// Dimension / spacing attributes.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct DimStyle {
    pub space_xs: f64,
    pub space_sm: f64,
    pub space_md: f64,
    pub space_lg: f64,
    pub space_xl: f64,
    pub height_cue_row: f64,
    pub min_panel_size: f64,
}
