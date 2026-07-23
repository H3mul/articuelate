//! Global stylesheet for the Articuelate application.

use floem::peniko::Color;
use floem::style::Style;
use floem::style_class;
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
        s.background(Color::TRANSPARENT)
            .color(theme().color.text_primary)
            .border(0.0)
            .font_size(theme().font.body.size)
            .hover(|s| s.background(theme().color.border_subtle))
    })
}
