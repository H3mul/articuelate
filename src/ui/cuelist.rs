//! Cuelist — the main cue table.
//!
//! Uses Floem's virtual_list for performance with large show files.

use floem::IntoView;
use floem::event::EventPropagation;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate, SignalWith, create_memo};
use floem::style::AlignItems;
use floem::views::{
    Decorators, VirtualDirection, VirtualItemSize, container, h_stack, label, scroll, text,
    v_stack, virtual_list, empty,
};

use std::sync::Arc;

use crate::model::{Cue, CueColor, CueId, CueKind, CueState, Cuelist};
use crate::style::theme;
use crate::ui::icons::{AppIcon, app_icon};

fn microtime(mmss: &str) -> String {
    if let Some((m, s)) = mmss.split_once(':') {
        let m_int: usize = m.parse().unwrap_or(0);
        format!("{}:{}.00", m_int, s)
    } else {
        mmss.to_string()
    }
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

    container(
        text(value).style(move |s| {
            s.font_family(theme().font.mono_sm.family.clone())
                .font_size(if running { theme().font.body.size } else { theme().font.mono_sm.size })
                .color(if running { theme().color.text_primary } else { theme().color.text_disabled })
                .min_width(0.0)
                .padding_horiz(6.0)
        })
    )
    .style(move |s| {
        let mut s = s.height(theme().dim.time_cell)
            .min_width(0.0)
            .items_center()
            .justify_end()
            .border_radius(theme().dim.radius_sm);

        if has_fill {
            // Use a linear-style fill: set background with a clip
            // Floem doesn't support partial fills natively, so we'll
            // use a simple background color for the fill state
            s = s.background(theme().color.status_running_bg_30);
        }
        if running {
            s = s.background(theme().color.status_running_bg_30);
        }
        s
    })
}

fn cue_row(
    position: usize,
    id: CueId,
    cue: Arc<Cue>,
    selected: RwSignal<Option<CueId>>,
    active_cue: RwSignal<Option<CueId>>,
) -> impl IntoView {
    let is_running = active_cue.get() == Some(id);
    let is_selected = selected.get() == Some(id);
    let is_standby = cue.state == CueState::Standby;
    let is_group = matches!(cue.kind, CueKind::Group);
    let stripe_color = match cue.color {
        CueColor::None => None,
        CueColor::Red => Some(theme().color.status_error),
        CueColor::Orange => Some(theme().color.status_group),
        CueColor::Green => Some(theme().color.status_running),
        CueColor::Blue => Some(theme().color.status_playhead),
        CueColor::Purple => Some(theme().color.status_standby),
    };

    let row_id = id;
    let name = cue.name.clone();
    let number = cue.number.clone();
    let target = strip_type(&cue.target);
    let kind = cue.kind.clone();
    let pre_wait = microtime(&cue.pre_wait);
    let duration = microtime(&cue.duration);
    let post_wait = microtime(&cue.post_wait);

    let pre_progress = cue.pre_progress;
    let progress = cue.progress;
    let post_progress = cue.post_progress;

    let row = h_stack((
        // Drag handle
        app_icon(AppIcon::GripVertical, theme().dim.icon_sm as f32, theme().color.text_disabled)
            .style(|s| s.width(theme().dim.col_drag).items_center().justify_center()),
        // Playhead indicator
        container(
            if is_standby || is_running {
                app_icon(AppIcon::Play, theme().dim.icon_sm as f32,
                    if is_standby { theme().color.status_standby } else { theme().color.status_running }).into_any()
            } else {
                empty().into_any()
            }
        )
        .style(|s| s.width(theme().dim.col_playhead).items_center().justify_center()),
        // Cue number
        label(move || number.clone()).style(move |s| {
            s.width(theme().dim.col_cue_number)
                .font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
                .color(if is_running { theme().color.status_running } else { theme().color.text_secondary })
                .min_width(0.0)
        }),
        // Name + icon
        h_stack((
            app_icon(kind_icon(&kind), theme().dim.icon_sm as f32, theme().color.text_disabled),
            label(move || name.clone()).style(move |s| {
                s.font_family(theme().font.body.family.clone())
                    .font_size(theme().font.body.size)
                    .font_weight(floem::text::Weight::MEDIUM)
                    .color(theme().color.text_primary)
                    .min_width(0.0)
            }),
        ))
        .style(move |s| {
            s.min_width(0.0).flex_grow(1.0).items_center().gap(theme().dim.space_sm)
                .padding_left(theme().dim.icon_sm * cue.depth as f64)
        }),
        // Target
        label(move || target.clone()).style(|s| {
            s.font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
                .color(theme().color.text_disabled)
                .min_width(0.0).flex_grow(1.0)
        }),
        // Pre-wait
        time_cell(pre_wait, pre_progress, false),
        // Duration
        time_cell(duration, progress, is_running),
        // Post-wait
        time_cell(post_wait, post_progress, false),
        // Menu button
        app_icon(AppIcon::EllipsisVertical, theme().dim.icon_sm as f32, theme().color.text_secondary)
            .style(|s| s.width(theme().dim.col_menu).items_center().justify_center()),
    ))
    .style(move |s| {
        s.items_center().width_full().min_width(0.0)
            .height(theme().dim.height_cue_row).gap(4.0)
            .padding_horiz(theme().dim.space_sm).padding_left(theme().dim.space_xs)
    });

    let row_id_clone = id;
    container(row)
        .style(move |s| {
            let mut s = s.width_full().min_width(0.0)
                .height(theme().dim.height_cue_row)
                .border_bottom(1.0).border_color(theme().color.border_row_divider);

            if active_cue.get() == Some(row_id_clone) {
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
                s = s.border_left(3.0).border_color(theme().color.status_standby);
            }
            if let Some(c) = stripe_color {
                s = s.border_left(3.0).border_color(c);
            }
            s
        })
        .on_click(move |_| {
            selected.set(Some(row_id_clone));
            EventPropagation::Stop
        })
        .into_any()
}

