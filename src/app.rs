//! Floem application setup and UI-side execution-state ingestion.

use std::future::Future;
use std::sync::Arc;

use arc_swap::ArcSwap;
use crossbeam_channel::Receiver;

use floem::ext_event::update_signal_from_channel;

use floem::reactive::{Context, Effect, Memo, ReadSignal, RwSignal, SignalGet, SignalUpdate};
use floem::views::{Decorators, Stack, dyn_container};
use floem::window::WindowConfig;
use floem::{Application, IntoView};

use tokio::sync::watch;
use tracing::{debug, info};

use crate::audio::AudioEngine;
use crate::devtools::DebugInspectorExt;
use crate::exec::ExecutionHandle;
use crate::model::{
    AppState, ExecutionState, PlaybackStatus, Playhead, TransientCueState, WorkspaceState,
};
use crate::style::{Theme, global_stylesheet, load_theme, theme};
use crate::ui::panel::PanelSystem;
use crate::ui::{cuelist, detail, media, status_bar, toolbar};

/// The Floem application and its UI-side execution-state channel.
pub struct App {
    app_state: AppState,
    exec_state_rx: Receiver<Arc<ExecutionState>>,
    execution: ExecutionHandle,
    audio_engine: Arc<AudioEngine>,
    theme_signal: RwSignal<Theme>,
    theme_rx: crossbeam_channel::Receiver<Theme>,
}

impl App {
    /// Set up the UI and prepare a future that forwards execution state into Floem.
    ///
    /// The returned future is intended to be spawned on the shared Tokio runtime
    /// via `handle.spawn(forwarder)`, keeping `main.rs` free of crossbeam/Floem details.
    ///
    /// `App::new` is responsible for assembling the unified `AppState` from the
    /// raw communication channels provided by `main.rs`. This keeps the UI domain
    /// self-contained.
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
        // ── Bridge execution state from tokio watch → crossbeam → Floem signal ──
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

        // ── Theme reload channel ──
        let (theme_tx, theme_rx) = crossbeam_channel::unbounded();
        let theme_signal = RwSignal::new(load_theme());

        // ── Assemble AppState from the channels ──
        let workspace_state = workspace.load_full();
        let telemetry = audio_engine.telemetry();
        let sample_exec = crate::model::sample_execution_state(&workspace_state.cuelist);
        let app_state = AppState {
            workspace: RwSignal::new(workspace_state.as_ref().clone()),
            execution: RwSignal::new(sample_exec),
            audio_telemetry: telemetry,
            selected_cue: RwSignal::new(None),
        };

        (
            Self {
                app_state,
                exec_state_rx: ui_exec_state_rx,
                execution,
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
            app_state,
            exec_state_rx,
            execution,
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
                move |_| -> _ {
                    let exec_state_signal_r = RwSignal::new(None::<Arc<ExecutionState>>);
                    update_signal_from_channel(exec_state_signal_r.write_only(), exec_state_rx);

                    // Provide the theme signal as context so `theme()` works
                    // anywhere in the view tree.
                    Context::provide(theme_signal);

                    // Bridge theme changes from the tokio thread into the Floem
                    // reactive system via a crossbeam channel, so that the
                    // RwSignal is always set on the Floem main thread.
                    let theme_from_channel = RwSignal::new(None::<Theme>);
                    update_signal_from_channel(theme_from_channel.write_only(), theme_rx);

                    // A counter that bumps on theme change, driving a full
                    // rebuild via dyn_container.
                    let theme_gen = RwSignal::new(0usize);
                    Effect::new(move |_| {
                        if let Some(theme) = theme_from_channel.get() {
                            theme_signal.set(theme);
                            theme_gen.update(|n| *n = n.wrapping_add(1));
                        }
                    });

                    let state = app_state.clone();
                    let exec = execution.clone();
                    dyn_container(
                        move || theme_gen.get(),
                        move |_| {
                            app_view(state.clone(), exec_state_signal_r.read_only(), exec.clone())
                        },
                    )
                    // Make the base view fill the window
                    .style(|s| s.size_full().min_size(0.0, 0.0))
                    .attach_inspector()
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

fn app_view(
    app_state: AppState,
    exec_state: ReadSignal<Option<Arc<ExecutionState>>>,
    execution: ExecutionHandle,
) -> impl IntoView {
    // ── Derive memoised signals from AppState ──
    let cuelist_memo = {
        let s = app_state.clone();
        Memo::new(move |_| s.workspace.get().cuelist.clone())
    };

    // Wire execution state from the watch channel into the AppState signal
    {
        let s = app_state.clone();
        Effect::new(move |_| {
            if let Some(state) = exec_state.get() {
                s.execution.set(state.as_ref().clone());
            }
        });
    }

    // ── Build a TransientCueState for the currently selected cue ──
    let selected_transient = {
        let s = app_state.clone();
        Memo::new(move |_| s.selected_cue.get().and_then(|id| s.cue_state(id)))
    };

    // ── Build TransientCueState for the active (playing) cue ──
    let active_transient = {
        let s = app_state.clone();
        Memo::new(move |_| {
            let playhead_id = match s.execution.get().playhead {
                Playhead::Stopped => None,
                Playhead::Playing(id) => Some(id),
            };
            playhead_id.and_then(|id| s.cue_state(id))
        })
    };

    // ── Build TransientCueState for the next cue (after playhead) ──
    let next_transient = {
        let s = app_state.clone();
        Memo::new(move |_| {
            let next_id = match s.execution.get().playhead {
                Playhead::Stopped => None,
                Playhead::Playing(current_id) => {
                    let cuelist = s.workspace.get().cuelist;
                    cuelist
                        .iter_after(current_id)
                        .and_then(|mut it| it.next().map(|cue| cue.id))
                }
            };
            next_id.and_then(|id| s.cue_state(id))
        })
    };

    // ── Running cues for the media sidebar ──
    let running_transients = {
        let s = app_state.clone();
        Memo::new(move |_| {
            let exec = s.execution.get();
            let mut running: Vec<TransientCueState> = Vec::new();
            for (id, ces) in exec.cue_execution_state.iter() {
                if ces.status == PlaybackStatus::Playing || ces.status == PlaybackStatus::Standby {
                    if let Some(tcs) = s.cue_state(*id) {
                        running.push(tcs);
                    }
                }
            }
            running
        })
    };

    let cue_count = cuelist_memo.get().len();
    let selected_count_rw = RwSignal::new(0usize);

    // Track selected count
    {
        let s = app_state.clone();
        Effect::new(move |_| {
            if s.selected_cue.get().is_some() {
                selected_count_rw.set(1);
            } else {
                selected_count_rw.set(0);
            }
        });
    }

    let main_view = Stack::vertical((
        toolbar::view(execution, active_transient, next_transient),
        cuelist::view(cuelist_memo, app_state.clone()),
    ))
    .style(|s| s.width_full().height_full().min_width(0.0));
    let sidebar_view = media::view(running_transients);
    let detail_view = detail::view(selected_transient);

    let panel_system = PanelSystem::new();
    let panel = panel_system
        .builder()
        .with_main(main_view)
        .with_right(sidebar_view, Some(theme().dim.sidebar_width as f32))
        .with_bottom(detail_view, Some(theme().dim.detail_height as f32))
        .build();

    let status_bar_view =
        status_bar::view(selected_count_rw.get_untracked(), cue_count, panel_system);

    Stack::vertical((panel, status_bar_view))
        .style(|s| {
            s.flex_col()
                .size_full()
                .gap(1.0)
                .background(theme().color.bg_app)
        })
        .style(global_stylesheet)
}
