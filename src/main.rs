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
mod tabbed;
mod theme;
mod toolbar;

use std::sync::Arc;

use crossbeam_channel;

use arc_swap::ArcSwap;
use floem::ext_event::create_signal_from_channel;
use floem::keyboard::Key;
use floem::reactive::{
    ReadSignal, RwSignal, SignalGet, SignalUpdate, SignalWith, create_effect, create_memo,
    create_rw_signal,
};
use floem::views::{Decorators, h_stack, text, v_stack};
use floem::window::WindowConfig;
use floem::{Application, IntoView};
use tokio::sync::mpsc::Sender;

use crate::exec::{ExecutionEngine, UiEvent};
use crate::model::{ExecutionState, Playhead, WorkspaceState};
use crate::panel::PanelSystem;
use crate::theme::*;

fn main() {
    // Temporarily use programmatic samples
    // TODO: remove after save state is implemented
    let workspace = Arc::new(ArcSwap::from_pointee(WorkspaceState::sample()));

    let (exec_engine, exec_state_rx, events_tx) = ExecutionEngine::init(workspace.clone());
    let (tokio_tx, tokio_rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build shared tokio runtime");

        rt.block_on(async move {
            let handle = tokio::runtime::Handle::current();
            let _ = tokio_tx.send(handle);

            exec_engine.run().await;
        });
    });

    let tokio_handle = tokio_rx.recv().expect("Tokio runtime failed to start");

    let (ui_exec_state_tx, ui_exec_state_rx) =
        crossbeam_channel::unbounded::<Arc<ExecutionState>>();

    let mut exec_state_r = exec_state_rx.clone();
    let initial_val = exec_state_r.borrow().clone();
    let _ = ui_exec_state_tx.send(initial_val);

    tokio_handle.spawn(async move {
        while exec_state_r.changed().await.is_ok() {
            let next = exec_state_r.borrow_and_update().clone();
            if ui_exec_state_tx.send(next).is_err() {
                break;
            }
        }
    });

    Application::new()
        .window(
            move |_| {
                let exec_state_signal_r =
                    create_signal_from_channel::<Arc<ExecutionState>>(ui_exec_state_rx);

                app_view(workspace, exec_state_signal_r, events_tx)
            },
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

/// Apply a mutation to the workspace on the UI thread.
///
/// The `RwSignal` is the single source of truth for the UI: writing it notifies
/// every subscriber (cuelist, detail, toolbar). The `ArcSwap` is the engine's
/// read path: `store` publishes the new `Arc<WorkspaceState>` so the next time
/// the Execution Engine reads it (e.g. on `Go`), it sees the latest edits —
/// without locking or copying the cue list.
///
/// This is the single entry point for any UI edit to the workspace. It is
/// currently unused because no view mutates the workspace yet; wire it into
/// editors (e.g. the detail inspector) as they gain write access.
#[allow(dead_code)]
fn update_workspace(
    signal: RwSignal<Arc<WorkspaceState>>,
    shared: &Arc<ArcSwap<WorkspaceState>>,
    f: impl FnOnce(&mut WorkspaceState),
) {
    // `WorkspaceState` is immutable behind `Arc`, so clone -> modify -> rewrap.
    let mut next = signal.get().as_ref().clone();
    f(&mut next);
    let next = Arc::new(next);
    signal.set(next.clone()); // (a) notify UI subscribers
    shared.store(next); // (b) arcswap for the exec engine
}

fn app_view(
    workspace: Arc<ArcSwap<WorkspaceState>>,
    exec_state: ReadSignal<Option<Arc<ExecutionState>>>,
    events_tx: Sender<UiEvent>,
) -> impl IntoView {
    // UI-side source of truth for the workspace. Seeded from the current
    // `ArcSwap` snapshot; kept in sync with edits via `update_workspace`.
    let workspace_signal: RwSignal<Arc<WorkspaceState>> = create_rw_signal(workspace.load_full());

    let cuelist_memo = create_memo(move |_| workspace_signal.with(|ws| ws.cuelist.clone()));

    let selected = create_rw_signal(None);
    let active_cue = create_rw_signal(None);
    let search = create_rw_signal(String::new());

    {
        let act = active_cue;
        let sel = selected;
        create_effect(move |_| {
            if let Some(state) = exec_state.get() {
                let p = match state.playhead {
                    Playhead::Stopped => None,
                    Playhead::Playing(id) => Some(id),
                };

                act.set(p);
                sel.set(p);
            }
        });
    }

    // Panel system owns layout + resize; we just hand it windows by location.
    let panels = PanelSystem::new();
    let active = panels.active();
    let visible = panels.visibility();

    let toolbar = toolbar::view(
        cuelist_memo,
        active_cue,
        selected,
        search,
        active,
        visible,
        events_tx,
    );
    let cuelist_view = cuelist::view(cuelist_memo, selected, active_cue, search);
    let media = media::view(visible);
    let detail = detail::view(selected, cuelist_memo);

    let root = panels
        .with_main(cuelist_view)
        .with_bottom(detail)
        .with_right(media)
        .build(toolbar, status_bar())
        .style(global_stylesheet)
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
