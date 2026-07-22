//! Top transport toolbar: Pause, GO, Back, global search, and Panic.
//!
//! Flat, icon-style buttons in the Lapce tradition (icons from `lucide-floem`).
//! GO is the prominent theatre-green action; PANIC is red. Panel toggles live
//! here too and flip the shared `PanelVisible` signal owned by the panel system.

use floem::IntoView;
use floem::peniko::Color;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate, SignalWith, create_rw_signal};
use floem::taffy::Display;
use floem::views::{Decorators, button, h_stack, label, text, text_input};
use lucide_floem::Icon;
use tokio::sync::mpsc;

use std::sync::Arc;

use crate::exec::UiEvent;
use crate::model::{CueId, Cuelist};
use crate::panel::PanelFlags;
use crate::theme::*;

/// Which optional panel a toggle controls.
#[derive(Clone, Copy)]
enum PanelWhich {
    Left,
    Right,
    Bottom,
}

pub fn view(
    cuelist: impl SignalGet<Arc<Cuelist>> + SignalWith<Arc<Cuelist>> + Copy + 'static,
    active_cue: RwSignal<Option<CueId>>,
    selected: RwSignal<Option<CueId>>,
    search: RwSignal<String>,
    active: RwSignal<PanelFlags>,
    visible: RwSignal<PanelFlags>,
    events: mpsc::Sender<UiEvent>,
) -> impl IntoView {
    let paused = create_rw_signal(false);

    let pause_btn = button(icon(
        Icon::Pause,
        toggle_color(paused, theme().color.text_primary),
        16.0,
    ))
    .action(move || paused.update(|p| *p = !*p))
    .style(button_style());

    let go_btn = button(h_stack((
        icon(Icon::Play, theme().color.status_active, 14.0),
        text("GO").style(|s| {
            s.color(theme().color.bg_app)
                .font_weight(floem::text::Weight::BOLD)
                .font_size(13.0)
        }),
    )))
    .action(move || {
        // Push the Go intent onto the Execution Thread event bus. The Exec
        // Thread advances the playhead and broadcasts the new state back; the
        // UI mirrors it into its active/selected cues via ingestion.
        let _ = events.try_send(UiEvent::Go);
    })
    .style(|s| {
        s.background(theme().color.status_active)
            .color(theme().color.bg_app)
            .padding_horiz(16.0)
            .padding_vert(6.0)
            .border_radius(4.0)
            .gap(6.0)
            .hover(|s| s.background(theme().color.status_active))
    });

    let back_btn = button(icon(Icon::SkipBack, theme().color.text_primary, 16.0))
        .action(move || {
            let cl = cuelist.get();
            if let Some(id) = active_cue.get() {
                if let Some(pos) = cl.iter().position(|c| c.id == id) {
                    if pos > 0 {
                        if let Some(prev) = cl.iter().nth(pos - 1) {
                            let prev_id = prev.id;
                            active_cue.set(Some(prev_id));
                            selected.set(Some(prev_id));
                        }
                    }
                }
            }
        })
        .style(button_style());

    let search_icon = icon(Icon::Search, theme().color.text_secondary, 14.0);
    let search_box = text_input(search)
        .placeholder("Search cues…")
        .style(|s| {
            s.background(theme().color.bg_overlay)
                .color(theme().color.text_primary)
                .border(1.0)
                .border_color(theme().color.border_subtle)
                .border_radius(4.0)
                .padding_horiz(8.0)
                .padding_vert(4.0)
                .width(200.0)
                .font_size(12.0)
        })
        .keyboard_navigable();
    let search_wrap = h_stack((search_icon, search_box)).style(|s| s.items_center().gap(6.0));

    let bottom_toggle = panel_toggle(Icon::PanelBottom, PanelWhich::Bottom, active, visible);
    let left_toggle = panel_toggle(Icon::PanelLeft, PanelWhich::Left, active, visible);
    let right_toggle = panel_toggle(Icon::PanelRight, PanelWhich::Right, active, visible);

    let cue_readout = label(move || {
        let cl = cuelist.get();
        let len = cl.len();
        let idx = active_cue.with(|a| {
            a.and_then(|id| cl.iter().position(|c| c.id == id))
                .map(|i| i + 1)
                .unwrap_or(0)
        });
        format!("CUE {}/{}", idx, len)
    })
    .style(|s| {
        s.font_family(theme().font.mono_sm.family.clone())
            .color(theme().color.text_secondary)
            .font_size(12.0)
    });

    let panic_btn = button(h_stack((
        icon(Icon::Ban, theme().color.status_error, 14.0),
        text("PANIC").style(|s| {
            s.color(theme().color.status_error)
                .font_weight(floem::text::Weight::BOLD)
                .font_size(12.0)
        }),
    )))
    .action(move || active_cue.set(None))
    .style(|s| {
        s.background(theme().color.bg_overlay)
            .border(1.0)
            .border_color(theme().color.status_error)
            .border_radius(4.0)
            .padding_horiz(12.0)
            .padding_vert(6.0)
            .gap(6.0)
            .hover(|s| s.background(theme().color.status_error.multiply_alpha(0.25)))
    });

    h_stack((
        pause_btn,
        go_btn,
        back_btn,
        search_wrap,
        left_toggle,
        bottom_toggle,
        right_toggle,
        text("|").style(|s| s.color(theme().color.text_disabled)),
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
            .background(theme().color.bg_surface)
            .border_bottom(1.0)
            .border_color(theme().color.border_subtle)
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
        theme().color.status_active
    } else {
        on
    }
}

/// A small chevron/panel icon that toggles an optional panel.
fn panel_toggle(
    icon: Icon,
    which: PanelWhich,
    active: RwSignal<PanelFlags>,
    visible: RwSignal<PanelFlags>,
) -> impl IntoView {
    let shown = move || {
        visible.with(|v| match which {
            PanelWhich::Left => v.left,
            PanelWhich::Right => v.right,
            PanelWhich::Bottom => v.bottom,
        })
    };
    let hide = move || {
        active.with(|v| match which {
            PanelWhich::Left => !v.left,
            PanelWhich::Right => !v.right,
            PanelWhich::Bottom => !v.bottom,
        })
    };
    let child = icon.into_view().style(move |s| {
        s.size(16.0, 16.0).color(if shown() {
            theme().color.status_active
        } else {
            theme().color.text_secondary
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
        .style(move |s| {
            s.background(theme().color.bg_overlay)
                .color(theme().color.text_primary)
                .border_radius(4.0)
                .padding_horiz(8.0)
                .padding_vert(6.0)
                .font_size(12.0)
                .hover(|s| s.background(theme().color.border_subtle))
                .apply_if(hide(), |s| s.display(Display::None))
        })
}

/// Shared base styling for the plain icon buttons.
fn button_style() -> impl Fn(floem::style::Style) -> floem::style::Style + 'static {
    move |s: floem::style::Style| {
        s.background(theme().color.bg_overlay)
            .color(theme().color.text_primary)
            .border_radius(4.0)
            .padding_horiz(10.0)
            .padding_vert(6.0)
            .font_size(14.0)
            .min_width(34.0)
            .hover(|s| s.background(theme().color.border_subtle))
    }
}