pub fn view(
    cuelist: impl SignalGet<Arc<Cuelist>> + SignalWith<Arc<Cuelist>> + Copy + 'static,
    selected: RwSignal<Option<CueId>>,
    active_cue: RwSignal<Option<CueId>>,
) -> impl IntoView {
    let filtered = create_memo(move |_| {
        cuelist.with(|list| {
            list.iter()
                .enumerate()
                .map(|(i, cue)| (i + 1, cue.id, cue.clone()))
                .collect::<im::Vector<(usize, CueId, Arc<Cue>)>>()
        })
    });

    let rows = virtual_list(
        VirtualDirection::Vertical,
        VirtualItemSize::Fixed(Box::new(|| theme().dim.height_cue_row)),
        move || filtered.get(),
        |(_, id, _)| *id,
        move |(pos, id, cue)| cue_row(pos, id, cue, selected, active_cue),
    )
    .style(|s| {
        s.width_full().flex_col().min_width(0.0).min_height(0.0)
            .align_items(AlignItems::Stretch)
    });

    let rows = scroll(rows).style(|s| {
        s.width_full().flex_col().min_size(0.0, 0.0)
            .align_items(AlignItems::Stretch).flex_grow(1.0)
    });

    let header = h_stack((
        text("").style(|s| s.width(theme().dim.col_drag)),
        text("").style(|s| s.width(theme().dim.col_playhead)),
        text("CUE").style(|s| s.width(theme().dim.col_cue_number).color(theme().color.text_secondary)
            .font_family(theme().font.mono_sm.family.clone()).font_size(theme().font.mono_sm.size)),
        text("NAME").style(|s| s.flex_grow(1.0).color(theme().color.text_secondary)
            .font_family(theme().font.mono_sm.family.clone()).font_size(theme().font.mono_sm.size).min_width(0.0)),
        text("TARGET").style(|s| s.flex_grow(1.0).color(theme().color.text_secondary)
            .font_family(theme().font.mono_sm.family.clone()).font_size(theme().font.mono_sm.size).min_width(0.0)),
        text("PRE").style(|s| s.width(theme().dim.col_time).color(theme().color.text_secondary)
            .font_family(theme().font.mono_sm.family.clone()).font_size(theme().font.mono_sm.size)),
        text("DURATION").style(|s| s.width(theme().dim.col_time).color(theme().color.text_secondary)
            .font_family(theme().font.mono_sm.family.clone()).font_size(theme().font.mono_sm.size)),
        text("POST").style(|s| s.width(theme().dim.col_time).color(theme().color.text_secondary)
            .font_family(theme().font.mono_sm.family.clone()).font_size(theme().font.mono_sm.size)),
        text("").style(|s| s.width(theme().dim.col_menu)),
    ))
    .style(|s| {
        s.items_center().width_full().min_width(0.0)
            .height(theme().dim.height_cue_row).gap(4.0)
            .padding_horiz(theme().dim.space_sm).padding_left(theme().dim.space_xs)
            .border_bottom(1.0).border_color(theme().color.element_border)
            .background(theme().color.bg_surface)
    });

    let add_btn = container(
        app_icon(AppIcon::Plus, theme().dim.icon_sm as f32, theme().color.text_secondary)
    )
    .style(|s| {
        s.items_center().justify_center().width(theme().dim.space_xl).height(theme().dim.space_xl)
    });

    let footer = container(add_btn)
        .style(|s| {
            s.width_full().items_center().padding_vert(theme().dim.space_md)
                .padding_horiz(theme().dim.space_sm).border_top(1.0)
                .border_color(theme().color.border_divider_40)
        });

    v_stack((header, rows, footer))
        .style(|s| {
            s.flex_col().min_size(0.0, 0.0).width_full().height_full()
                .align_items(AlignItems::Stretch).background(theme().color.bg_surface)
        })
}