//! Top transport toolbar: Pause, GO, Back, global search, and Panic.
//!
//! Flat, icon-style buttons in the Lapce tradition (icons from `lucide-floem`).
//! GO is the prominent theatre-green action; PANIC is red. Panel toggles live
//! here too and flip the shared `PanelVisible` signal owned by the panel system.

use floem::IntoView;
use floem::peniko::Color;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate, SignalWith, create_rw_signal};
use floem::views::{Decorators, button, h_stack, label, text, text_input};
use lucide_floem::Icon;

use crate::panel::PanelVisible;
use crate::theme::*;

/// Which optional panel a toggle controls.
#[derive(Clone, Copy)]
enum PanelWhich {
    Left,
    Right,
    Bottom,
}

#[allow(clippy::too_many_arguments)]
pub fn view(
    cues_len: usize,
    active_cue: RwSignal<usize>,
    selected: RwSignal<usize>,
    search: RwSignal<String>,
    visible: RwSignal<PanelVisible>,
) -> impl IntoView {
    let paused = create_rw_signal(false);

    let pause_btn = button(icon(
        Icon::Pause,
        toggle_color(paused, theme().color.fg),
        16.0,
    ))
    .action(move || paused.update(|p| *p = !*p))
    .style(button_style());

    let go_btn = button(h_stack((
        icon(Icon::Play, theme().color.accent, 14.0),
        text("GO").style(|s| {
            s.color(theme().color.bg)
                .font_weight(floem::text::Weight::BOLD)
                .font_size(13.0)
        }),
    )))
    .action(move || {
        let next = (active_cue.get() + 1).min(cues_len.saturating_sub(1));
        active_cue.set(next);
        selected.set(next);
    })
    .style(|s| {
        s.background(theme().color.accent)
            .color(theme().color.bg)
            .padding_horiz(16.0)
            .padding_vert(6.0)
            .border_radius(4.0)
            .gap(6.0)
            .hover(|s| s.background(theme().color.accent_dim))
    });

    let back_btn = button(icon(Icon::SkipBack, theme().color.fg, 16.0))
        .action(move || {
            let prev = active_cue.get().saturating_sub(1);
            active_cue.set(prev);
            selected.set(prev);
        })
        .style(button_style());

    let search_icon = icon(Icon::Search, theme().color.text_dim, 14.0);
    let search_box = text_input(search)
        .placeholder("Search cues…")
        .style(|s| {
            s.background(theme().color.panel_alt)
                .color(theme().color.fg)
                .border(1.0)
                .border_color(theme().color.border)
                .border_radius(4.0)
                .padding_horiz(8.0)
                .padding_vert(4.0)
                .width(200.0)
                .font_size(12.0)
        })
        .keyboard_navigable();
    let search_wrap = h_stack((search_icon, search_box)).style(|s| s.items_center().gap(6.0));

    let bottom_toggle = panel_toggle(Icon::PanelBottom, PanelWhich::Bottom, visible);
    let left_toggle = panel_toggle(Icon::PanelLeft, PanelWhich::Left, visible);
    let right_toggle = panel_toggle(Icon::PanelRight, PanelWhich::Right, visible);

    let cue_readout = label(move || {
        format!(
            "CUE {}/{}",
            active_cue.get().saturating_add(1).min(cues_len),
            cues_len
        )
    })
    .style(|s| {
        s.font_family(theme().font.mono.to_string())
            .color(theme().color.text_dim)
            .font_size(12.0)
    });

    let panic_btn = button(h_stack((
        icon(Icon::Ban, theme().color.panic, 14.0),
        text("PANIC").style(|s| {
            s.color(theme().color.panic)
                .font_weight(floem::text::Weight::BOLD)
                .font_size(12.0)
        }),
    )))
    .action(move || active_cue.set(0))
    .style(|s| {
        s.background(theme().color.panel_alt)
            .border(1.0)
            .border_color(theme().color.panic_dim)
            .border_radius(4.0)
            .padding_horiz(12.0)
            .padding_vert(6.0)
            .gap(6.0)
            .hover(|s| s.background(theme().color.panic_dim.multiply_alpha(0.25)))
    });

    h_stack((
        pause_btn,
        go_btn,
        back_btn,
        search_wrap,
        left_toggle,
        bottom_toggle,
        right_toggle,
        text("|").style(|s| s.color(theme().color.text_faint)),
        cue_readout,
        // spacer pushes PANIC to the far right
        text("").style(|s| s.flex_grow(1.0)),
        panic_btn,
    ))
    .style(|s| {
        s.items_center()
            .gap(8.0)
            .padding_horiz(10.0)
            .padding_vert(6.0)
            .background(theme().color.panel)
            .border_bottom(1.0)
            .border_color(theme().color.border)
            .height(44.0)
    })
}

/// A lucide icon styled to a fixed size + colour.
fn icon(icon: Icon, color: Color, size: f32) -> impl IntoView {
    icon.into_view()
        .style(move |s| s.size(size, size).color(color))
}

/// Colour for a toggle that reflects its paused/active state.
fn toggle_color(paused: RwSignal<bool>, on: Color) -> Color {
    if paused.get() {
        theme().color.accent_dim
    } else {
        on
    }
}

/// A small chevron/panel icon that toggles an optional panel.
fn panel_toggle(icon: Icon, which: PanelWhich, visible: RwSignal<PanelVisible>) -> impl IntoView {
    let shown = move || {
        visible.with(|v| match which {
            PanelWhich::Left => v.left,
            PanelWhich::Right => v.right,
            PanelWhich::Bottom => v.bottom,
        })
    };
    let child = icon.into_view().style(move |s| {
        s.size(16.0, 16.0).color(if shown() {
            theme().color.accent
        } else {
            theme().color.text_dim
        })
    });
    button(child)
        .action(move || {
            visible.update(|v| match which {
                PanelWhich::Left => v.left = !v.left,
                PanelWhich::Right => v.right = !v.right,
                PanelWhich::Bottom => v.bottom = !v.bottom,
            })
        })
        .style(|s| {
            s.background(theme().color.panel_alt)
                .color(theme().color.fg)
                .border_radius(4.0)
                .padding_horiz(8.0)
                .padding_vert(6.0)
                .font_size(12.0)
                .hover(|s| s.background(theme().color.border))
        })
}

/// Shared base styling for the plain icon buttons.
fn button_style() -> impl Fn(floem::style::Style) -> floem::style::Style + 'static {
    move |s: floem::style::Style| {
        s.background(theme().color.panel_alt)
            .color(theme().color.fg)
            .border_radius(4.0)
            .padding_horiz(10.0)
            .padding_vert(6.0)
            .font_size(14.0)
            .min_width(34.0)
            .hover(|s| s.background(theme().color.border))
    }
}
