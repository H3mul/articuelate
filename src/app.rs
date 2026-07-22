//! Floem application setup and UI-side execution-state ingestion.

use std::future::Future;
use std::sync::Arc;

use arc_swap::ArcSwap;
use crossbeam_channel::Receiver;
use floem::ext_event::create_signal_from_channel;
use floem::keyboard::Key;
use floem::reactive::{
    ReadSignal, RwSignal, SignalGet, SignalUpdate, SignalWith, create_effect, create_memo,
    create_rw_signal, create_signal, provide_context,
};
use floem::views::{Decorators, dyn_container, h_stack, text, v_stack};
use floem::window::WindowConfig;
use floem::{Application, IntoView};
use tokio::sync::{mpsc::Sender, watch};

use crate::cuelist;
use crate::detail;
use crate::exec::UiEvent;
use crate::media;
use crate::model::{ExecutionState, Playhead, WorkspaceState};
use crate::panel::PanelSystem;

use crate::theme::{ThemeSignal, global_stylesheet, load_theme, theme};
use crate::toolbar;

/// The Floem application and its UI-side execution-state channel.
pub struct App {
    workspace: Arc<ArcSwap<WorkspaceState>>,
    exec_state_rx: Receiver<Arc<ExecutionState>>,
    events_tx: Sender<UiEvent>,
    theme_signal: ThemeSignal,
}

impl App {
    /// Set up the UI and prepare a future that forwards execution state into Floem.
    ///
    /// The returned future is intended to be spawned on the shared Tokio runtime
    /// via `handle.spawn(forwarder)`, keeping `main.rs` free of crossbeam/Floem details.
    pub fn init(
        workspace: Arc<ArcSwap<WorkspaceState>>,
        exec_state_rx: watch::Receiver<Arc<ExecutionState>>,
        events_tx: Sender<UiEvent>,
    ) -> (Self, impl Future<Output = ()> + Send + 'static, ThemeSignal) {
        let (ui_exec_state_tx, ui_exec_state_rx) = crossbeam_channel::unbounded();
        let mut exec_state_r = exec_state_rx;
        let initial_val = exec_state_r.borrow().clone();
        let _ = ui_exec_state_tx.send(initial_val);

        let state_forwarder = async move {
            while exec_state_r.changed().await.is_ok() {
                let next = exec_state_r.borrow_and_update().clone();
                if ui_exec_state_tx.send(next).is_err() {
                    break;
                }
            }
        };

        let theme_signal: ThemeSignal = create_signal(load_theme());

        (
            Self {
                workspace,
                exec_state_rx: ui_exec_state_rx,
                events_tx,
                theme_signal,
            },
            state_forwarder,
            theme_signal,
        )
    }

    pub fn run(self) {
        let Self {
            workspace,
            exec_state_rx,
            events_tx,
            theme_signal,
        } = self;

        Application::new()
            .window(
                move |_| {
                    let exec_state_signal_r =
                        create_signal_from_channel::<Arc<ExecutionState>>(exec_state_rx);

                    // Provide the theme signal as context so `theme()` works
                    // anywhere in the view tree.
                    provide_context(theme_signal);

                    // A counter that bumps on theme change, driving a full
                    // rebuild via dyn_container.
                    let theme_gen = create_rw_signal(0usize);
                    {
                        let (rx, _) = theme_signal;
                        create_effect(move |_| {
                            rx.get();
                            theme_gen.update(|n| *n = n.wrapping_add(1));
                        });
                    }

                    let ws = workspace.clone();
                    let tx = events_tx.clone();
                    dyn_container(
                        move || theme_gen.get(),
                        move |_| app_view(ws.clone(), exec_state_signal_r, tx.clone()),
                    )
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
}

/// Apply a mutation to the workspace on the UI thread.
///
/// The `RwSignal` is the single source of truth for the UI: writing it notifies
/// every subscriber (cuelist, detail, toolbar). The `ArcSwap` is the engine's
/// read path: `store` publishes the new `Arc<WorkspaceState>` so the next time
/// the Execution Engine reads it (e.g. on `Go`), it sees the latest edits —
/// without locking or copying the cue list.
#[allow(dead_code)]
fn update_workspace(
    signal: RwSignal<Arc<WorkspaceState>>,
    shared: &Arc<ArcSwap<WorkspaceState>>,
    f: impl FnOnce(&mut WorkspaceState),
) {
    let mut next = signal.get().as_ref().clone();
    f(&mut next);
    let next = Arc::new(next);
    signal.set(next.clone());
    shared.store(next);
}

fn app_view(
    workspace: Arc<ArcSwap<WorkspaceState>>,
    exec_state: ReadSignal<Option<Arc<ExecutionState>>>,
    events_tx: Sender<UiEvent>,
) -> impl IntoView {
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

    panels
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
        )
}

fn status_bar() -> impl IntoView {
    let left = text("STATUS: Connected (ASIO: Focusrite USB)").style(|s| {
        s.color(theme().color.status_active)
            .font_size(11.0)
            .font_family(theme().font.mono_sm.family.clone())
    });
    let right = text("CPU: 4%   DSP: 12%").style(|s| {
        s.color(theme().color.text_secondary)
            .font_size(11.0)
            .font_family(theme().font.mono_sm.family.clone())
    });
    let spacer = text("").style(|s| s.flex_grow(1.0));

    v_stack((h_stack((left, spacer, right)).style(|s| {
        s.items_center()
            .gap(10.0)
            .padding_horiz(12.0)
            .padding_vert(4.0)
            .background(theme().color.bg_surface)
            .border_top(1.0)
            .border_color(theme().color.border_subtle)
            .height(24.0)
    }),))
    .style(|s| s.width_full())
}
