//! Context-dependent detail panel — bottom inspector for selected cue.

use floem::IntoView;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate, SignalWith, create_rw_signal};
use floem::views::{Decorators, button, container, h_stack, h_stack_from_iter, label, scroll, text_input, v_stack, empty};

use std::sync::Arc;

use crate::model::{Cue, CueColor, CueId, CueKind, Cuelist, TriggerCondition};
use crate::style::theme;
use crate::ui::icons::{AppIcon, app_icon};

const COLOR_ORDER: &[CueColor] = &[
    CueColor::None, CueColor::Red, CueColor::Orange,
    CueColor::Green, CueColor::Blue, CueColor::Purple,
];

fn field_label(text: &'static str) -> impl IntoView {
    label(|| text.to_string()).style(|s| {
        s.font_family(theme().font.mono_sm.family.clone())
            .font_size(theme().font.mono_sm.size)
            .font_weight(floem::text::Weight::SEMIBOLD)
            .color(theme().color.text_disabled)
    })
}

fn text_field(label_text: &'static str, value: String, mono: bool) -> impl IntoView {
    v_stack((
        field_label(label_text),
        text_input(create_rw_signal(value)).style(move |s| {
            s.height(theme().dim.control_sm)
                .font_family(if mono { theme().font.mono_sm.family.clone() } else { theme().font.body.family.clone() })
                .font_size(theme().font.body.size)
                .padding_horiz(theme().dim.space_sm)
                .background(theme().color.element_bg)
                .border(1.0).border_color(theme().color.element_border)
                .border_radius(theme().dim.radius_sm)
                .color(theme().color.text_primary).outline(0.0).min_width(0.0)
                .focus_visible(|s| s.border_color(theme().color.border_focus))
        }),
    )).style(|s| s.flex_col().gap(theme().dim.space_xs))
}

fn duration_field(value: String) -> impl IntoView {
    let v = value;
    v_stack((
        field_label("Duration"),
        h_stack((
            label(move || v.clone()).style(|s| s.font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.body.size).color(theme().color.text_primary)),
            label(|| "(derived from media)".to_string()).style(|s| s.font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size).color(theme().color.text_disabled)),
        )).style(|s| s.items_center().gap(theme().dim.space_sm)),
    )).style(|s| s.flex_col().gap(theme().dim.space_xs))
}

fn trigger_selector(initial: TriggerCondition) -> impl IntoView {
    let mode = create_rw_signal(initial);
    v_stack((
        field_label("Trigger Condition"),
        h_stack((
            trigger_btn("Playhead", TriggerCondition::Playhead, mode),
            trigger_btn("With Cue", TriggerCondition::WithCue, mode),
            trigger_btn("After Cue", TriggerCondition::AfterCue, mode),
        )).style(|s| s.border_radius(theme().dim.radius_sm)
            .border(1.0).border_color(theme().color.element_border)),
    )).style(|s| s.flex_col().gap(theme().dim.space_sm))
}

fn trigger_btn(label_text: &'static str, this: TriggerCondition, mode: RwSignal<TriggerCondition>) -> impl IntoView {
    button(label_text).action(move || mode.set(this)).style(move |s| {
        let is_active = mode.get() == this;
        s.height(theme().dim.control_sm).padding_horiz(theme().dim.space_md).padding_vert(theme().dim.space_sm)
            .font_family(theme().font.body.family.clone()).font_size(12.0).font_weight(floem::text::Weight::MEDIUM)
            .outline(0.0).background(if is_active { theme().color.bg_selection } else { theme().color.element_bg })
            .color(if is_active { theme().color.text_primary } else { theme().color.text_disabled })
            .hover(|s| s.background(if is_active { theme().color.bg_selection } else { theme().color.element_bg_hover }))
            .focus_visible(|s| s.border_color(theme().color.border_focus))
    })
}

fn color_picker(initial: CueColor) -> impl IntoView {
    let color = create_rw_signal(initial);
    let swatches: Vec<_> = COLOR_ORDER.iter().map(|c| {
        let this = *c;
        let bg_color = match this {
            CueColor::None => theme().color.text_disabled_50,
            CueColor::Red => theme().color.status_error,
            CueColor::Orange => theme().color.status_group,
            CueColor::Green => theme().color.status_running,
            CueColor::Blue => theme().color.status_playhead,
            CueColor::Purple => theme().color.status_standby,
        };
        container(empty()).style(move |s| {
            let is_active = color.get() == this;
            s.size(theme().dim.space_xl, theme().dim.space_xl).border_radius(theme().dim.radius_full)
                .background(bg_color).outline(0.0)
                .apply_if(is_active, |s| s.border(2.0).border_color(theme().color.text_primary))
        }).on_click(move |_| { color.set(this); floem::event::EventPropagation::Stop }).into_any()
    }).collect();

    v_stack((
        field_label("Highlight"),
        h_stack_from_iter(swatches).style(|s| s.items_center().gap(theme().dim.space_sm)),
    )).style(|s| s.flex_col().gap(theme().dim.space_sm))
}

