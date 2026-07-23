//! Global stylesheet for the Articuelate application.

use floem::peniko::Color;
use floem::style::Style;
use floem::style_class;
use floem::views::ButtonClass;
use floem::views::scroll::{ScrollClass, ScrollCustomStyle};

use crate::style::theme::theme;

style_class!(pub StatusBarButton);

/// Apply global class-based styles to the base view.
///
/// These styles are applied once and cascade to all matching views.
pub fn global_stylesheet(s: Style) -> Style {
    s.class(ScrollClass, |s| {
        s.size_full()
            .min_size(0.0, 0.0)
            .apply_custom(ScrollCustomStyle::new().handle_thickness(theme().dim.space_xs))
    })
    .class(StatusBarButton, |s| {
        s.color(theme().color.text_primary)
            .background(Color::TRANSPARENT)
            .border_color(Color::TRANSPARENT)
            .font_size(theme().dim.status_icon_size)
            .border_radius(theme().dim.radius_sm)
            .padding_horiz(theme().dim.space_xs)
    })
    .class(ButtonClass, |s| apply_interactable_base_styles(s))
}

fn apply_interactable_base_styles(s: Style) -> Style {
    s.background(theme().color.bg_overlay)
        .hover(|s| s.background(theme().color.bg_hover))
        .active(|s| s.background(Color::TRANSPARENT))
        .focus(|s| {
            s.hover(|s| s.background(theme().color.bg_hover))
                .active(|s| s.background(Color::TRANSPARENT))
                .active(|s| s.hover(|s| s.background(theme().color.bg_hover)))
        })
}
