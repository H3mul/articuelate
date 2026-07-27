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
    Color::parse(&s.trim())
        .ok_or_else(|| format!("invalid colour `{s}`"))
        .map_err(serde::de::Error::custom)
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
    pub bg_surface_raised: Color,
    pub bg_surface_overlay: Color,

    pub element_bg: Color,
    pub element_border: Color,
    pub element_bg_hover: Color,
    pub element_bg_active: Color,

    pub bg_selection: Color,
    pub bg_selection_active: Color,
    pub border_focus: Color,

    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,

    pub status_playhead: Color,
    pub status_running: Color,
    pub status_wait: Color,
    pub status_error: Color,
    pub status_standby: Color,
    pub status_group: Color,

    pub border_subtle: Color,
    pub border_divider: Color,
    pub border_emphasized: Color,

    pub status_running_bg: Color,
    pub border_row_divider: Color,
    pub status_playhead_bg: Color,
    pub status_running_bg_30: Color,
    pub status_running_bg_20: Color,
    pub status_running_bg_70: Color,
    pub status_group_bg: Color,
    pub status_error_bg: Color,
    pub status_error_bg_12: Color,
    pub status_group_bg_25: Color,
    pub border_divider_40: Color,
    pub text_disabled_50: Color,

    pub slider_track: Color,
    pub slider_fill: Color,
    pub slider_thumb: Color,
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
    pub border_size: f64,

    pub status_bar_height: f64,
    pub status_icon_size: f64,
    pub toolbar_height: f64,

    pub radius_sm: f64,
    pub radius_md: f64,
    pub radius_full: f64,

    pub control_sm: f64,
    pub control_md: f64,
    pub time_cell: f64,

    pub col_playhead: f64,
    pub col_drag: f64,
    pub col_cue_number: f64,
    pub col_time: f64,
    pub col_menu: f64,

    pub led_dot: f64,
    pub meter_width_sm: f64,
    pub meter_width_md: f64,
    pub dot_sm: f64,

    pub sidebar_width: f64,
    pub detail_height: f64,
    pub active_card_height: f64,
    pub btn_go_width: f64,
    pub btn_panic_width: f64,
    pub textarea_height: f64,

    pub icon_sm: f64,
    pub icon_md: f64,

    pub slider_track_height: f64,
    pub slider_thumb_size: f64,
}