fn general_tab(cue: &Cue) -> impl IntoView {
    let number = cue.number.clone();
    let name = cue.name.clone();
    let notes = cue.notes.clone();
    let pre_wait = cue.pre_wait.clone();
    let post_wait = cue.post_wait.clone();
    let duration = cue.duration.clone();

    h_stack((
        v_stack((
            text_field("Number", number, true),
            text_field("Name", name, false),
            v_stack((
                field_label("Notes"),
                text_input(create_rw_signal(notes)).style(move |s| {
                    s.height(theme().dim.textarea_height).border_radius(theme().dim.radius_sm)
                        .border(1.0).border_color(theme().color.element_border).background(theme().color.element_bg)
                        .padding_vert(6.0).padding_horiz(theme().dim.space_sm)
                        .font_family(theme().font.body.family.clone()).font_size(theme().font.body.size)
                        .color(theme().color.text_primary).outline(0.0).min_width(0.0)
                        .focus_visible(|s| s.border_color(theme().color.border_focus))
                }),
            )).style(|s| s.flex_col().gap(theme().dim.space_xs)),
        )).style(|s| s.flex_col().gap(theme().dim.space_lg)),
        v_stack((
            text_field("Pre-delay", pre_wait, true),
            text_field("Post-delay", post_wait, true),
            duration_field(duration),
        )).style(|s| s.flex_col().gap(theme().dim.space_lg)),
        v_stack((
            trigger_selector(cue.trigger_condition),
            color_picker(cue.color),
        )).style(|s| s.flex_col().gap(theme().dim.space_lg)),
    )).style(|s| s.gap(theme().dim.space_xl))
}

fn audio_tab(cue: &Cue) -> impl IntoView {
    let media_file = cue.media_file.clone().unwrap_or_default();
    let volume = cue.volume;
    let vol_pct = create_rw_signal(volume);

    v_stack((
        v_stack((
            field_label("Media File"),
            h_stack((
                app_icon(AppIcon::FileAudio, theme().dim.icon_sm as f32, theme().color.status_playhead),
                label(move || media_file.clone()).style(|s| s.font_family(theme().font.mono_sm.family.clone())
                    .font_size(theme().font.mono_sm.size).color(theme().color.text_primary).min_width(0.0).flex_grow(1.0)),
                button("Browse…").style(|s| s.height(theme().dim.control_sm).padding_horiz(theme().dim.space_sm)
                    .padding_vert(theme().dim.space_xs).font_family(theme().font.body.family.clone())
                    .font_size(theme().font.mono_sm.size).background(theme().color.element_bg_hover)
                    .border(1.0).border_color(theme().color.element_border).border_radius(theme().dim.radius_sm)
                    .color(theme().color.text_secondary).outline(0.0).hover(|s| s.background(theme().color.bg_surface_raised))
                    .focus_visible(|s| s.border_color(theme().color.border_focus))),
            )).style(|s| s.items_center().gap(theme().dim.space_sm).height(theme().dim.control_sm)
                .border_radius(theme().dim.radius_sm).border(1.0).border_color(theme().color.element_border)
                .background(theme().color.element_bg).padding_horiz(theme().dim.space_sm)),
        )).style(|s| s.flex_col().gap(theme().dim.space_xs)),
        v_stack((
            field_label("Target Volume"),
            h_stack((
                label(move || format!("{:.0}%", vol_pct.get() * 100.0)).style(|s| s.font_family(theme().font.mono_sm.family.clone())
                    .font_size(theme().font.mono_sm.size).color(theme().color.text_secondary).min_width(40.0)),
                label(|| "".to_string()).style(|s| s.flex_grow(1.0)),
            )).style(|s| s.items_center().gap(theme().dim.space_sm)),
        )).style(|s| s.flex_col().gap(theme().dim.space_sm)),
        h_stack((
            text_field("Fade In", "00:02".into(), true),
            text_field("Fade Out", "00:03".into(), true),
        )).style(|s| s.gap(theme().dim.space_md)),
    )).style(|s| s.flex_col().gap(theme().dim.space_lg))
}

