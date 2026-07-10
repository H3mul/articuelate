//! Bottom "Context-Dependent Detail Panel" (Lapce's collapsible bottom panel).
//!
//! Tabbed inspector driven by the currently selected cue. For the placeholder we
//! render the three docs/ui.md tabs - General, Audio Routing, Fades - using
//! flat Lapce-style sliders, number inputs, and a toggle-button routing matrix.
//! Collapsing is handled by the panel system (toolbar toggle / Ctrl+J).

use floem::IntoView;
use floem::reactive::{
    RwSignal, SignalGet, SignalUpdate, SignalWith, create_get_update, create_rw_signal,
    create_signal,
};
use floem::unit::Pct;
use floem::views::{
    Decorators, button, empty, h_stack, label, slider, tab, text, text_input, v_stack,
};

use crate::model::Cue;
use crate::theme::*;

const TABS: &[&str] = &["General", "Audio Routing", "Fades"];

#[allow(clippy::too_many_arguments)]
pub fn view(selected: RwSignal<usize>, cues: RwSignal<im::Vector<Cue>>) -> impl IntoView {
    let (active_tab, set_active_tab) = create_signal(0usize);
    let routing: RwSignal<[[bool; 4]; 2]> = create_rw_signal([[false; 4]; 2]);
    let duration = create_rw_signal(3.0_f64);
    let volume = create_rw_signal(-24.0_f64);

    let duration_pct = create_get_update(
        duration,
        |v: &f64| Pct(v / 10.0 * 100.0),
        |p: &Pct| p.0 / 100.0 * 10.0,
    );
    let volume_pct = create_get_update(
        volume,
        |v: &f64| Pct((v + 60.0) / 60.0 * 100.0),
        |p: &Pct| p.0 / 100.0 * 60.0 - 60.0,
    );

    let b0 = tab_button(0, "General", active_tab, set_active_tab);
    let b1 = tab_button(1, "Audio Routing", active_tab, set_active_tab);
    let b2 = tab_button(2, "Fades", active_tab, set_active_tab);
    let tab_buttons = h_stack((b0, b1, b2)).style(|s| {
        s.items_center()
            .gap(2.0)
            .background(theme().color.panel)
            .border_bottom(1.0)
            .border_color(theme().color.border)
    });

    let content = tab(
        move || active_tab.get(),
        move || TABS.iter().copied(),
        |t| *t,
        move |t| match t {
            "General" => general_tab(selected, cues).into_any(),
            "Audio Routing" => routing_tab(routing).into_any(),
            "Fades" => fades_tab(duration, volume, duration_pct, volume_pct).into_any(),
            _ => empty().into_any(),
        },
    )
    .style(|s| {
        s.flex_col()
            .flex_grow(1.0)
            .min_height(0.0)
            .padding(14.0)
            .background(theme().color.bg)
    });

    v_stack((tab_buttons, content)).style(|s| {
        s.flex_col()
            .height(240.0)
            .background(theme().color.bg)
            .border_top(1.0)
            .border_color(theme().color.border)
    })
}

fn tab_button(
    i: usize,
    name: &'static str,
    at: floem::reactive::ReadSignal<usize>,
    st: floem::reactive::WriteSignal<usize>,
) -> impl IntoView {
    button(name).action(move || st.set(i)).style(move |s| {
        s.padding_horiz(14.0)
            .padding_vert(8.0)
            .font_size(12.0)
            .color(if at.get() == i {
                theme().color.fg
            } else {
                theme().color.text_dim
            })
            .background(if at.get() == i {
                theme().color.panel_alt
            } else {
                theme().color.panel
            })
            .apply_if(at.get() == i, |s| {
                s.border_bottom(2.0).border_color(theme().color.accent)
            })
            .hover(|s| s.background(theme().color.border))
    })
}

/// Reactive accessor for the selected cue's name.
fn cue_name(
    selected: RwSignal<usize>,
    cues: RwSignal<im::Vector<Cue>>,
) -> impl Fn() -> String + 'static {
    move || {
        cues.with(|c| {
            c.get(selected.get())
                .map(|cue| cue.name.clone())
                .unwrap_or_else(|| "—".to_string())
        })
    }
}

fn general_tab(selected: RwSignal<usize>, cues: RwSignal<im::Vector<Cue>>) -> impl IntoView {
    let (s1, s2) = (selected, selected);
    let name_fn = cue_name(s1, cues);

    let ctx = label(move || format!("Task {} — {}", s2.get() + 1, name_fn())).style(|s| {
        s.color(theme().color.fg)
            .font_size(13.0)
            .font_weight(floem::text::Weight::BOLD)
    });

    let target = field("Target", "BGM");
    let property = field("Property", "Volume");

    let vol_in = text_input(create_rw_signal("-24".to_string())).style(input_style());
    let dur_in = text_input(create_rw_signal("3.0".to_string())).style(input_style());

    let grid = h_stack((
        v_stack((text("Target Vol:").style(field_label()), vol_in)).style(field_col()),
        v_stack((text("Duration:").style(field_label()), dur_in)).style(field_col()),
    ))
    .style(|s| s.gap(24.0).items_end());

    v_stack((
        ctx,
        h_stack((target, property)).style(|s| s.gap(24.0).margin_top(10.0)),
        grid,
    ))
    .style(|s| s.flex_col().gap(6.0))
}

