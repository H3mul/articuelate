//! Top transport toolbar: Pause, GO, Back, global search, and Panic.
//!
//! Flat, borderless icon-style buttons in the Lapce tradition. GO is the
//! prominent theatre-green action; PANIC is red. Toggling the side panels lives
//! here too (small chevrons), mirroring Lapce's panel toggles.

use floem::reactive::{create_rw_signal, RwSignal, SignalGet, SignalUpdate};
use floem::views::{button, h_stack, label, text, text_input, Decorators};
use floem::IntoView;

use crate::theme::*;

#[allow(clippy::too_many_arguments)]
pub fn view(
    cues_len: usize,
    active_cue: RwSignal<usize>,
    selected: RwSignal<usize>,
    search: RwSignal<String>,
    show_bottom: RwSignal<bool>,
    show_right: RwSignal<bool>,
) -> impl IntoView {
    let paused = create_rw_signal(false);

    let pause_btn = button("‖")
        .action(move || paused.update(|p| *p = !*p))
        .style(move |s| {
            s.background(if paused.get() {
                ACCENT_DIM.multiply_alpha(0.35)
            } else {
                PANEL_ALT
            })
            .color(TEXT)
            .border_radius(4.0)
            .padding_horiz(10.0)
            .padding_vert(6.0)
            .font_size(14.0)
            .min_width(34.0)
            .hover(|s| s.background(BORDER))
        });

    let go_btn = button("▶  GO")
        .action(move || {
            let next = (active_cue.get() + 1).min(cues_len.saturating_sub(1));
            active_cue.set(next);
            selected.set(next);
        })
        .style(|s| {
            s.background(ACCENT)
                .color(BG)
                .font_weight(floem::text::Weight::BOLD)
                .padding_horiz(18.0)
                .padding_vert(6.0)
                .font_size(14.0)
                .border_radius(4.0)
                .hover(|s| s.background(ACCENT_DIM))
        });

    let back_btn = button("◀")
        .action(move || {
            let prev = active_cue.get().saturating_sub(1);
            active_cue.set(prev);
            selected.set(prev);
        })
        .style(|s| {
            s.background(PANEL_ALT)
                .color(TEXT)
                .border_radius(4.0)
                .padding_horiz(10.0)
                .padding_vert(6.0)
                .font_size(14.0)
                .min_width(34.0)
                .hover(|s| s.background(BORDER))
        });

    let search_box = text_input(search)
        .placeholder("Search cues…")
        .style(|s| {
            s.background(PANEL_ALT)
                .color(TEXT)
                .border(1.0)
                .border_color(BORDER)
                .border_radius(4.0)
                .padding_horiz(8.0)
                .padding_vert(4.0)
                .width(220.0)
                .font_size(12.0)
        })
        .keyboard_navigable();

    let panic_btn = button("⏻  PANIC")
        .action(move || active_cue.set(0))
        .style(|s| {
            s.background(PANEL_ALT)
                .color(PANIC)
                .border(1.0)
                .border_color(PANIC_DIM)
                .border_radius(4.0)
                .padding_horiz(12.0)
                .padding_vert(6.0)
                .font_size(12.0)
                .font_weight(floem::text::Weight::BOLD)
                .hover(|s| s.background(PANIC_DIM.multiply_alpha(0.25)))
        });

    let bottom_toggle = panel_toggle("▾", show_bottom);
    let right_toggle = panel_toggle("▸", show_right);

    let cue_readout = label(move || {
        format!(
            "CUE {}/{}",
            active_cue.get().saturating_add(1).min(cues_len),
            cues_len
        )
    })
    .style(|s| s.font_family(MONO.to_string()).color(TEXT_DIM).font_size(12.0));

    h_stack((
        pause_btn,
        go_btn,
        back_btn,
        search_box,
        bottom_toggle,
        right_toggle,
        text("|").style(|s| s.color(TEXT_FAINT)),
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
            .background(PANEL)
            .border_bottom(1.0)
            .border_color(BORDER)
            .height(44.0)
    })
}

/// A small chevron that toggles a collapsible panel.
fn panel_toggle(glyph: &'static str, show: RwSignal<bool>) -> impl IntoView {
    button(glyph)
        .action(move || show.update(|v| *v = !*v))
        .style(move |s| {
            s.background(PANEL_ALT)
                .color(if show.get() { ACCENT } else { TEXT_DIM })
                .border_radius(4.0)
                .padding_horiz(8.0)
                .padding_vert(6.0)
                .font_size(12.0)
                .hover(|s| s.background(BORDER))
        })
}
