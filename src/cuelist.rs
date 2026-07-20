//! Main cuelist - the central "editor" pane, rendered with Floem's
//! `virtual_list` so a 5,000-cue show stays instant and memory-light.
//!
//! Selection (click) and the live "active" cue are driven by signals, so the
//! rest of the UI reacts surgically without rebuilding the list.

use floem::IntoView;
use floem::event::EventPropagation;
use floem::peniko::Color;
use floem::reactive::{Memo, RwSignal, SignalGet, SignalUpdate, SignalWith, create_memo};
use floem::views::{
    Decorators, VirtualDirection, VirtualItemSize, container, h_stack, label, text, v_stack,
    virtual_list,
};

use std::sync::Arc;

use crate::model::{Cue, CueId, Cuelist, Trigger, TriggerMode};
use crate::theme::*;

/// Fixed row height (px) for the virtualized list.
const ROW_H: f64 = 82.0;

pub fn view(
    cuelist: impl SignalGet<Arc<Cuelist>> + SignalWith<Arc<Cuelist>> + Copy + 'static,
    selected: RwSignal<Option<CueId>>,
    active_cue: RwSignal<Option<CueId>>,
    search: RwSignal<String>,
) -> impl IntoView {
    // Filtered copy of the flat chain, keyed by CueId (not an index), so the
    // list stays correct even when cues reorder at runtime.
    let filtered: Memo<im::Vector<(CueId, Arc<Cue>)>> = create_memo(move |_| {
        let q = search.with(|s| s.to_lowercase());
        cuelist.with(|cl| {
            cl.iter()
                .filter(|c| q.is_empty() || c.name.to_lowercase().contains(&q))
                .map(|c| (c.id, c.clone()))
                .collect::<im::Vector<_>>()
        })
    });

    let list = virtual_list(
        VirtualDirection::Vertical,
        VirtualItemSize::Fixed(Box::new(|| ROW_H)),
        move || filtered.get(),
        move |(id, _)| *id,
        move |(id, cue)| cue_row(id, cue, selected, active_cue),
    )
    .style(|s| s.flex_col().width_full());

    let header = text("CUES").style(|s| {
        s.font_family(theme().font.mono.to_string())
            .font_size(11.0)
            .color(theme().color.text_dim)
            .padding_horiz(12.0)
            .padding_vert(8.0)
            .background(theme().color.panel)
            .border_bottom(1.0)
            .border_color(theme().color.border)
            .width_full()
    });

    v_stack((
        header,
        list.style(|s| s.width_full().flex_grow(1.0).min_height(0.0)),
    ))
    .style(|s| {
        s.flex_col()
            .flex_grow(1.0)
            .min_width(0.0)
            .background(theme().color.bg)
            .border_right(1.0)
            .border_color(theme().color.border)
    })
}

fn cue_row(
    id: CueId,
    cue: Arc<Cue>,
    selected: RwSignal<Option<CueId>>,
    active_cue: RwSignal<Option<CueId>>,
) -> impl IntoView {
    let name = cue.name.clone();
    let trigger = cue.trigger;

    // Fresh copies for each closure site (RwSignal/CueId are Copy).
    let (sel, act, cid) = (selected, active_cue, id);

    let trigger_badge = match trigger {
        Trigger {
            mode: TriggerMode::Playhead,
            ..
        } => text(trigger.badge()).style(|s| {
            s.color(theme().color.text_dim)
                .font_family(theme().font.mono.to_string())
                .font_size(9.0)
                .border(1.0)
                .border_color(theme().color.border)
                .border_radius(3.0)
                .padding_horiz(5.0)
                .padding_vert(1.0)
        }),
        other => text(other.badge()).style(|s| {
            s.color(theme().color.lapce_blue)
                .font_family(theme().font.mono.to_string())
                .font_size(9.0)
                .border(1.0)
                .border_color(theme().color.lapce_blue.multiply_alpha(0.5))
                .border_radius(3.0)
                .padding_horiz(5.0)
                .padding_vert(1.0)
        }),
    };

    let header_line = h_stack((
        label(move || name.clone()).style(|s| {
            s.color(theme().color.fg)
                .font_size(13.0)
                .font_weight(floem::text::Weight::BOLD)
        }),
        text("").style(|s| s.flex_grow(1.0)),
        trigger_badge,
    ))
    .style(|s| s.items_center().gap(8.0));

    let is_selected = move || sel.get() == Some(cid);

    let (act_c, cid_c) = (act, cid);
    container(header_line.style(|s| s.flex_col().gap(4.0)))
        .style(move |s| {
            s.width_full()
                .height(ROW_H)
                .padding_horiz(12.0)
                .padding_vert(8.0)
                .background(if is_selected() {
                    theme().color.accent_dim.multiply_alpha(0.22)
                } else {
                    Color::TRANSPARENT
                })
                .border_bottom(1.0)
                .border_color(theme().color.border)
                .apply_if(act_c.get() == Some(cid_c), |s| {
                    s.border_left(3.0).border_color(theme().color.accent)
                })
        })
        .on_click(move |_| {
            sel.set(Some(cid));
            EventPropagation::Stop
        })
}
