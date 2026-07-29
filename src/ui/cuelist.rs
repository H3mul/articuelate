//! Cuelist — the main cue table.
//!
//! Uses Floem's virtual_list for performance with large show files.

use floem::IntoView;
use floem::reactive::{Effect, RwSignal, SignalGet, SignalUpdate, SignalWith};
use floem::style::AlignItems;
use floem::views::scroll::Scroll;
use floem::views::{Container, Decorators, Empty, Label, Stack, virtual_list};

use std::sync::Arc;

use crate::model::{AppState, CueColor, CueKind, Cuelist, PlaybackStatus, TransientCueState};
use crate::style::theme;
use crate::ui::icons::{AppIcon, app_icon};

fn fmt_duration(d: std::time::Duration) -> String {
    let total_secs = d.as_secs();
    let m = total_secs / 60;
    let s = total_secs % 60;
    format!("{:02}:{:02}", m, s)
}

fn strip_type(target: &str) -> String {
    if let Some(idx) = target.find(" · ") {
        target[idx + 3..].to_string()
    } else {
        target.to_string()
    }
}

fn kind_icon(kind: &CueKind) -> AppIcon {
    match kind {
        CueKind::Group => AppIcon::Folder,
        CueKind::Audio { .. } => AppIcon::Music,
        CueKind::Control { .. } => AppIcon::ListVideo,
        CueKind::Osc { .. } => AppIcon::Network,
        CueKind::Fade { .. } => AppIcon::Spline,
    }
}

fn time_cell(value: String, fill: Option<f64>, running: bool) -> impl IntoView {
    let fill_pct = fill.unwrap_or(0.0).min(1.0).max(0.0);
    let has_fill = fill.is_some() && fill_pct > 0.0;

    let label = Label::new(value).style(move |s| {
        s.font_family(theme().font.mono_sm.family.clone())
            .font_size(if running {
                theme().font.body.size
            } else {
                theme().font.mono_sm.size
            })
            .color(if running {
                theme().color.text_primary
            } else {
                theme().color.text_disabled
            })
            .min_width(0.0)
            .padding_horiz(6.0)
    });
    Container::new(label).style(move |s| {
        let mut s = s
            .height(theme().dim.time_cell)
            .min_width(0.0)
            .items_center()
            .justify_end()
            .border_radius(theme().dim.radius_sm);

        if has_fill {
            s = s.background(theme().color.status_running_bg_30);
        }
        if running {
            s = s.background(theme().color.status_running_bg_30);
        }
        s
    })
}

