//! Right sidebar: "Currently Playing Media" live telemetry monitor.
//!
//! This is where Floem's fine-grained reactivity shines. A 50fps timer random-
//! walks one `RwSignal<f64>` per audio channel and pushes it through a derived
//! `Pct` signal. Only the bound meter bars repaint - the rest of the UI sleeps,
//! which is exactly what an audio app needs for cheap, smooth metering.

use floem::IntoView;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate, create_get_update, create_rw_signal};
use floem::unit::{Pct, UnitExt};
use floem::views::{Decorators, button, h_stack, label, list, slider, text, v_stack};
use lucide_floem::Icon;

use crate::model::sample_active_media;
use crate::panel::PanelFlags;
use crate::theme::*;

pub fn view(visible: RwSignal<PanelFlags>) -> impl IntoView {
    let names = sample_active_media();
    let n = names.len();

    // One independent level signal per channel -> surgical updates.
    let levels: Vec<RwSignal<f64>> = (0..n).map(|_| create_rw_signal(0.0)).collect();

    // Telemetry pump: smooth random walk at ~20fps.
    // let levels_timer = levels.clone();
    // let tick = create_rw_signal(());
    // create_effect(move |_| {
    //     tick.track();
    //     let lt = levels_timer.clone();
    //     exec_after(Duration::from_millis(50), move |_| {
    //         for l in &lt {
    //             l.update(|x| {
    //                 let target = fastrand::f64();
    //                 *x = (*x + (target - *x) * 0.35).clamp(0.0, 1.0);
    //             });
    //         }
    //         tick.set(());
    //     });
    // });

    let mut channels = Vec::new();
    for idx in 0..n {
        let name = names[idx].clone();
        let lvl = levels[idx];
        let pct = create_get_update(lvl, |v: &f64| Pct(*v), |p: &Pct| p.0);
        channels.push(channel_view(idx, name, pct));
    }

    let collapse = button(
        Icon::X
            .into_view()
            .style(|s| s.size(16.0, 16.0).color(theme().color.status_active)),
    )
    .action(move || visible.update(|v| v.right = false))
    .style(|s| {
        s.background(theme().color.bg_overlay)
            .color(theme().color.status_active)
            .border_radius(4.0)
            .padding_horiz(8.0)
            .padding_vert(4.0)
            .font_size(12.0)
            .hover(|s| s.background(theme().color.border_subtle))
    });

    let header = h_stack((
        text("ACTIVE MEDIA").style(|s| {
            s.font_family(theme().font.mono_sm.family)
                .font_size(11.0)
                .color(theme().color.text_secondary)
                .flex_grow(1.0)
        }),
        collapse,
    ))
    .style(|s| {
        s.items_center()
            .gap(8.0)
            .padding_horiz(12.0)
            .padding_vert(8.0)
            .border_bottom(1.0)
            .border_color(theme().color.border_subtle)
    });

    let body = list(channels).style(|s| s.flex_col().gap(14.0).padding(12.0).flex_grow(1.0));

    v_stack((header, body)).style(|s| {
        s.flex_col()
            .width(260.0)
            .min_width(200.0)
            .background(theme().color.bg_surface)
            .height_full()
    })
}

fn channel_view(
    idx: usize,
    name: std::sync::Arc<str>,
    pct: impl SignalGet<Pct> + Copy + 'static,
) -> impl IntoView {
    let _ = idx;
    let db = label(move || format!("{:+.1} dB", db_from_pct(pct.get().0))).style(|s| {
        s.font_family(theme().font.mono_sm.family)
            .color(theme().color.status_status_running)
            .font_size(11.0)
    });

    let meter = slider::Slider::new(move || pct.get())
        .disabled(|| true)
        .slider_style(|s| {
            s.handle_radius(0)
                .bar_radius(25.pct())
                .accent_bar_radius(25.pct())
        })
        .style(|s| s.width_full().height(14.0));

    v_stack((
        h_stack((
            label(move || name.to_string()).style(|s| {
                s.color(theme().color.text_primary)
                    .font_size(12.0)
                    .flex_grow(1.0)
            }),
            db,
        ))
        .style(|s| s.items_center().gap(8.0)),
        meter,
    ))
    .style(|s| s.flex_col().gap(4.0))
}

fn db_from_pct(p: f64) -> f64 {
    20.0 * p.max(1e-4).log10()
}