fn osc_tab(cue: &Cue) -> impl IntoView {
    let (task, host, port) = match &cue.kind {
        CueKind::Osc { task, host, port } => (task.clone(), host.clone(), *port),
        _ => ("/projector/power 1".into(), "10.0.0.42".into(), 3333),
    };
    v_stack((
        text_field("OSC Task", task, false),
        text_field("Host", host, false),
        text_field("Port", port.to_string(), true),
    )).style(|s| s.flex_col().gap(theme().dim.space_lg))
}

fn empty_state() -> impl IntoView {
    container(label(|| "Select a cue to edit its settings".to_string()).style(|s| s.color(theme().color.text_disabled)
        .font_size(theme().font.body.size)))
    .style(|s| s.items_center().justify_center().size_full())
}

fn tab_button(label: &'static str, active: RwSignal<&'static str>, this: &'static str) -> impl IntoView {
    button(label).action(move || active.set(this)).style(move |s| {
        let is_active = active.get() == this;
        s.height(theme().dim.control_sm).padding_horiz(theme().dim.space_md).padding_vert(theme().dim.space_sm)
            .font_family(theme().font.body.family.clone()).font_size(12.0).font_weight(floem::text::Weight::MEDIUM)
            .outline(0.0).background(if is_active { theme().color.bg_selection } else { theme().color.element_bg })
            .color(if is_active { theme().color.text_primary } else { theme().color.text_disabled })
            .hover(|s| s.background(if is_active { theme().color.bg_selection } else { theme().color.element_bg_hover }).color(theme().color.text_primary))
            .focus_visible(|s| s.border_color(theme().color.border_focus))
    })
}

pub fn view(
    selected: RwSignal<Option<CueId>>,
    cuelist: impl SignalGet<Arc<Cuelist>> + SignalWith<Arc<Cuelist>> + Copy + 'static,
) -> impl IntoView {
    let active_tab: RwSignal<&'static str> = create_rw_signal("general");

    let cue = cuelist.get();
    let selected_cue = selected.get().and_then(|id| cue.get_cue(id).cloned());

    let tabs: Vec<&'static str> = {
        let mut t = vec!["general"];
        if let Some(ref cue) = selected_cue {
            match cue.kind {
                CueKind::Audio { .. } => t.push("audio"),
                CueKind::Osc { .. } => t.push("osc"),
                _ => {}
            }
        }
        t
    };

    let tab_bar = h_stack_from_iter(tabs.iter().map(|t| {
        let label_str = match *t {
            "general" => "General",
            "audio" => "Audio",
            "osc" => "OSC",
            _ => "General",
        };
        tab_button(label_str, active_tab, *t).into_any()
    }).collect::<Vec<_>>())
    .style(|s| s.flex_shrink(0.0).items_center().gap(0.0).background(theme().color.element_bg_hover));

    let selected_cue_name = selected_cue.as_ref().map(|c| format!("{} · {}", c.number, c.name));
    let header_info = h_stack((
        label(|| "Cue Settings".to_string()).style(|s| s.font_family(theme().font.body.family.clone())
            .font_size(theme().font.body.size).font_weight(floem::text::Weight::SEMIBOLD).color(theme().color.text_primary)),
        label(move || selected_cue_name.clone().unwrap_or_default())
            .style(|s| s.font_family(theme().font.mono_sm.family.clone()).font_size(theme().font.mono_sm.size).color(theme().color.text_disabled)),
    )).style(|s| s.items_center().gap(theme().dim.space_md).padding_horiz(theme().dim.space_md).border_left(1.0).border_color(theme().color.element_border));

    let header = h_stack((tab_bar, header_info))
        .style(|s| s.flex_shrink(0.0).items_center().border_bottom(1.0).border_color(theme().color.element_border));

    let content = match &selected_cue {
        Some(cue) => {
            let tab_content: floem::AnyView = match active_tab.get() {
                "general" => general_tab(cue).into_any(),
                "audio" => audio_tab(cue).into_any(),
                "osc" => osc_tab(cue).into_any(),
                _ => general_tab(cue).into_any(),
            };
            scroll(tab_content).style(|s| s.flex_grow(1.0).min_height(0.0).padding(theme().dim.space_lg)).into_any()
        }
        None => empty_state().into_any(),
    };

    v_stack((header, content))
        .style(|s| s.flex_col().height(theme().dim.detail_height).flex_shrink(0.0)
            .background(theme().color.bg_surface).width_full())
}