fn cue_row(position: usize, tcs: TransientCueState, app_state: AppState) -> impl IntoView {
    let id = tcs.id;
    let cue = tcs.workspace.get();
    let exec = tcs.execution.get();

    let is_running = exec.status == PlaybackStatus::Playing;
    let is_standby = exec.status == PlaybackStatus::Standby;
    let is_group = matches!(cue.kind, CueKind::Group);
    let stripe_color = match cue.color {
        CueColor::None => None,
        CueColor::Red => Some(theme().color.status_error),
        CueColor::Orange => Some(theme().color.status_group),
        CueColor::Green => Some(theme().color.status_running),
        CueColor::Blue => Some(theme().color.status_playhead),
        CueColor::Purple => Some(theme().color.status_standby),
    };

    let name = cue.name.clone();
    let number = app_state
        .workspace
        .get()
        .cuelist
        .position_of(id)
        .map(|n| n.to_string())
        .unwrap_or_else(|| "?".to_string());
    let target = strip_type(&cue.kind.target_label());
    let kind = cue.kind.clone();
    let pre_wait = fmt_duration(cue.pre_wait);
    let post_wait = fmt_duration(cue.post_wait);

    // Duration derived from audio telemetry when running
    let duration_label = {
        let metrics = tcs.read_audio_telemetry();
        let dur = metrics.total_duration_sec();
        if dur > 0.0 {
            fmt_duration(std::time::Duration::from_secs_f64(dur))
        } else {
            "00:00".to_string()
        }
    };

    // Progress fill from telemetry
    let progress_fill = {
        let metrics = tcs.read_audio_telemetry();
        let dur = metrics.total_duration_sec();
        if dur > 0.0 {
            Some(metrics.current_time_sec() / dur)
        } else {
            None
        }
    };

    // Pre-wait progress from execution state
    let pre_progress = if cue.pre_wait > std::time::Duration::ZERO {
        Some(exec.pre_wait_elapsed.as_secs_f64() / cue.pre_wait.as_secs_f64())
    } else {
        None
    };

    let row = Stack::horizontal((
        // Drag handle
        app_icon(
            AppIcon::GripVertical,
            theme().dim.icon_sm as f32,
            theme().color.text_disabled,
        )
        .style(|s| {
            s.width(theme().dim.col_drag)
                .items_center()
                .justify_center()
        }),
        // Playhead indicator
        Container::new(if is_standby || is_running {
            app_icon(
                AppIcon::Play,
                theme().dim.icon_sm as f32,
                if is_standby {
                    theme().color.status_standby
                } else {
                    theme().color.status_running
                },
            )
            .into_any()
        } else {
            Empty::new().into_any()
        })
        .style(|s| {
            s.width(theme().dim.col_playhead)
                .items_center()
                .justify_center()
        }),
        // Cue number (derived from position in cuelist)
        Label::derived(move || number.clone()).style(move |s| {
            s.width(theme().dim.col_cue_number)
                .font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
                .color(if is_running {
                    theme().color.status_running
                } else {
                    theme().color.text_secondary
                })
                .min_width(0.0)
        }),
        // Name + icon
        Stack::horizontal((
            app_icon(
                kind_icon(&kind),
                theme().dim.icon_sm as f32,
                theme().color.text_disabled,
            ),
            Label::derived(move || name.clone()).style(move |s| {
                s.font_family(theme().font.body.family.clone())
                    .font_size(theme().font.body.size)
                    .font_weight(floem::text::FontWeight::MEDIUM)
                    .color(theme().color.text_primary)
                    .min_width(0.0)
            }),
        ))
        .style(move |s| {
            s.min_width(0.0)
                .flex_grow(1.0)
                .items_center()
                .gap(theme().dim.space_sm)
        }),
        // Target
        Label::derived(move || target.clone()).style(|s| {
            s.font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
                .color(theme().color.text_disabled)
                .min_width(0.0)
                .flex_grow(1.0)
        }),
        // Pre-wait
        time_cell(pre_wait, pre_progress, false),
        // Duration
        time_cell(duration_label, progress_fill, is_running),
        // Post-wait
        time_cell(post_wait, None, false),
        // Menu button
        app_icon(
            AppIcon::EllipsisVertical,
            theme().dim.icon_sm as f32,
            theme().color.text_secondary,
        )
        .style(|s| {
            s.width(theme().dim.col_menu)
                .items_center()
                .justify_center()
        }),
    ))
    .style(move |s| {
        s.items_center()
            .width_full()
            .min_width(0.0)
            .height(theme().dim.height_cue_row)
            .gap(4.0)
            .padding_horiz(theme().dim.space_sm)
            .padding_left(theme().dim.space_xs)
    });

    let row_id_clone = id;
    let selected = app_state.selected_cue;
    Container::new(row)
        .style(move |s| {
            let mut s = s
                .width_full()
                .min_width(0.0)
                .height(theme().dim.height_cue_row)
                .border_bottom(1.0)
                .border_color(theme().color.border_row_divider);

            if is_running {
                s = s.background(theme().color.status_running_bg_20);
            } else if selected.get() == Some(row_id_clone) {
                s = s.background(theme().color.bg_selection);
            } else if position % 2 == 1 {
                s = s.background(theme().color.bg_surface_raised);
            } else {
                s = s.background(theme().color.bg_surface);
            }
            if is_group {
                s = s.border_left(3.0).border_color(theme().color.status_group);
            }
            if is_standby {
                s = s
                    .border_left(3.0)
                    .border_color(theme().color.status_standby);
            }
            if let Some(c) = stripe_color {
                s = s.border_left(3.0).border_color(c);
            }
            s
        })
        .on_event_stop(floem::event::listener::Click, move |_cx, _| {
            selected.set(Some(row_id_clone));
        })
        .into_any()
}

