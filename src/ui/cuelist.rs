//! Main cuelist table.
//!
//! The view stays virtualized while presenting a compact, Zed-inspired table:
//! a small icon gutter, one-based cue position, cue name, and timing columns.

use floem::IntoView;
use floem::event::EventPropagation;
use floem::reactive::{Memo, RwSignal, SignalGet, SignalUpdate, SignalWith, create_memo};
use floem::style::AlignItems;
use floem::views::{
    Decorators, VirtualDirection, VirtualItemSize, container, h_stack, label, scroll, text,
    v_stack, virtual_list,
};

use std::sync::Arc;

use crate::model::{Cue, CueId, Cuelist, TriggerMode};
use crate::style::*;

const NUMBER_WIDTH: f32 = 56.0;
const TIME_WIDTH: f32 = 92.0;

pub fn view(
    cuelist: impl SignalGet<Arc<Cuelist>> + SignalWith<Arc<Cuelist>> + Copy + 'static,
    selected: RwSignal<Option<CueId>>,
    active_cue: RwSignal<Option<CueId>>,
) -> impl IntoView {
    let filtered: Memo<im::Vector<(usize, CueId, Arc<Cue>)>> = create_memo(move |_| {
        cuelist.with(|list| {
            list.iter()
                .enumerate()
                .map(|(display_index, cue)| (display_index + 1, cue.id, cue.clone()))
                .collect()
        })
    });

    let rows = virtual_list(
        VirtualDirection::Vertical,
        VirtualItemSize::Fixed(Box::new(|| theme().dim.height_cue_row)),
        move || filtered.get(),
        |(_, id, _)| *id,
        move |(position, id, cue)| cue_row(position, id, cue, selected, active_cue),
    )
    .style(|s| {
        s.width_full()
            .flex_col()
            .min_width(0.0)
            .min_height(0.0)
            .align_items(AlignItems::Stretch)
    });

    let rows = scroll(rows).style(|s| {
        s.width_full()
            .flex_col()
            .min_size(0.0, 0.0)
            .align_items(AlignItems::Stretch)
    });

    v_stack((table_header(), rows)).style(|s| {
        s.flex_col()
            .flex_col()
            .min_size(0.0, 0.0)
            .width_full()
            .align_items(AlignItems::Stretch)
            .background(theme().color.bg_surface)
    })
}

fn table_header() -> impl IntoView {
    h_stack((
        header_cell("#", NUMBER_WIDTH),
        header_cell("Cue", 0.0).style(|s| s.flex_grow(1.0).min_width(0.0)),
        header_cell("Pre-Delay", TIME_WIDTH),
        header_cell("Duration", TIME_WIDTH),
        header_cell("Post-Delay", TIME_WIDTH),
    ))
    .style(|s| {
        s.items_center()
            .width_full()
            .min_width(0.0)
            .height(theme().dim.height_cue_row)
            .background(theme().color.bg_surface)
            .border_bottom(theme().dim.border_size)
            .border_color(theme().color.border_divider)
    })
}

fn header_cell(title: &'static str, width: f32) -> impl IntoView {
    text(title).style(move |s| {
        s.apply_if(width > 0.0, |s| s.width(width))
            .padding_horiz(theme().dim.space_sm)
            .color(theme().color.text_secondary)
            .font_family(theme().font.body.family.clone())
            .font_size(theme().font.body.size as f32)
    })
}

fn cue_row(
    position: usize,
    id: CueId,
    cue: Arc<Cue>,
    selected: RwSignal<Option<CueId>>,
    active_cue: RwSignal<Option<CueId>>,
) -> impl IntoView {
    let _is_group = cue.trigger.mode != TriggerMode::Playhead;
    let name = cue.name.clone();
    // let icon = match cue.trigger.mode {
    //     TriggerMode::Playhead => Icon::Play,
    //     TriggerMode::WithCue => Icon::ArrowRight,
    //     TriggerMode::AfterCue => Icon::ArrowRight,
    // };

    let row = h_stack((
        // container(icon).style(|s| {
        //     s.items_center()
        //         .justify_center()
        //         .width(ICON_WIDTH)
        //         .height(theme().dim.height_cue_row)
        // }),
        text(position.to_string()).style(|s| {
            s.width(NUMBER_WIDTH)
                .padding_horiz(theme().dim.space_sm)
                .color(theme().color.text_secondary)
                .font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size as f32)
        }),
        label(move || name.clone()).style(|s| {
            s.flex_grow(1.0)
                .min_width(0.0)
                .padding_horiz(theme().dim.space_sm)
                .color(theme().color.text_primary)
                .font_family(theme().font.body.family.clone())
                .font_size(theme().font.body.size as f32)
        }),
        time_cell(),
        time_cell(),
        time_cell(),
    ))
    .style(|s| {
        s.items_center()
            .width_full()
            .min_width(0.0)
            .height(theme().dim.height_cue_row)
    });

    let row_id = id;
    container(row)
        .style(move |s| {
            let background = if active_cue.get() == Some(row_id) {
                theme().color.status_running.multiply_alpha(0.22)
            } else if selected.get() == Some(row_id) {
                theme().color.bg_selection_active
            } else if position % 2 == 0 {
                theme().color.bg_surface_raised
            } else {
                theme().color.bg_surface
            };

            s.width_full()
                .height(theme().dim.height_cue_row)
                .background(background)
                .border_bottom(theme().dim.border_size)
                .border_color(theme().color.border_subtle)
                // .apply_if(is_group, |s| {
                //     s.border_left(theme().dim.border_size)
                //         .border_color(theme().color.status_group)
                // })
                .apply_if(active_cue.get() == Some(row_id), |s| {
                    s.border_left(theme().dim.border_size)
                        .border_color(theme().color.status_playhead)
                })
        })
        .on_click(move |_| {
            selected.set(Some(row_id));
            EventPropagation::Stop
        })
}

fn time_cell() -> impl IntoView {
    text("0.0s").style(|s| {
        s.width(TIME_WIDTH)
            .padding_horiz(theme().dim.space_sm)
            .color(theme().color.text_disabled)
            .font_family(theme().font.mono_sm.family.clone())
            .font_size(theme().font.mono_sm.size as f32)
    })
}
