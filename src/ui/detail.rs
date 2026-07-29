//! Context-dependent detail panel — bottom inspector for selected cue.

use floem::IntoView;
use floem::reactive::{Memo, RwSignal, SignalGet, SignalUpdate};
use floem::views::{Button, Container, Decorators, Empty, Label, Stack, TextInput};

use crate::model::{CueColor, CueKind, TransientCueState, TriggerCondition};
use crate::style::theme;
use crate::ui::icons::{AppIcon, app_icon};

const COLOR_ORDER: &[CueColor] = &[
    CueColor::None,
    CueColor::Red,
    CueColor::Orange,
    CueColor::Green,
    CueColor::Blue,
    CueColor::Purple,
];

fn field_label(text: &'static str) -> impl IntoView {
    Label::derived(|| text.to_string()).style(|s| {
        s.font_family(theme().font.mono_sm.family.clone())
            .font_size(theme().font.mono_sm.size)
            .font_weight(floem::text::FontWeight::SEMI_BOLD)
            .color(theme().color.text_disabled)
    })
}

fn text_field(label_text: &'static str, value: String, mono: bool) -> impl IntoView {
    Stack::vertical((
        field_label(label_text),
        TextInput::new(RwSignal::new(value)).style(move |s| {
            s.height(theme().dim.control_sm)
                .font_family(if mono {
                    theme().font.mono_sm.family.clone()
                } else {
                    theme().font.body.family.clone()
                })
                .font_size(theme().font.body.size)
                .padding_horiz(theme().dim.space_sm)
                .background(theme().color.element_bg)
                .border(1.0)
                .border_color(theme().color.element_border)
                .border_radius(theme().dim.radius_sm)
                .color(theme().color.text_primary)
                .outline(0.0)
                .min_width(0.0)
                .focus_visible(|s| s.border_color(theme().color.border_focus))
        }),
    ))
    .style(|s| s.flex_col().gap(theme().dim.space_xs))
}

fn duration_field(label_text: &'static str, value: String) -> impl IntoView {
    let v = value;
    Stack::vertical((
        field_label(label_text),
        Stack::horizontal((
            Label::derived(move || v.clone()).style(|s| {
                s.font_family(theme().font.mono_sm.family.clone())
                    .font_size(theme().font.body.size)
                    .color(theme().color.text_primary)
            }),
            Label::derived(|| "(derived from media)".to_string()).style(|s| {
                s.font_family(theme().font.mono_sm.family.clone())
                    .font_size(theme().font.mono_sm.size)
                    .color(theme().color.text_disabled)
            }),
        ))
        .style(|s| s.items_center().gap(theme().dim.space_sm)),
    ))
    .style(|s| s.flex_col().gap(theme().dim.space_xs))
}

fn trigger_selector(initial: TriggerCondition) -> impl IntoView {
    let mode = RwSignal::new(initial);
    Stack::vertical((
        field_label("Trigger Condition"),
        Stack::horizontal((
            trigger_btn("Playhead", TriggerCondition::Playhead, mode),
            trigger_btn(
                "With Cue",
                TriggerCondition::WithCue {
                    target: crate::model::CueId::new(),
                },
                mode,
            ),
            trigger_btn(
                "After Cue",
                TriggerCondition::AfterCue {
                    target: crate::model::CueId::new(),
                },
                mode,
            ),
        ))
        .style(|s| {
            s.border_radius(theme().dim.radius_sm)
                .border(1.0)
                .border_color(theme().color.element_border)
        }),
    ))
    .style(|s| s.flex_col().gap(theme().dim.space_sm))
}

