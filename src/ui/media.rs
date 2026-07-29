//! Runtime sidebar — active cues panel with global controls.

use floem::IntoView;
use floem::reactive::{Memo, SignalGet, SignalWith, create_rw_signal};

use floem::views::{
    Decorators, button, container, empty, h_stack, label, scroll, slider, v_stack,
    v_stack_from_iter,
};

use crate::model::{CueColor, TransientCueState};
use crate::style::theme;
use crate::ui::icons::{AppIcon, app_icon};

fn audio_meter(level_l: f64, level_r: f64) -> impl IntoView {
    h_stack((vertical_led_meter(level_l), vertical_led_meter(level_r)))
        .style(|s| s.gap(theme().dim.space_xs))
}

fn vertical_led_meter(level: f64) -> impl IntoView {
    let count = 10;
    let lit_count = (level.min(1.0).max(0.0) * count as f64).round() as usize;
    let mut dots = Vec::new();
    for i in 0..count {
        let is_lit = i < lit_count;
        let color = if i >= (count as f64 * 0.83) as usize {
            theme().color.status_error
        } else if i >= (count as f64 * 0.66) as usize {
            theme().color.status_wait
        } else {
            theme().color.status_running
        };
        dots.insert(
            0,
            container(empty())
                .style(move |s| {
                    s.size(theme().dim.led_dot, theme().dim.led_dot)
                        .border_radius(theme().dim.radius_full)
                        .background(theme().color.element_bg)
                        .border(1.0)
                        .border_color(theme().color.element_border_25)
                        .apply_if(is_lit, |s| s.background(color))
                })
                .into_any(),
        );
    }
    v_stack_from_iter(dots).style(|s| s.gap(2.0))
}

fn fmt(seconds: f64) -> String {
    let m = (seconds / 60.0).floor() as u32;
    let s = seconds as u32 % 60;
    format!("{:02}:{:02}", m, s)
}

