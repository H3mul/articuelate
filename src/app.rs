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
use floem::views::{Decorators, dyn_container, h_stack, v_stack};
use floem::window::WindowConfig;
use floem::{Application, IntoView};

use tokio::sync::watch;
use tracing::{debug, info};

use crate::audio::{AudioEngine, AudioTelemetry};
use crate::exec::ExecutionHandle;
use crate::model::{ExecutionState, Playhead, WorkspaceState};
use crate::style::{Theme, global_stylesheet, load_theme, theme};
use crate::ui::{cuelist, detail, media, status_bar, toolbar};

/// The Floem application and its UI-side execution-state channel.
pub struct App {
    workspace: Arc<ArcSwap<WorkspaceState>>,
    exec_state_rx: Receiver<Arc<ExecutionState>>,
    execution: ExecutionHandle,
    telemetry: Option<Arc<AudioTelemetry>>,
    audio_engine: Arc<AudioEngine>,
    theme_signal: RwSignal<Theme>,
    theme_rx: crossbeam_channel::Receiver<Theme>,
}

impl App {
    /// Set up the UI and prepare a future that forwards execution state into Floem.
    ///
    /// The returned future is intended to be spawned on the shared Tokio runtime
    /// via `handle.spawn(forwarder)`, keeping `main.rs` free of crossbeam/Floem details.
    pub fn new(
        workspace: Arc<ArcSwap<WorkspaceState>>,
        exec_state_rx: watch::Receiver<Arc<ExecutionState>>,
        execution: ExecutionHandle,
        audio_engine: Arc<AudioEngine>,
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
            info!("Execution state forwarder started");
            while exec_state_r.changed().await.is_ok() {
                let next = exec_state_r.borrow_and_update().clone();
                debug!(?next.playhead, "Forwarding execution state to UI");
                if ui_exec_state_tx.send(next).is_err() {
                    break;
                }
            }
            info!("Execution state forwarder stopped");
        };

        let (theme_tx, theme_rx) = crossbeam_channel::unbounded();
        let theme_signal = create_rw_signal(load_theme());

        (
            Self {
                workspace,
                exec_state_rx: ui_exec_state_rx,
                execution,
                telemetry: Some(audio_engine.telemetry()),
                audio_engine,
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
            execution,
            telemetry: _,
            audio_engine,
            theme_signal,
            theme_rx,
        } = self;

        let devices = audio_engine.output_devices();
        if let Some(device) = devices.first().cloned() {
            info!(device = %device, "Setting initial audio device");
            let _ = execution.send_user_intent(crate::exec::UiEvent::SetAudioDevice(device));
        }

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

                    let ws = workspace.clone();
                    let execution = execution.clone();
                    dyn_container(
                        move || theme_gen.get(),
                        move |_| {
                            app_view(
                                ws.clone(),
                                exec_state_signal_r,
                                execution.clone(),
                            )
                        },
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
    execution: ExecutionHandle,
) -> impl IntoView {
    let workspace_signal: RwSignal<Arc<WorkspaceState>> = create_rw_signal(workspace.load_full());
    let cuelist_memo = create_memo(move |_| workspace_signal.with(|ws| ws.cuelist.clone()));

    let selected = create_rw_signal(None);
    let active_cue = create_rw_signal(None);

    // Wire execution state to active/selected signals
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
    let toolbar_view = toolbar::view(execution);
    let detail_view = detail::view(selected, cuelist_memo);
    let sidebar_view = media::view();
    let cue_count = cuelist_memo.get().len();
    let selected_count_rw = create_rw_signal(0usize);

    // Track selected count
    create_effect(move |_| {
        if selected.get().is_some() {
            selected_count_rw.set(1);
        } else {
            selected_count_rw.set(0);
        }
    });

    let status_bar_view = status_bar::view(
        selected_count_rw.get_untracked(),
        cue_count,
    );

    // Left column: toolbar + cuelist
    let left_column = v_stack((toolbar_view, cuelist_view))
        .style(|s| s.flex_col().min_width(0.0).flex_grow(1.0).height_full());

    // Main workspace: left column + sidebar (1px gutter between)
    let main_workspace = v_stack((
        h_stack((
            left_column,
            sidebar_view,
        ))
        .style(|s| s.flex_row().flex_grow(1.0).min_height(0.0).height_full().width_full().gap(1.0)),
        detail_view,
    ))
    .style(|s| s.flex_col().flex_grow(1.0).min_height(0.0).height_full().width_full().gap(1.0));

    v_stack((main_workspace, status_bar_view))
        .style(|s| {
            s.flex_col()
                .width_full()
                .height_full()
                .gap(1.0)
                .background(theme().color.bg_app)
        })
        .style(global_stylesheet)
}