fn trigger_btn(
    label_text: &'static str,
    this: TriggerCondition,
    mode: RwSignal<TriggerCondition>,
) -> impl IntoView {
    let tag = this.discriminant_tag();
    Button::new(label_text)
        .action(move || mode.set(this))
        .style(move |s| {
            let is_active = mode.get().discriminant_tag() == tag;
            s.height(theme().dim.control_sm)
                .padding_horiz(theme().dim.space_md)
                .padding_vert(theme().dim.space_sm)
                .font_family(theme().font.body.family.clone())
                .font_size(12.0)
                .font_weight(floem::text::FontWeight::MEDIUM)
                .outline(0.0)
                .background(if is_active {
                    theme().color.bg_selection
                } else {
                    theme().color.element_bg
                })
                .color(if is_active {
                    theme().color.text_primary
                } else {
                    theme().color.text_disabled
                })
                .hover(|s| {
                    s.background(if is_active {
                        theme().color.bg_selection
                    } else {
                        theme().color.element_bg_hover
                    })
                })
                .focus_visible(|s| s.border_color(theme().color.border_focus))
        })
}

fn color_picker(initial: CueColor) -> impl IntoView {
    let color = RwSignal::new(initial);
    let swatches: Vec<_> = COLOR_ORDER
        .iter()
        .map(|c| {
            let this = *c;
            let bg_color = match this {
                CueColor::None => theme().color.text_disabled_50,
                CueColor::Red => theme().color.status_error,
                CueColor::Orange => theme().color.status_group,
                CueColor::Green => theme().color.status_running,
                CueColor::Blue => theme().color.status_playhead,
                CueColor::Purple => theme().color.status_standby,
            };
            Container::new(Empty::new())
                .style(move |s| {
                    let is_active = color.get() == this;
                    s.size(theme().dim.space_xl, theme().dim.space_xl)
                        .border_radius(theme().dim.radius_full)
                        .background(bg_color)
                        .outline(0.0)
                        .apply_if(is_active, |s| {
                            s.border(2.0).border_color(theme().color.text_primary)
                        })
                })
                .on_event_stop(floem::event::listener::Click, move |_cx, _| {
                    color.set(this);
                })
                .into_any()
        })
        .collect();

    Stack::vertical((
        field_label("Highlight"),
        Stack::horizontal_from_iter(swatches).style(|s| s.items_center().gap(theme().dim.space_sm)),
    ))
    .style(|s| s.flex_col().gap(theme().dim.space_sm))
}

fn fmt_duration(d: std::time::Duration) -> String {
    let total_secs = d.as_secs();
    let m = total_secs / 60;
    let s = total_secs % 60;
    format!("{:02}:{:02}", m, s)
}

fn general_tab(tcs: &TransientCueState) -> impl IntoView {
    let cue = tcs.workspace.get();
    let name = cue.name.clone();
    let notes = cue.notes.clone();
    let pre_wait = fmt_duration(cue.pre_wait);
    let post_wait = fmt_duration(cue.post_wait);

    // Duration from audio telemetry
    let metrics = tcs.read_audio_telemetry();
    let duration_str = {
        let dur = metrics.total_duration_sec();
        if dur > 0.0 {
            fmt_duration(std::time::Duration::from_secs_f64(dur))
        } else {
            "00:00".to_string()
        }
    };

    Stack::horizontal((
        Stack::vertical((
            text_field("Name", name, false),
            Stack::vertical((
                field_label("Notes"),
                TextInput::new(RwSignal::new(notes)).style(move |s| {
                    s.height(theme().dim.textarea_height)
                        .border_radius(theme().dim.radius_sm)
                        .border(1.0)
                        .border_color(theme().color.element_border)
                        .background(theme().color.element_bg)
                        .padding_vert(6.0)
                        .padding_horiz(theme().dim.space_sm)
                        .font_family(theme().font.body.family.clone())
                        .font_size(theme().font.body.size)
                        .color(theme().color.text_primary)
                        .outline(0.0)
                        .min_width(0.0)
                        .focus_visible(|s| s.border_color(theme().color.border_focus))
                }),
            ))
            .style(|s| s.flex_col().gap(theme().dim.space_xs)),
        ))
        .style(|s| s.flex_col().gap(theme().dim.space_lg)),
        Stack::vertical((
            text_field("Pre-delay", pre_wait, true),
            text_field("Post-delay", post_wait, true),
            duration_field("Duration", duration_str),
        ))
        .style(|s| s.flex_col().gap(theme().dim.space_lg)),
        Stack::vertical((
            trigger_selector(cue.trigger_condition),
            color_picker(cue.color),
        ))
        .style(|s| s.flex_col().gap(theme().dim.space_lg)),
    ))
    .style(|s| s.gap(theme().dim.space_xl))
}

