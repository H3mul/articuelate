//! Articuelate - a native, Floem-based audio cue system.
//!
//! This is a placeholder skeleton UI. It assembles a Lapce-inspired, three-pane
//! workspace from docs/ui.md using a decoupled, resizable panel system
//! (see `panel.rs`): a top transport toolbar, a virtualized centre cuelist,
//! a collapsible bottom inspector, a live right-hand media monitor, a left
//! navigation sidebar, and a footer status bar.

mod cuelist;
mod detail;
mod exec;
mod media;
mod model;
mod panel;
mod theme;
mod toolbar;

use std::sync::Arc;
use std::time::Duration;

use floem::action::exec_after;
use floem::keyboard::Key;
use floem::prelude::SignalTrack;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate, create_effect, create_rw_signal};
use floem::views::{Decorators, h_stack, text, v_stack};
use floem::window::WindowConfig;
use floem::{Application, IntoView};

use crate::exec::ExecutorHandle;
use crate::model::{Cue, ExecutionState, WorkspaceState, sample_cues};
use crate::panel::PanelSystem;
use crate::theme::*;

fn main() {
    // The UI-owned, read-only show snapshot. Shared (Arc) with the Execution
    // Thread so it can query cue data lock-free on every event.
    let workspace = Arc::new(WorkspaceState::new(sample_cues()));

    // Boot the Execution Thread; keep the UI-side handles.
    let executor = exec::spawn(workspace.clone());
    let cues_len = workspace.cues.len();

    Application::new()
        .window(
            move |_| app_view(workspace, executor, cues_len),
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

fn app_view(workspace: Arc<WorkspaceState>, executor: ExecutorHandle, cues_len: usize) -> impl IntoView {
    // Mirror the workspace cues into the UI's reactive list (kept as im::Vector
    // to preserve the existing cuelist/detail bindings).
    let cues: RwSignal<im::Vector<Cue>> =
        create_rw_signal(workspace.cues.iter().cloned().collect());

    let selected = create_rw_signal(0usize);
    let active_cue = create_rw_signal(0usize);
    let search = create_rw_signal(String::new());

    // --- Execution state ingestion -------------------------------------
    // The Exec Thread publishes via a `watch` channel (it must not touch Floem
    // signals directly, since those live in the UI thread). We mirror the
    // latest value into a Floem `RwSignal` on a light UI-thread poll, which the
    // rest of the reactive UI then subscribes to.
    let exec_state = create_rw_signal(ExecutionState::default());
    let ingest_tick = create_rw_signal(());
    {
        let rx = executor.state.clone();
        let set = exec_state;
        let tick = ingest_tick;
        create_effect(move |_| {
            tick.track();
            let rx = rx.clone();
            exec_after(Duration::from_millis(50), move |_| {
                set.set((*rx.borrow()).clone());
                tick.set(());
            });
        });
    }

    // Mirror the Exec playhead into the UI's active/selected cues. This only
    // re-runs when `exec_state` actually changes, so local cursor moves
    // (back / panic) remain until the next Execution Thread update.
    {
        let act = active_cue;
        let sel = selected;
        create_effect(move |_| {
            let p = exec_state.get().playhead;
            act.set(p);
            sel.set(p);
        });
    }

    // Panel system owns layout + resize; we just hand it windows by location.
    let panels = PanelSystem::new();
    let visible = panels.visibility();

    let toolbar = toolbar::view(cues_len, active_cue, selected, search, visible, executor.events);
    let cuelist = cuelist::view(cues, selected, active_cue, search);
    let media = media::view(visible);
    let detail = detail::view(selected, cues);

    let root = panels
        .with_main(cuelist)
        .with_bottom(detail)
        .with_right(media)
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
