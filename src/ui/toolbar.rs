//! Compact transport toolbar for the cuelist panel.

use floem::IntoView;
use floem::menu::{Menu, MenuItem};
use floem::peniko::Color;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate, SignalWith, create_rw_signal};
use floem::views::{Decorators, button, h_stack, text};
use lucide_floem::Icon;
use std::sync::Arc;

use crate::exec::ExecutionHandle;

use crate::exec::UiEvent;
use crate::model::{CueId, Cuelist};
use crate::style::*;

pub fn view(
    cuelist: impl SignalGet<Arc<Cuelist>> + SignalWith<Arc<Cuelist>> + Copy + 'static,
    active_cue: RwSignal<Option<CueId>>,
    selected: RwSignal<Option<CueId>>,
    events: ExecutionHandle,
    devices: Vec<String>,
    selected_device: RwSignal<Option<String>>,
) -> impl IntoView {
    let paused = create_rw_signal(false);

    let menu_events = events.clone();
    let devices_button = button(text("Devices"))
        .context_menu(move || device_menu(&devices, selected_device, menu_events.clone()));

    let pause = button(icon(
        Icon::Pause,
        theme().color.status_wait,
        theme().dim.status_icon_size as f32,
    ))
    .action(move || paused.update(|value| *value = !*value));
    let back = button(icon(
        Icon::SkipBack,
        theme().color.text_primary,
        theme().dim.status_icon_size as f32,
    ))
    .action(move || {
        if let Some(active) = active_cue.get() {
            let list = cuelist.get();
            if let Some(position) = list.iter().position(|cue| cue.id == active) {
                if position > 0 {
                    if let Some(previous) = list.iter().nth(position - 1) {
                        active_cue.set(Some(previous.id));
                        selected.set(Some(previous.id));
                    }
                }
            }
        }
    });
    let go = button(h_stack((
        icon(
            Icon::Play,
            theme().color.status_running,
            theme().dim.status_icon_size as f32,
        ),
        text("GO").style(|s| {
            s.color(theme().color.text_primary)
                .font_weight(floem::text::Weight::BOLD)
        }),
    )))
    .action(move || {
        let _ = events.send_user_intent(UiEvent::Go);
    });
    let panic = button(h_stack((
        icon(
            Icon::Ban,
            theme().color.status_error,
            theme().dim.status_icon_size as f32,
        ),
        text("Panic").style(|s| s.color(theme().color.status_error)),
    )))
    .action(move || active_cue.set(None));

    h_stack((
        devices_button,
        go,
        pause,
        back,
        text("").style(|s| s.flex_grow(1.0)),
        panic,
    ))
    .style(|s| {
        s.items_center()
            .width_full()
            .gap(theme().dim.space_sm)
            .padding_horiz(theme().dim.space_md)
            .background(theme().color.bg_surface)
            .border_bottom(theme().dim.border_size)
            .border_color(theme().color.border_divider)
            .height(theme().dim.toolbar_height)
    })
}

fn device_menu(
    devices: &[String],
    selected: RwSignal<Option<String>>,
    events: ExecutionHandle,
) -> Menu {
    devices.iter().fold(Menu::new("Devices"), |menu, device| {
        let is_selected = selected.get().as_deref() == Some(device.as_str());
        let title = format!("{}{}", if is_selected { "● " } else { "○ " }, device);
        let device_name = device.clone();
        let selected = selected;
        let events = events.clone();
        menu.entry(MenuItem::new(title).action(move || {
            selected.set(Some(device_name.clone()));
            let _ = events.send_user_intent(UiEvent::SetAudioDevice(device_name.clone()));
        }))
    })
}

fn icon(icon: Icon, color: Color, size: f32) -> impl IntoView {
    icon.into_view()
        .style(move |s| s.size(size, size).color(color))
}