fn audio_tab(tcs: &TransientCueState) -> impl IntoView {
    let cue = tcs.workspace.get();
    let (file_path, volume, fade_in, fade_out) = match &cue.kind {
        CueKind::Audio {
            file_path,
            volume,
            fade_in_sec,
            fade_out_sec,
            ..
        } => (file_path.clone(), *volume, *fade_in_sec, *fade_out_sec),
        _ => (std::path::PathBuf::new(), 0.0, 0.0, 0.0),
    };

    let media_file = file_path.to_string_lossy().to_string();
    let vol_pct = RwSignal::new(volume);

    Stack::vertical((
        Stack::vertical((
            field_label("Media File"),
            Stack::horizontal((
                app_icon(
                    AppIcon::FileAudio,
                    theme().dim.icon_sm as f32,
                    theme().color.status_playhead,
                ),
                Label::derived(move || media_file.clone()).style(|s| {
                    s.font_family(theme().font.mono_sm.family.clone())
                        .font_size(theme().font.mono_sm.size)
                        .color(theme().color.text_primary)
                        .min_width(0.0)
                        .flex_grow(1.0)
                }),
                Button::new("Browse…").style(|s| {
                    s.height(theme().dim.control_sm)
                        .padding_horiz(theme().dim.space_sm)
                        .padding_vert(theme().dim.space_xs)
                        .font_family(theme().font.body.family.clone())
                        .font_size(theme().font.mono_sm.size)
                        .background(theme().color.element_bg_hover)
                        .border(1.0)
                        .border_color(theme().color.element_border)
                        .border_radius(theme().dim.radius_sm)
                        .color(theme().color.text_secondary)
                        .outline(0.0)
                        .hover(|s| s.background(theme().color.bg_surface_raised))
                        .focus_visible(|s| s.border_color(theme().color.border_focus))
                }),
            ))
            .style(|s| {
                s.items_center()
                    .gap(theme().dim.space_sm)
                    .height(theme().dim.control_sm)
                    .border_radius(theme().dim.radius_sm)
                    .border(1.0)
                    .border_color(theme().color.element_border)
                    .background(theme().color.element_bg)
                    .padding_horiz(theme().dim.space_sm)
            }),
        ))
        .style(|s| s.flex_col().gap(theme().dim.space_xs)),
        Stack::vertical((
            field_label("Target Volume"),
            Stack::horizontal((
                Label::derived(move || format!("{:.0}%", vol_pct.get() * 100.0)).style(|s| {
                    s.font_family(theme().font.mono_sm.family.clone())
                        .font_size(theme().font.mono_sm.size)
                        .color(theme().color.text_secondary)
                        .min_width(40.0)
                }),
                Label::new("").style(|s| s.flex_grow(1.0)),
            ))
            .style(|s| s.items_center().gap(theme().dim.space_sm)),
        ))
        .style(|s| s.flex_col().gap(theme().dim.space_sm)),
        Stack::horizontal((
            text_field("Fade In", format!("{:.1}s", fade_in), true),
            text_field("Fade Out", format!("{:.1}s", fade_out), true),
        ))
        .style(|s| s.gap(theme().dim.space_md)),
    ))
    .style(|s| s.flex_col().gap(theme().dim.space_lg))
}

fn osc_tab(tcs: &TransientCueState) -> impl IntoView {
    let cue = tcs.workspace.get();
    let (task, host, port) = match &cue.kind {
        CueKind::Osc { task, host, port } => (task.clone(), host.clone(), *port),
        _ => ("/projector/power 1".into(), "10.0.0.42".into(), 3333),
    };
    Stack::vertical((
        text_field("OSC Task", task, false),
        text_field("Host", host, false),
        text_field("Port", port.to_string(), true),
    ))
    .style(|s| s.flex_col().gap(theme().dim.space_lg))
}

fn empty_state() -> impl IntoView {
    Container::new(
        Label::derived(|| "Select a cue to edit its settings".to_string()).style(|s| {
            s.color(theme().color.text_disabled)
                .font_size(theme().font.body.size)
        }),
    )
    .style(|s| s.items_center().justify_center().size_full())
}