fn routing_tab(routing: RwSignal<[[bool; 4]; 2]>) -> impl IntoView {
    let row_l = routing_row(0, routing);
    let row_r = routing_row(1, routing);
    let matrix = v_stack((row_l, row_r)).style(|s| s.flex_col().gap(8.0).margin_top(6.0));

    v_stack((
        text("Crosspoint Matrix (Input → Outputs)").style(field_label()),
        matrix,
    ))
    .style(|s| s.flex_col().gap(4.0))
}

fn routing_row(r: usize, routing: RwSignal<[[bool; 4]; 2]>) -> impl IntoView {
    let c0 = cell_button(r, 0, routing);
    let c1 = cell_button(r, 1, routing);
    let c2 = cell_button(r, 2, routing);
    let c3 = cell_button(r, 3, routing);
    let cell_row = h_stack((c0, c1, c2, c3)).style(|s| s.gap(0.0));
    h_stack((
        text(["In L", "In R"][r]).style(|s| {
            s.font_family(theme().font.mono.to_string())
                .color(theme().color.text_dim)
                .font_size(11.0)
                .width(44.0)
        }),
        cell_row,
    ))
    .style(|s| s.items_center().gap(6.0))
}

fn cell_button(r: usize, c: usize, routing: RwSignal<[[bool; 4]; 2]>) -> impl IntoView {
    let out_label = ["1", "2", "3", "4"][c];
    button(out_label)
        .action(move || routing.update(|m| m[r][c] = !m[r][c]))
        .style(move |s| {
            s.width(46.0)
                .height(34.0)
                .margin(3.0)
                .font_family(theme().font.mono.to_string())
                .font_size(12.0)
                .color(if routing.get()[r][c] {
                    theme().color.bg
                } else {
                    theme().color.text_dim
                })
                .background(if routing.get()[r][c] {
                    theme().color.accent
                } else {
                    theme().color.panel_alt
                })
                .border(1.0)
                .border_color(theme().color.border)
                .border_radius(4.0)
                .hover(|s| s.background(theme().color.border))
        })
}

#[allow(clippy::too_many_arguments)]
fn fades_tab(
    duration: RwSignal<f64>,
    volume: RwSignal<f64>,
    duration_pct: impl SignalGet<Pct> + SignalUpdate<Pct> + Copy + 'static,
    volume_pct: impl SignalGet<Pct> + SignalUpdate<Pct> + Copy + 'static,
) -> impl IntoView {
    let dur_row = h_stack((
        text("Fade Duration").style(field_label()),
        slider::Slider::new_rw(duration_pct).style(|s| s.width(240.0)),
        label(move || format!("{:.1} s", duration.get())).style(mono_value()),
    ))
    .style(row_style());

    let vol_row = h_stack((
        text("Target Volume").style(field_label()),
        slider::Slider::new_rw(volume_pct).style(|s| s.width(240.0)),
        label(move || format!("{:+.0} dB", volume.get())).style(mono_value()),
    ))
    .style(row_style());

    v_stack((dur_row, vol_row)).style(|s| s.flex_col().gap(14.0).margin_top(4.0))
}

// --- small style helpers --------------------------------------------------

fn field(label_text: &'static str, value: &'static str) -> impl IntoView {
    h_stack((
        text(label_text).style(field_label()),
        text(value).style(|s| {
            s.color(theme().color.fg)
                .font_family(theme().font.mono.to_string())
                .font_size(12.0)
        }),
    ))
    .style(|s| s.gap(8.0).items_center())
}

fn field_label() -> impl Fn(floem::style::Style) -> floem::style::Style + 'static {
    move |s: floem::style::Style| {
        s.color(theme().color.text_dim)
            .font_size(11.0)
            .width(110.0)
            .font_family(theme().font.mono.to_string())
    }
}

fn field_col() -> impl Fn(floem::style::Style) -> floem::style::Style + 'static {
    move |s: floem::style::Style| s.flex_col().gap(4.0)
}

fn input_style() -> impl Fn(floem::style::Style) -> floem::style::Style + 'static {
    move |s: floem::style::Style| {
        s.background(theme().color.panel_alt)
            .color(theme().color.fg)
            .border(1.0)
            .border_color(theme().color.border)
            .border_radius(4.0)
            .padding_horiz(8.0)
            .padding_vert(4.0)
            .width(90.0)
            .font_size(12.0)
            .font_family(theme().font.mono.to_string())
    }
}

fn mono_value() -> impl Fn(floem::style::Style) -> floem::style::Style + 'static {
    move |s: floem::style::Style| {
        s.color(theme().color.accent)
            .font_family(theme().font.mono.to_string())
            .font_size(12.0)
            .width(60.0)
    }
}

fn row_style() -> impl Fn(floem::style::Style) -> floem::style::Style + 'static {
    move |s: floem::style::Style| s.items_center().gap(12.0)
}