pub fn view(
    cuelist: impl SignalGet<Arc<Cuelist>> + SignalWith<Arc<Cuelist>> + Copy + 'static,
    app_state: AppState,
) -> impl IntoView {
    let filtered = RwSignal::new(Vec::<(usize, TransientCueState)>::new());
    {
        let filtered = filtered;
        let s = app_state.clone();
        Effect::new(move |_| {
            let items = cuelist.with(|list| {
                list.iter()
                    .enumerate()
                    .filter_map(|(i, cue)| s.cue_state(cue.id).map(|tcs| (i + 1, tcs)))
                    .collect::<Vec<(usize, TransientCueState)>>()
            });
            filtered.set(items);
        });
    }

    let rows = virtual_list(
        move || filtered,
        |(_, tcs)| tcs.id,
        move |_index, (pos, tcs)| cue_row(pos, tcs, app_state.clone()),
    )
    .style(|s| {
        s.width_full()
            .flex_col()
            .min_width(0.0)
            .min_height(0.0)
            .align_items(AlignItems::Stretch)
    });

    let rows = Scroll::new(rows).style(|s| {
        s.width_full()
            .flex_col()
            .min_size(0.0, 0.0)
            .align_items(AlignItems::Stretch)
            .flex_grow(1.0)
    });

    let header = Stack::horizontal((
        Label::new("").style(|s| s.width(theme().dim.col_drag)),
        Label::new("").style(|s| s.width(theme().dim.col_playhead)),
        Label::new("CUE").style(|s| {
            s.width(theme().dim.col_cue_number)
                .color(theme().color.text_secondary)
                .font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
        }),
        Label::new("NAME").style(|s| {
            s.flex_grow(1.0)
                .color(theme().color.text_secondary)
                .font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
                .min_width(0.0)
        }),
        Label::new("TARGET").style(|s| {
            s.flex_grow(1.0)
                .color(theme().color.text_secondary)
                .font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
                .min_width(0.0)
        }),
        Label::new("PRE").style(|s| {
            s.width(theme().dim.col_time)
                .color(theme().color.text_secondary)
                .font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
        }),
        Label::new("DURATION").style(|s| {
            s.width(theme().dim.col_time)
                .color(theme().color.text_secondary)
                .font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
        }),
        Label::new("POST").style(|s| {
            s.width(theme().dim.col_time)
                .color(theme().color.text_secondary)
                .font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
        }),
        Label::new("").style(|s| s.width(theme().dim.col_menu)),
    ))
    .style(|s| {
        s.items_center()
            .width_full()
            .min_width(0.0)
            .height(theme().dim.height_cue_row)
            .gap(4.0)
            .padding_horiz(theme().dim.space_sm)
            .padding_left(theme().dim.space_xs)
            .border_bottom(1.0)
            .border_color(theme().color.element_border)
            .background(theme().color.bg_surface)
    });

    let add_btn = Container::new(app_icon(
        AppIcon::Plus,
        theme().dim.icon_sm as f32,
        theme().color.text_secondary,
    ))
    .style(|s| {
        s.items_center()
            .justify_center()
            .width(theme().dim.space_xl)
            .height(theme().dim.space_xl)
    });

    let footer = Container::new(add_btn).style(|s| {
        s.width_full()
            .items_center()
            .padding_vert(theme().dim.space_md)
            .padding_horiz(theme().dim.space_sm)
            .border_top(1.0)
            .border_color(theme().color.border_divider_40)
    });

    Stack::vertical((header, rows, footer)).style(|s| {
        s.flex_col()
            .min_size(0.0, 0.0)
            .width_full()
            .height_full()
            .align_items(AlignItems::Stretch)
            .background(theme().color.bg_surface)
    })
}