fn active_cue_row(tcs: &TransientCueState) -> impl IntoView {
    let cue = tcs.workspace.get();
    let metrics = tcs.read_audio_telemetry();

    let stripe_color = match cue.color {
        CueColor::None => theme().color.status_playhead,
        CueColor::Red => theme().color.status_error,
        CueColor::Orange => theme().color.status_group,
        CueColor::Green => theme().color.status_running,
        CueColor::Blue => theme().color.status_playhead,
        CueColor::Purple => theme().color.status_standby,
    };
    let name = cue.name.clone();
    let file = cue
        .kind
        .media_file()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "—".to_string());
    let duration = fmt(metrics.total_duration_sec());
    let remaining = fmt(metrics.total_duration_sec() - metrics.current_time_sec());
    let levels = (metrics.left_peak() as f64, metrics.right_peak() as f64);

    let top_row = h_stack((
        h_stack((label(move || name.clone()).style(|s| {
            s.font_family(theme().font.body.family.clone())
                .font_size(12.0)
                .font_weight(floem::text::Weight::MEDIUM)
                .color(theme().color.text_primary)
                .min_width(0.0)
        }),))
        .style(|s| {
            s.min_width(0.0)
                .items_center()
                .gap(theme().dim.space_xs)
                .flex_grow(1.0)
        }),
        h_stack((
            container(app_icon(
                AppIcon::SkipBack,
                theme().dim.icon_sm as f32,
                theme().color.text_secondary,
            ))
            .style(|s| {
                s.size(theme().dim.space_xl, theme().dim.space_xl)
                    .items_center()
                    .justify_center()
                    .border_radius(theme().dim.radius_sm)
                    .border(1.0)
                    .border_color(theme().color.element_border)
                    .background(theme().color.element_bg)
            }),
            container(app_icon(
                AppIcon::Pause,
                theme().dim.icon_sm as f32,
                theme().color.text_secondary,
            ))
            .style(|s| {
                s.size(theme().dim.space_xl, theme().dim.space_xl)
                    .items_center()
                    .justify_center()
                    .border_radius(theme().dim.radius_sm)
                    .border(1.0)
                    .border_color(theme().color.element_border)
                    .background(theme().color.element_bg)
            }),
            container(app_icon(
                AppIcon::Spline,
                theme().dim.icon_sm as f32,
                theme().color.text_secondary,
            ))
            .style(|s| {
                s.size(theme().dim.space_xl, theme().dim.space_xl)
                    .items_center()
                    .justify_center()
                    .border_radius(theme().dim.radius_sm)
                    .border(1.0)
                    .border_color(theme().color.element_border)
                    .background(theme().color.element_bg)
            }),
            container(app_icon(
                AppIcon::Stop,
                theme().dim.icon_sm as f32,
                theme().color.status_error,
            ))
            .style(|s| {
                s.size(theme().dim.space_xl, theme().dim.space_xl)
                    .items_center()
                    .justify_center()
                    .border_radius(theme().dim.radius_sm)
                    .border(1.0)
                    .border_color(theme().color.element_border)
                    .background(theme().color.element_bg)
            }),
        ))
        .style(|s| s.flex_shrink(0.0).items_center().gap(theme().dim.space_xs)),
    ))
    .style(|s| {
        s.flex_shrink(0.0)
            .items_center()
            .justify_between()
            .gap(theme().dim.space_xs)
    });

    let bottom_row = h_stack((
        h_stack((
            label(move || file.clone()).style(|s| {
                s.min_width(0.0)
                    .flex_grow(1.0)
                    .font_family(theme().font.mono_sm.family.clone())
                    .font_size(theme().font.mono_sm.size)
                    .color(theme().color.text_disabled)
            }),
            label(move || format!("{} -{}", duration, remaining)).style(|s| {
                s.flex_shrink(0.0)
                    .font_family(theme().font.mono_sm.family.clone())
                    .font_size(theme().font.mono_sm.size)
                    .color(theme().color.text_disabled)
            }),
        ))
        .style(|s| {
            s.min_width(0.0)
                .flex_grow(1.0)
                .items_center()
                .gap(theme().dim.space_xs)
        }),
        audio_meter(levels.0, levels.1),
    ))
    .style(|s| s.flex_row().items_center().gap(6.0).width_full());

    v_stack((top_row, bottom_row)).style(move |s| {
        s.flex_col()
            .gap(6.0)
            .height(theme().dim.active_card_height)
            .border_radius(theme().dim.radius_sm)
            .border(1.0)
            .border_color(theme().color.element_border)
            .background(theme().color.element_bg_hover)
            .padding_horiz(theme().dim.space_sm)
            .padding_top(theme().dim.space_xs)
            .border_left(3.0)
    })
}