fn tab_button(
    label: &'static str,
    active: RwSignal<&'static str>,
    this: &'static str,
) -> impl IntoView {
    Button::new(label)
        .action(move || active.set(this))
        .style(move |s| {
            let is_active = active.get() == this;
            s.height(theme().dim.control_sm)
                .padding_horiz(theme().dim.space_md)
                .padding_vert(theme().dim.space_sm)
                .font_family(theme().font.body.family.clone())
                .font_size(12.0)
                .font_weight(floem::text::FontWeight::MEDIUM)
                .outline(0.0)
                .background(if is_active {
                    theme().color.bg_selection
                } else {
                    theme().color.element_bg
                })
                .color(if is_active {
                    theme().color.text_primary
                } else {
                    theme().color.text_disabled
                })
                .hover(|s| {
                    s.background(if is_active {
                        theme().color.bg_selection
                    } else {
                        theme().color.element_bg_hover
                    })
                    .color(theme().color.text_primary)
                })
                .focus_visible(|s| s.border_color(theme().color.border_focus))
        })
}

pub fn view(selected_transient: Memo<Option<TransientCueState>>) -> impl IntoView {
    let active_tab: RwSignal<&'static str> = RwSignal::new("general");

    let selected = selected_transient.get();

    let tabs: Vec<&'static str> = {
        let mut t = vec!["general"];
        if let Some(ref tcs) = selected {
            let cue = tcs.workspace.get();
            match cue.kind {
                CueKind::Audio { .. } => t.push("audio"),
                CueKind::Osc { .. } => t.push("osc"),
                _ => {}
            }
        }
        t
    };

    let tab_bar = Stack::horizontal_from_iter(
        tabs.iter()
            .map(|t| {
                let label_str = match *t {
                    "general" => "General",
                    "audio" => "Audio",
                    "osc" => "OSC",
                    _ => "General",
                };
                tab_button(label_str, active_tab, *t).into_any()
            })
            .collect::<Vec<_>>(),
    )
    .style(|s| {
        s.flex_shrink(0.0)
            .items_center()
            .gap(0.0)
            .background(theme().color.element_bg_hover)
    });

    let selected_cue_name = selected.as_ref().map(|tcs| {
        let cue = tcs.workspace.get();
        format!("{} · {}", cue.id, cue.name)
    });
    let header_info = Stack::horizontal((
        Label::derived(|| "Cue Settings".to_string()).style(|s| {
            s.font_family(theme().font.body.family.clone())
                .font_size(theme().font.body.size)
                .font_weight(floem::text::FontWeight::SEMI_BOLD)
                .color(theme().color.text_primary)
        }),
        Label::derived(move || selected_cue_name.clone().unwrap_or_default()).style(|s| {
            s.font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
                .color(theme().color.text_disabled)
        }),
    ))
    .style(|s| {
        s.items_center()
            .gap(theme().dim.space_md)
            .padding_horiz(theme().dim.space_md)
            .border_left(1.0)
            .border_color(theme().color.element_border)
    });

    let header = Stack::horizontal((tab_bar, header_info)).style(|s| {
        s.flex_shrink(0.0)
            .items_center()
            .border_bottom(1.0)
            .border_color(theme().color.element_border)
    });

    let content = match &selected {
        Some(tcs) => {
            let tab_content: floem::AnyView = match active_tab.get() {
                "general" => general_tab(tcs).into_any(),
                "audio" => audio_tab(tcs).into_any(),
                "osc" => osc_tab(tcs).into_any(),
                _ => general_tab(tcs).into_any(),
            };
            (tab_content)
                .style(|s| {
                    s.flex_grow(1.0)
                        .min_height(0.0)
                        .padding(theme().dim.space_lg)
                })
                .into_any()
        }
        None => empty_state().into_any(),
    };

    Stack::vertical((header, content)).style(|s| {
        s.flex_col()
            .height(theme().dim.detail_height)
            .flex_shrink(0.0)
            .background(theme().color.bg_surface)
            .width_full()
    })
}
