//! Bottom status bar — show timer, selection count, cue count, layout toggles.
//!
//! Layout (matching prototype):
//! [🕐 Show 00:14:22] | [🖱 1 selected] | 6 cues | [💾] [⚙️] | Layout [▤] [▦]

use floem::IntoView;
use floem::views::{Container, Decorators, Empty, Label, Stack};

use crate::style::theme;

use crate::ui::icons::{AppIcon, app_icon};

/// Status bar view.
pub fn view(selected_count: usize, cue_count: usize) -> impl IntoView {
    // Show timer
    let timer = Stack::horizontal((
        app_icon(
            AppIcon::Clock,
            theme().dim.icon_sm as f32,
            theme().color.text_secondary,
        ),
        Label::new("Show 00:14:22").style(|s| {
            s.font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
                .color(theme().color.text_primary)
        }),
    ))
    .style(|s| s.items_center().gap(theme().dim.space_sm));

    // Selection count
    let selection = Stack::horizontal((
        app_icon(
            AppIcon::MousePointerClick,
            theme().dim.icon_sm as f32,
            theme().color.text_secondary,
        ),
        Label::derived(move || format!("{} selected", selected_count)).style(|s| {
            s.font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
                .color(theme().color.text_secondary)
        }),
    ))
    .style(|s| s.items_center().gap(theme().dim.space_sm));

    // Cue count
    let cue_count_label = Label::derived(move || format!("{} cues", cue_count)).style(|s| {
        s.font_family(theme().font.mono_sm.family.clone())
            .font_size(theme().font.mono_sm.size)
            .color(theme().color.text_disabled)
    });

    // Global action buttons
    let save_btn = Container::new(app_icon(
        AppIcon::Save,
        theme().dim.icon_sm as f32,
        theme().color.text_secondary,
    ))
    .style(|s| {
        s.size(theme().dim.space_xl, theme().dim.space_xl)
            .items_center()
            .justify_center()
            .border_radius(theme().dim.radius_sm)
            .border(1.0)
            .border_color(theme().color.element_border)
            .background(theme().color.element_bg)
            .hover(|s| s.background(theme().color.element_bg_hover))
            .active(|s| s.background(theme().color.element_bg_active))
    });

    let settings_btn = Container::new(app_icon(
        AppIcon::Settings,
        theme().dim.icon_sm as f32,
        theme().color.text_secondary,
    ))
    .style(|s| {
        s.size(theme().dim.space_xl, theme().dim.space_xl)
            .items_center()
            .justify_center()
            .border_radius(theme().dim.radius_sm)
            .border(1.0)
            .border_color(theme().color.element_border)
            .background(theme().color.element_bg)
            .hover(|s| s.background(theme().color.element_bg_hover))
            .active(|s| s.background(theme().color.element_bg_active))
    });

    // Layout label
    let layout_label = Label::derived(|| "Layout".to_string()).style(|s| {
        s.font_family(theme().font.mono_sm.family.clone())
            .font_size(theme().font.mono_sm.size)
            .color(theme().color.text_disabled)
    });

    // Layout toggle buttons
    let columns_btn = Container::new(app_icon(
        AppIcon::Columns2,
        theme().dim.icon_sm as f32,
        theme().color.text_primary,
    ))
    .style(|s| {
        s.size(theme().dim.space_xl, theme().dim.space_xl)
            .items_center()
            .justify_center()
            .border_radius(theme().dim.radius_sm)
            .border(1.0)
            .border_color(theme().color.element_border)
            .background(theme().color.element_bg)
            .hover(|s| s.background(theme().color.element_bg_hover))
    });

    let rows_btn = Container::new(app_icon(
        AppIcon::Rows2,
        theme().dim.icon_sm as f32,
        theme().color.text_disabled,
    ))
    .style(|s| {
        s.size(theme().dim.space_xl, theme().dim.space_xl)
            .items_center()
            .justify_center()
            .border_radius(theme().dim.radius_sm)
            .border(1.0)
            .border_color(theme().color.element_border)
            .background(theme().color.element_bg)
            .hover(|s| s.background(theme().color.element_bg_hover))
    });

    // Assemble left section
    let left = Stack::horizontal((timer, selection, cue_count_label))
        .style(|s| s.items_center().gap(theme().dim.space_lg));

    // Assemble right section
    let right = Stack::horizontal((
        save_btn,
        settings_btn,
        Container::new(Empty::new()).style(|s| {
            s.width(1.0)
                .height(theme().dim.space_md)
                .background(theme().color.element_border)
        }),
        layout_label,
        columns_btn,
        rows_btn,
    ))
    .style(|s| s.items_center().gap(theme().dim.space_sm));

    Stack::horizontal((
        left,
        Container::new(Empty::new()).style(|s| s.flex_grow(1.0)),
        right,
    ))
    .style(|s| {
        s.items_center()
            .width_full()
            .gap(theme().dim.space_lg)
            .padding_horiz(theme().dim.space_md)
            .background(theme().color.bg_surface)
            .border_top(theme().dim.border_size)
            .border_color(theme().color.border_divider)
            .height(theme().dim.status_bar_height)
    })
}
