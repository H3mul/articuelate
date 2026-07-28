//! Transport toolbar with conductor panel.
//!
//! Layout (matching prototype):
//! ┌──────────┐ ┌────────────────────────────────────────────────┐
//! │  Panic   │ │  [Playing] 3  Thunderclap                      │
//! │   GO     │ │  [Next]     4  Rain Ambience                   │
//! │          │ │              Loop through the storm scene.     │
//! └──────────┘ └────────────────────────────────────────────────┘

use floem::IntoView;
use floem::reactive::{Memo, SignalGet};
use floem::views::{Decorators, button, h_stack, label, text, v_stack};
use lucide_floem::Icon;

use crate::exec::ExecutionHandle;
use crate::exec::UiEvent;
use crate::model::TransientCueState;
use crate::style::style::{
    BadgeChip, BadgeRunning, BadgeSm, BtnGo, BtnPanic, ConductorCurrent, ConductorNext,
    TransportGroup,
};
use crate::style::theme;
use crate::ui::icons::{AppIcon, app_icon};

pub fn view(
    events: ExecutionHandle,
    active_transient: Memo<Option<TransientCueState>>,
    next_transient: Memo<Option<TransientCueState>>,
) -> impl IntoView {
    let events_panic = events.clone();
    let events_go = events.clone();
    let panic_btn = button(v_stack((
        app_icon(
            AppIcon::Panic,
            theme().dim.icon_sm as f32,
            theme().color.status_error,
        ),
        text("Panic").style(|s| {
            s.color(theme().color.status_error)
                .font_size(10.0)
                .font_weight(floem::text::Weight::SEMIBOLD)
        }),
    )))
    .class(BtnPanic)
    .style(|s| s.height(theme().dim.control_md))
    .action(move || {
        let _ = events_panic.send_user_intent(UiEvent::Panic);
    });

    let go_btn = button(h_stack((
        Icon::Play.into_view().style(move |s| {
            s.size(theme().dim.icon_md, theme().dim.icon_md)
                .color(theme().color.status_running)
        }),
        text("GO").style(|s| {
            s.color(theme().color.text_primary)
                .font_weight(floem::text::Weight::BOLD)
                .font_size(16.0)
        }),
    )))
    .class(BtnGo)
    .style(|s| s.height(theme().dim.control_md))
    .action(move || {
        let _ = events_go.send_user_intent(UiEvent::Go);
    });

    // Transport group: Panic + GO stacked
    let transport_group = v_stack((panic_btn, go_btn)).class(TransportGroup);

    // Current cue conductor — derived from the active TransientCueState
    let current_cue = {
        let active = active_transient;
        h_stack((
            label(|| "Playing".to_string())
                .class(BadgeSm)
                .class(BadgeRunning)
                .class(BadgeChip),
            label(move || {
                active
                    .get()
                    .map(|tcs| {
                        let cue = tcs.workspace.get();
                        cue.name.clone()
                    })
                    .unwrap_or_else(|| "—".to_string())
            })
            .style(|s| {
                s.font_family(theme().font.body.family.clone())
                    .font_size(theme().font.body.size)
                    .color(theme().color.text_disabled)
                    .min_width(0.0)
            }),
        ))
        .class(ConductorCurrent)
    };

    // Next cue conductor — derived from the next TransientCueState
    let next_cue = {
        let next = next_transient;
        h_stack((
            label(|| "Next".to_string()).class(BadgeSm).style(|s| {
                s.background(theme().color.status_playhead_bg)
                    .color(theme().color.status_playhead)
                    .min_width(68.0)
            }),
            label(move || {
                next.get()
                    .map(|tcs| {
                        let cue = tcs.workspace.get();
                        cue.name.clone()
                    })
                    .unwrap_or_else(|| "—".to_string())
            })
            .style(|s| {
                s.font_family(theme().font.body.family.clone())
                    .font_size(theme().font.heading.size as f32)
                    .font_weight(floem::text::Weight::SEMIBOLD)
                    .color(theme().color.text_primary)
            }),
            v_stack((
                label(move || {
                    next.get()
                        .map(|tcs| {
                            let cue = tcs.workspace.get();
                            cue.name.clone()
                        })
                        .unwrap_or_else(|| "—".to_string())
                })
                .style(|s| {
                    s.font_family(theme().font.body.family.clone())
                        .font_size(theme().font.heading.size as f32)
                        .font_weight(floem::text::Weight::SEMIBOLD)
                        .color(theme().color.text_primary)
                        .min_width(0.0)
                }),
                label(move || {
                    next.get()
                        .map(|tcs| {
                            let cue = tcs.workspace.get();
                            cue.notes.clone()
                        })
                        .unwrap_or_default()
                })
                .style(|s| {
                    s.font_family(theme().font.body.family.clone())
                        .font_size(theme().font.mono_sm.size)
                        .font_style(floem::text::Style::Italic)
                        .color(theme().color.text_disabled)
                        .min_width(0.0)
                }),
            ))
            .style(|s| s.flex_col().min_width(0.0)),
        ))
        .class(ConductorNext)
    };

    let conductor = v_stack((current_cue, next_cue)).style(|s| {
        s.flex_col()
            .min_width(0.0)
            .flex_grow(1.0)
            .gap(theme().dim.space_sm)
    });

    h_stack((transport_group, conductor)).style(|s| {
        s.items_center()
            .width_full()
            .gap(theme().dim.space_sm)
            .padding(theme().dim.space_sm)
            .background(theme().color.bg_surface)
    })
}
