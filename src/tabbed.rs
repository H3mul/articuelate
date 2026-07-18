//! A small, decoupled, Lapce-inspired *tabbed window*.
//!
//! Given a list of `(name, view-factory)` tuples it wraps them in a single,
//! isolated window: a window border, a tab switcher row that is visually
//! separate from the content, a separating border line, and finally the active
//! child view. The tab switcher can sit on the `Top` (default), `Left`,
//! `Right` or `Bottom` of the window.
//!
//! ```text
//! ┌────────────────────────────┐   <- window border
//! │ General │ Audio │ Fades    │   <- tab switcher row (separate bg)
//! ├────────────────────────────┤   <- separating border line
//! │                            │
//! │   active child view        │   <- content
//! │                            │
//! └────────────────────────────┘
//! ```
//!
//! The tab switcher buttons are simple Lapce-style buttons: the title rests on
//! the background colour and, on hover, gets a rounded-square fill in a lighter
//! colour. The active tab carries an accent-coloured indicator line on the
//! outer edge facing the window border.

use floem::peniko::Color;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate, create_rw_signal};
use floem::views::{
    Decorators, button, container, dyn_container, empty, h_stack_from_iter, v_stack_from_iter,
};
use floem::{AnyView, IntoView};

use crate::theme::*;

