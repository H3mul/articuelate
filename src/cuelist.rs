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
    Decorators, VirtualDirection, VirtualItemSize, container, h_stack, label, list, text, v_stack,
    virtual_list,
};
use lucide_floem::Icon;

use crate::model::{ActionKind, Cue, CueAction, FollowMode};
use crate::theme::*;

/// Fixed row height (px) for the virtualized list.
const ROW_H: f64 = 82.0;

pub fn view(
    cues: RwSignal<im::Vector<Cue>>,
    selected: RwSignal<usize>,
    active_cue: RwSignal<usize>,
    search: RwSignal<String>,
) -> impl IntoView {
    // Filtered + re-indexed copy of the flat chain.
    let filtered: Memo<im::Vector<(usize, Cue)>> = create_memo(move |_| {
        let q = search.with(|s| s.to_lowercase());
        cues.with(|cs| {
            cs.iter()
                .enumerate()
                .filter(|(_, c)| {
                    q.is_empty() || c.name.to_lowercase().contains(&q) || c.number.contains(&q)
                })
                .map(|(i, c)| (i, c.clone()))
                .collect::<im::Vector<_>>()
        })
    });

    let list = virtual_list(
        VirtualDirection::Vertical,
        VirtualItemSize::Fixed(Box::new(|| ROW_H)),
        move || filtered.get(),
        move |(i, _)| *i,
        move |(i, cue)| cue_row(i, cue, selected, active_cue),
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
    index: usize,
    cue: Cue,
    selected: RwSignal<usize>,
    active_cue: RwSignal<usize>,
) -> impl IntoView {
    let Cue {
        number,
        name,
        follow,
        depth,
        actions,
    } = cue;

    // Fresh copies for each closure site (RwSignal/usize are Copy).
    let (sel, act, idx) = (selected, active_cue, index);

    let follow_badge = match follow {
        FollowMode::Go => text(follow.badge()).style(|s| {
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

    let (act_num, idx_num) = (act, idx);
    let header_line = h_stack((
        label(move || number.clone()).style(move |s| {
            s.font_family(theme().font.mono.to_string())
                .color(if act_num.get() == idx_num {
                    theme().color.accent
                } else {
                    theme().color.text_dim
                })
                .font_size(13.0)
                .min_width(34.0)
        }),
        label(move || name.clone()).style(|s| {
            s.color(theme().color.fg)
                .font_size(13.0)
                .font_weight(floem::text::Weight::BOLD)
        }),
        text("").style(|s| s.flex_grow(1.0)),
        follow_badge,
    ))
    .style(|s| s.items_center().gap(8.0));

    let action_lines =
        list(actions.into_iter().map(action_line).collect::<Vec<_>>()).style(move |s| {
            s.flex_col()
                .gap(2.0)
                .margin_left((depth as f64) * 14.0 + 34.0)
        });

    let (sel_s, sel_i) = (sel, idx);
    let is_selected = move || sel_s.get() == sel_i;

    let (act_c, idx_c) = (act, idx);
    container(v_stack((header_line, action_lines)).style(|s| s.flex_col().gap(4.0)))
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
                .apply_if(act_c.get() == idx_c, |s| {
                    s.border_left(3.0).border_color(theme().color.accent)
                })
        })
        .on_click(move |_| {
            let (s, i) = (sel, idx);
            s.set(i);
            EventPropagation::Stop
        })
}

fn action_line(a: CueAction) -> impl IntoView {
    let CueAction {
        kind,
        target,
        detail,
    } = a;
    let (glyph_icon, glyph_color) = match kind {
        ActionKind::Play => (Icon::Play, theme().color.accent),
        ActionKind::Fade => (Icon::TrendingDown, theme().color.lapce_blue),
        ActionKind::Stop => (Icon::Square, theme().color.panic),
    };
    let glyph = glyph_icon
        .into_view()
        .style(move |s| s.size(12.0, 12.0).color(glyph_color).min_width(14.0));
    h_stack((
        glyph,
        label(move || target.clone()).style(|s| s.color(theme().color.fg).font_size(12.0)),
        label(move || detail.clone()).style(|s| {
            s.color(theme().color.text_dim)
                .font_size(11.0)
                .font_family(theme().font.mono.to_string())
        }),
    ))
    .style(|s| s.items_center().gap(6.0))
}