pub fn view(running_transients: Memo<Vec<TransientCueState>>) -> impl IntoView {
    let gain = create_rw_signal(0.88);

    let db_str = move || {
        let v = gain.get();
        if v == 0.0 {
            "-∞".to_string()
        } else {
            format!("{:.1}", (v * 24.0 - 24.0))
        }
    };

    let running_count = running_transients.with(|v| v.len());

    let header = h_stack((
        label(|| "Active Cues".to_string()).style(|s| {
            s.font_family(theme().font.body.family.clone())
                .font_size(theme().font.body.size)
                .font_weight(floem::text::Weight::SEMIBOLD)
                .color(theme().color.text_primary)
                .min_width(0.0)
        }),
        // Consume only the space available inside the sidebar so the badge
        // remains aligned to the parent's right edge.
        container(empty()).style(|s| s.flex_grow(1.0).min_width(0.0)),
        label(move || format!("{} running", running_count)).style(|s| {
            s.flex_shrink(0.0)
                .border_radius(theme().dim.radius_sm)
                .padding(theme().dim.space_sm)
                .font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
                .font_weight(floem::text::Weight::SEMIBOLD)
                .background(theme().color.status_running_bg)
                .color(theme().color.status_running)
        }),
    ))
    .style(|s| {
        s.items_center()
            .width_full()
            .min_width(0.0)
            .flex_shrink(1.0)
            .gap(theme().dim.space_sm)
            .padding_horiz(theme().dim.space_md)
            .padding_vert(theme().dim.space_sm)
            .border_bottom(1.0)
            .border_color(theme().color.element_border)
    });

    let btn_style = |s: floem::style::Style| {
        s.size(theme().dim.space_xl, theme().dim.space_xl)
            .items_center()
            .justify_center()
            .border_radius(theme().dim.radius_md)
            .border(1.0)
            .border_color(theme().color.element_border)
            .background(theme().color.element_bg)
    };

    let global_controls = v_stack((
        h_stack((
            button(app_icon(
                AppIcon::SkipBack,
                theme().dim.icon_md as f32,
                theme().color.text_secondary,
            ))
            .style(btn_style),
            button(app_icon(
                AppIcon::Pause,
                theme().dim.icon_md as f32,
                theme().color.text_secondary,
            ))
            .style(btn_style),
            button(app_icon(
                AppIcon::Spline,
                theme().dim.icon_md as f32,
                theme().color.text_secondary,
            ))
            .style(btn_style),
            button(app_icon(
                AppIcon::Stop,
                theme().dim.icon_md as f32,
                theme().color.status_error,
            ))
            .style(btn_style),
        ))
        .style(|s| s.items_center().gap(theme().dim.space_xs)),
        h_stack((
            label(|| "MASTER".to_string()).style(|s| {
                s.flex_shrink(0.0)
                    .font_family(theme().font.mono_sm.family.clone())
                    .font_size(theme().font.mono_sm.size)
                    .color(theme().color.text_disabled)
            }),
            slider::Slider::new(move || gain.get()).style(|s| s.flex_grow(1.0).height(6.0)),
            label(move || format!("{} dB", db_str())).style(|s| {
                s.width(48.0)
                    .flex_shrink(0.0)
                    .font_family(theme().font.mono_sm.family.clone())
                    .font_size(theme().font.mono_sm.size)
                    .color(theme().color.text_secondary)
            }),
        ))
        .style(|s| s.items_center().gap(theme().dim.space_sm)),
        h_stack((
            container(empty()).style(move |s| {
                s.size(theme().dim.dot_sm, theme().dim.dot_sm)
                    .border_radius(theme().dim.radius_full)
                    .background(theme().color.status_running)
            }),
            label(|| "Audio Interface 1".to_string()).style(|s| {
                s.font_family(theme().font.mono_sm.family.clone())
                    .font_size(theme().font.mono_sm.size)
                    .color(theme().color.text_disabled)
            }),
            label(|| "Operational".to_string()).style(|s| {
                s.font_family(theme().font.mono_sm.family.clone())
                    .font_size(theme().font.mono_sm.size)
                    .color(theme().color.text_disabled)
            }),
        ))
        .style(|s| {
            s.items_center()
                .width_full()
                .gap(8.0)
                .border_radius(theme().font.mono_sm.line_height)
                .border(1.0)
                .border_color(theme().color.element_border)
                .background(theme().color.element_bg)
                .padding_vert(4.0)
                .padding_horiz(theme().dim.space_md)
        }),
    ))
    .style(|s| {
        s.flex_grow(1.0)
            .min_width(0.0)
            .flex_col()
            .gap(theme().dim.space_sm)
            .padding_vert(theme().dim.space_sm)
    });

    let controls_with_meter = h_stack((
        global_controls,
        audio_meter(0.9, 0.8).style(|s| s.padding_vert(theme().dim.space_sm)),
    ))
    .style(|s| {
        s.flex_row()
            .width_full()
            .items_end()
            .gap(theme().dim.space_sm)
            .padding_horiz(theme().dim.space_md)
            .border_bottom(1.0)
            .border_color(theme().color.element_border)
    });

    let cue_rows = running_transients
        .get()
        .iter()
        .map(|tcs| active_cue_row(tcs).into_any())
        .collect::<Vec<_>>();

    let cue_list =
        scroll(v_stack_from_iter(cue_rows).style(|s| s.flex_col().gap(theme().dim.space_sm)))
            .style(|s| {
                s.flex_grow(1.0)
                    .min_height(0.0)
                    .padding(theme().dim.space_sm)
            });

    v_stack((header, controls_with_meter, cue_list)).style(|s| {
        s.flex_col()
            .width_full()
            .flex_shrink(0.0)
            .background(theme().color.bg_surface)
            .height_full()
            .min_height(0.0)
    })
}
