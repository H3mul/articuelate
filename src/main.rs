//! Articuelate - a native, Floem-based audio cue system.
//!
//! This is a placeholder skeleton UI. It assembles the three-pane, Lapce-
//! inspired workspace from docs/ui.md: a top transport toolbar, a virtualized
//! center cuelist, a collapsible bottom inspector, a live right-hand media
//! monitor, and a footer status bar.

mod cuelist;
mod detail;
mod media;
mod model;
mod theme;
mod toolbar;

use floem::keyboard::Key;
use floem::reactive::{create_rw_signal, RwSignal, SignalGet, SignalUpdate};
use floem::views::{dyn_container, empty, h_stack, text, v_stack, Decorators};
use floem::window::WindowConfig;
use floem::{Application, IntoView};

use crate::model::{sample_cues, Cue};
use crate::theme::*;

fn main() {
    let cues_len = sample_cues().len();
    Application::new()
        .window(
            move |_| app_view(cues_len),
            Some(
                WindowConfig::default()
                    .size((1280.0, 800.0))
                    .title("Articuelate")
                    .show_titlebar(true)
                    .resizable(true),
            ),
        )
        .run();
}

fn app_view(cues_len: usize) -> impl IntoView {
    let cues: RwSignal<im::Vector<Cue>> =
        create_rw_signal(sample_cues().into_iter().collect());
    let selected = create_rw_signal(0usize);
    let active_cue = create_rw_signal(0usize);
    let search = create_rw_signal(String::new());
    let show_bottom = create_rw_signal(true);
    let show_right = create_rw_signal(true);

    let toolbar = toolbar::view(cues_len, active_cue, selected, search, show_bottom, show_right);
    let cuelist = cuelist::view(cues, selected, active_cue, search);
    let media = media::view(show_right);

    // Bottom inspector - collapsible via Ctrl+J or the toolbar chevron.
    let detail_panel = dyn_container(
        move || show_bottom.get(),
        move |show| {
            if show {
                detail::view(selected, cues).into_any()
            } else {
                empty().into_any()
            }
        },
    );

    let left_col = v_stack((cuelist, detail_panel))
        .style(|s| s.flex_col().flex_grow(1.0).min_width(0.0).min_height(0.0));

    let middle = h_stack((left_col, media))
        .style(|s| s.flex_row().flex_grow(1.0).min_height(0.0));

    let root = v_stack((toolbar, middle, status_bar()))
        .style(|s| s.flex_col().width_full().height_full().background(BG))
        .keyboard_navigable()
        .on_key_down(
            Key::Character("j".into()),
            |m| m.control(),
            move |_| show_bottom.update(|v| *v = !*v),
        );

    root
}

fn status_bar() -> impl IntoView {
    let left = text("STATUS: Connected (ASIO: Focusrite USB)")
        .style(|s| s.color(ACCENT).font_size(11.0).font_family(MONO.to_string()));
    let right = text("CPU: 4%   DSP: 12%")
        .style(|s| s.color(TEXT_DIM).font_size(11.0).font_family(MONO.to_string()));

    h_stack((left, text("").style(|s| s.flex_grow(1.0)), right))
        .style(|s| {
            s.items_center()
                .gap(10.0)
                .padding_horiz(12.0)
                .padding_vert(4.0)
                .background(PANEL)
                .border_top(1.0)
                .border_color(BORDER)
                .height(24.0)
        })
}
