//! Floem application setup and UI-side execution-state ingestion.

use std::future::Future;
use std::sync::Arc;

use arc_swap::ArcSwap;
use crossbeam_channel::Receiver;
use floem::ext_event::create_signal_from_channel;
use floem::reactive::{
    ReadSignal, RwSignal, SignalGet, SignalUpdate, SignalWith, create_effect, create_memo,
    create_rw_signal, provide_context,
};
use floem::views::{Decorators, dyn_container, v_stack};
use floem::window::WindowConfig;
use floem::{Application, IntoView};
use ringbuf::HeapCons;
use tokio::sync::{mpsc::Sender, watch};

use crate::audio::AudioTelemetry;
use crate::exec::UiEvent;
use crate::model::{ExecutionState, Playhead, WorkspaceState};
use crate::style::{Theme, global_stylesheet, load_theme, theme};
use crate::ui::{cuelist, panel::PanelSystem, status_bar, toolbar};
use crate::ui::{detail, media};

/// The Floem application and its UI-side execution-state channel.
pub struct App {
    workspace: Arc<ArcSwap<WorkspaceState>>,
    exec_state_rx: Receiver<Arc<ExecutionState>>,
    events_tx: Sender<UiEvent>,
    telemetry: Option<HeapCons<AudioTelemetry>>,
    theme_signal: RwSignal<Theme>,
    theme_rx: crossbeam_channel::Receiver<Theme>,
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
        telemetry: HeapCons<AudioTelemetry>,
    ) -> (
        Self,
        impl Future<Output = ()> + Send + 'static,
        crossbeam_channel::Sender<Theme>,
    ) {
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

        let (theme_tx, theme_rx) = crossbeam_channel::unbounded();
        let theme_signal = create_rw_signal(load_theme());

        (
            Self {
                workspace,
                exec_state_rx: ui_exec_state_rx,
                events_tx,
                telemetry: Some(telemetry),
                theme_signal,
                theme_rx,
            },
            state_forwarder,
            theme_tx,
        )
    }

    pub fn run(self) {
        let Self {
            workspace,
            exec_state_rx,
            events_tx,
            telemetry,
            theme_signal,
            theme_rx,
        } = self;

        Application::new()
            .window(
                move |_| {
                    let exec_state_signal_r =
                        create_signal_from_channel::<Arc<ExecutionState>>(exec_state_rx);

                    // Provide the theme signal as context so `theme()` works
                    // anywhere in the view tree.
                    provide_context(theme_signal);

                    // Bridge theme changes from the tokio thread into the Floem
                    // reactive system via a crossbeam channel, so that the
                    // RwSignal is always set on the Floem main thread.
                    let theme_from_channel = create_signal_from_channel::<Theme>(theme_rx);

                    // A counter that bumps on theme change, driving a full
                    // rebuild via dyn_container.
                    let theme_gen = create_rw_signal(0usize);
                    create_effect(move |_| {
                        if let Some(theme) = theme_from_channel.get() {
                            theme_signal.set(theme);
                            theme_gen.update(|n| *n = n.wrapping_add(1));
                        }
                    });

                    let _media = telemetry.map(media::view);
                    let ws = workspace.clone();
                    let tx = events_tx.clone();
                    dyn_container(
                        move || theme_gen.get(),
                        move |_| app_view(ws.clone(), exec_state_signal_r, tx.clone()),
                    )
                    // Make the base view fill the window
                    .style(|s| s.size_full().min_size(0.0, 0.0))
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

    let cuelist_view = cuelist::view(cuelist_memo, selected, active_cue);

    let _detail = detail::view(selected, cuelist_memo);

    let panel_system = PanelSystem::new();

    let toolbar = toolbar::view(cuelist_memo, active_cue, selected, events_tx);

    let main_view = floem::views::v_stack((toolbar, cuelist_view))
        .style(|s| s.width_full().height_full().min_size(0.0, 0.0));

    let panels = panel_system
        .builder()
        .with_main(main_view)
        // .with_bottom(detail)
        // .with_right(media)
        .build()
        .into_view();

    let status_bar = status_bar::view(panel_system);

    v_stack((panels, status_bar.into_any()))
        .style(|s| {
            s.flex_col()
                .width_full()
                .height_full()
                .background(theme().color.bg_app)
        })
        .style(global_stylesheet)
}