/// A single tab: a stable name paired with a factory that (re)builds its view
/// each time the tab becomes active.
type TabEntry = (&'static str, Box<dyn Fn() -> AnyView>);

/// Where the tab switcher sits relative to the child view.
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TabPosition {
    Top,
    Left,
    Right,
    Bottom,
}

/// Builder for a tabbed window. Mirrors the decoupled spirit of `panel.rs`:
/// register each tab with a name and a factory that produces its view, then
/// [`TabbedWindow::build`] assembles the whole thing into one `impl IntoView`.
pub struct TabbedWindow {
    position: TabPosition,
    tabs: Vec<TabEntry>,
}

impl Default for TabbedWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl TabbedWindow {
    /// Start a tabbed window. The switcher defaults to the top.
    pub fn new() -> Self {
        TabbedWindow {
            position: TabPosition::Top,
            tabs: Vec::new(),
        }
    }

    /// Place the tab switcher on the given side of the window.
    pub fn with_position(mut self, position: TabPosition) -> Self {
        self.position = position;
        self
    }

    /// Register one tab. `make` is called every time the tab becomes active, so
    /// it should rebuild the view from captured (Copy) signals rather than hold
    /// a single view instance.
    pub fn with_tab(mut self, name: &'static str, make: impl Fn() -> AnyView + 'static) -> Self {
        self.tabs.push((name, Box::new(make)));
        self
    }

    /// Assemble the tabbed window into a single view.
    pub fn build(self) -> impl IntoView {
        let position = self.position;
        let titles: Vec<&'static str> = self.tabs.iter().map(|(n, _)| *n).collect();
        let factories: Vec<Box<dyn Fn() -> AnyView>> =
            self.tabs.into_iter().map(|(_, f)| f).collect();
        let active = create_rw_signal(0usize);

        let switcher = build_switcher(&titles, active, position);

        let content = dyn_container(move || active.get(), move |i: usize| factories[i]())
            .style(|s| {
                let bw = theme().panel.border_width as f32;
                s.flex_grow(1.0)
                    .min_size(0.0, 0.0)
                    .size_full()
                    .border(bw)
                    .border_color(theme().color.border)
                    .padding(14.0)
                    .background(theme().color.bg)
            });

        let inner: AnyView = match position {
            TabPosition::Top => {
                v_stack_from_iter([switcher, content.into_any()])
                    .style(|s| s.flex_col().size_full().min_height(0.0))
                    .into_any()
            }
            TabPosition::Bottom => {
                v_stack_from_iter([content.into_any(), switcher])
                    .style(|s| s.flex_col().size_full().min_height(0.0))
                    .into_any()
            }
            TabPosition::Left => {
                h_stack_from_iter([switcher, content.into_any()])
                    .style(|s| s.flex_row().size_full().min_width(0.0))
                    .into_any()
            }
            TabPosition::Right => {
                h_stack_from_iter([content.into_any(), switcher])
                    .style(|s| s.flex_row().size_full().min_width(0.0))
                    .into_any()
            }
        };

        container(inner).style(|s| {
            s.background(theme().color.bg)
                .size_full()
                .min_size(0.0, 0.0)
        })
    }
}

/// Convenience wrapper: build a tabbed window directly from a list of
/// `(name, view-factory)` tuples.
#[allow(dead_code)]
pub fn view(tabs: Vec<TabEntry>, position: TabPosition) -> impl IntoView {
    let mut window = TabbedWindow::new().with_position(position);
    for (name, make) in tabs {
        window = window.with_tab(name, make);
    }
    window.build()
}

/// Build the tab switcher row for the given position.
fn build_switcher(
    titles: &[&'static str],
    active: RwSignal<usize>,
    position: TabPosition,
) -> AnyView {
    let buttons: Vec<AnyView> = titles
        .iter()
        .enumerate()
        .map(|(i, name)| tab_button(i, name, active, position))
        .collect();

    match position {
        TabPosition::Top | TabPosition::Bottom => h_stack_from_iter(buttons)
            .style(move |s| {
                // Flush against the outer window border; only a small inner
                // (content-side) padding so the tab row breathes.
                let s = match position {
                    TabPosition::Top => s.padding_top(0.0).padding_bottom(4.0),
                    _ => s.padding_bottom(0.0).padding_top(4.0),
                };
                s.items_center()
                    .gap(2.0)
                    .width_full()
                    .background(theme().color.panel)
                    .padding_horiz(8.0)
            })
            .into_any(),
        TabPosition::Left | TabPosition::Right => v_stack_from_iter(buttons)
            .style(move |s| {
                let s = match position {
                    TabPosition::Left => s.padding_left(0.0).padding_right(4.0),
                    _ => s.padding_right(0.0).padding_left(4.0),
                };
                s.items_center()
                    .gap(2.0)
                    .height_full()
                    .background(theme().color.panel)
                    .padding_vert(8.0)
            })
            .into_any(),
    }
}

/// One Lapce-style tab button, with its active-state accent indicator placed on
/// the outer edge facing the window border.
fn tab_button(
    i: usize,
    name: &'static str,
    active: RwSignal<usize>,
    position: TabPosition,
) -> AnyView {
    let indicator = tab_indicator(i, active, position);

    let btn = button(name).action(move || active.set(i)).style(move |s| {
        s.padding_horiz(14.0)
            .padding_vert(6.0)
            .font_size(12.0)
            .border(0.0)
            .color(if active.get() == i {
                theme().color.fg
            } else {
                theme().color.text_dim
            })
            .background(theme().color.panel)
            .border_radius(3.0)
            .hover(|s| s.background(theme().color.panel_alt))
    });

    match position {
        TabPosition::Top => v_stack_from_iter([indicator.into_any(), btn.into_any()]).into_any(),
        TabPosition::Bottom => v_stack_from_iter([btn.into_any(), indicator.into_any()]).into_any(),
        TabPosition::Left => h_stack_from_iter([indicator.into_any(), btn.into_any()]).into_any(),
        TabPosition::Right => h_stack_from_iter([btn.into_any(), indicator.into_any()]).into_any(),
    }
}

/// The thin accent line shown on the active tab, sitting on the outer edge of
/// the switcher (above for `Top`, below for `Bottom`, left for `Left`, right
/// for `Right`). Transparent when the tab is inactive.
fn tab_indicator(i: usize, active: RwSignal<usize>, position: TabPosition) -> impl IntoView {
    empty().style(move |s| {
        let line = if active.get() == i {
            theme().color.accent
        } else {
            Color::TRANSPARENT
        };
        let bw = theme().panel.border_width as f32;
        match position {
            TabPosition::Top | TabPosition::Bottom => s
                .height(bw.max(2.0))
                .width_full()
                .background(line)
                .border_radius(1.0),
            TabPosition::Left | TabPosition::Right => s
                .width(bw.max(2.0))
                .height_full()
                .background(line)
                .border_radius(1.0),
        }
    })
}
