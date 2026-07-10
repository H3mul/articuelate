//! Articuelate - a native, Floem-based audio cue system.
//!
//! This is a placeholder skeleton UI. It assembles a Lapce-inspired, three-pane
//! workspace from docs/ui.md using a decoupled, resizable panel system
//! (see `panel.rs`): a top transport toolbar, a virtualized centre cuelist,
//! a collapsible bottom inspector, a live right-hand media monitor, a left
//! navigation sidebar, and a footer status bar.

mod cuelist;
mod detail;
mod media;
mod model;
mod panel;
mod theme;
mod toolbar;

use floem::keyboard::Key;
use floem::reactive::{RwSignal, SignalUpdate, create_rw_signal};
use floem::views::{Decorators, button, h_stack, list, text, v_stack};
use floem::window::WindowConfig;
use floem::{Application, IntoView};

use crate::model::{Cue, sample_cues};
use crate::panel::PanelSystem;
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
    let cues: RwSignal<im::Vector<Cue>> = create_rw_signal(sample_cues().into_iter().collect());
    let selected = create_rw_signal(0usize);
    let active_cue = create_rw_signal(0usize);
    let search = create_rw_signal(String::new());

    // Panel system owns layout + resize; we just hand it windows by location.
    let panels = PanelSystem::new();
    let visible = panels.visibility();

    let toolbar = toolbar::view(cues_len, active_cue, selected, search, visible);
    let cuelist = cuelist::view(cues, selected, active_cue, search);
    let media = media::view(visible);
    let detail = detail::view(selected, cues);

    let root = panels
        .with_main(cuelist)
        .with_bottom(detail)
        .with_right(media)
        .with_left(left_sidebar())
        .build(toolbar, status_bar())
        .into_view()
        .keyboard_navigable()
        .on_key_down(
            Key::Character("j".into()),
            |m| m.control(),
            move |_| visible.update(|v| v.bottom = !v.bottom),
        );

    root
}

/// Minimal left "Groups" navigation sidebar (placeholder window content).
fn left_sidebar() -> impl IntoView {
    let header = text("GROUPS").style(|s| {
        s.font_family(theme().font.mono.to_string())
            .font_size(11.0)
            .color(theme().color.text_dim)
            .padding_horiz(12.0)
            .padding_vert(8.0)
            .width_full()
    });

    let items = ["Cues", "Media", "Settings"];
    let nav = list(items.into_iter().map(|name| {
        button(text(name).style(|s| s.color(theme().color.fg).font_size(12.0))).style(|s| {
            s.width_full()
                .padding_horiz(12.0)
                .padding_vert(8.0)
                .background(theme().color.panel)
                .color(theme().color.fg)
                .border_radius(4.0)
                .hover(|s| s.background(theme().color.panel_alt))
        })
    }))
    .style(|s| s.flex_col().gap(2.0).padding(8.0));

    v_stack((header, nav)).style(|s| s.flex_col().width_full().background(theme().color.panel))
}

fn status_bar() -> impl IntoView {
    let left = text("STATUS: Connected (ASIO: Focusrite USB)").style(|s| {
        s.color(theme().color.accent)
            .font_size(11.0)
            .font_family(theme().font.mono.to_string())
    });
    let right = text("CPU: 4%   DSP: 12%").style(|s| {
        s.color(theme().color.text_dim)
            .font_size(11.0)
            .font_family(theme().font.mono.to_string())
    });

    let spacer = text("").style(|s| s.flex_grow(1.0));

    v_stack((h_stack((left, spacer, right)).style(|s| {
        s.items_center()
            .gap(10.0)
            .padding_horiz(12.0)
            .padding_vert(4.0)
            .background(theme().color.panel)
            .border_top(1.0)
            .border_color(theme().color.border)
            .height(24.0)
    }),))
    .style(|s| s.width_full())
}
