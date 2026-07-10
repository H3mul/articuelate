//! A small, decoupled, Lapce-inspired resizable panel system.
//!
//! A "window" is just any `impl IntoView`. You register each window with a
//! [`PanelLocation`] (`Main`, `Left`, `Right`, `Bottom`) and the system lays
//! them out and wires drag-to-resize handles automatically. The host only has
//! to implement the window UI; the panel system owns all reflow + resizing.
//!
//! Layout (Lapce style): left / right columns flank a centre column that stacks
//! the main window above the bottom panel.
//!
//! ```text
//! ┌──────┬───────────────────┬──────────┐
//! │ Left │       Main        │  Right   │
//! │      ├───────────────────┤          │
//! │      │      Bottom       │          │
//! └──────┴───────────────────┴──────────┘
//! ```

use floem::event::{Event, EventListener};
use floem::kurbo::{Point, Size};
use floem::reactive::{RwSignal, SignalGet, SignalUpdate, SignalWith, create_rw_signal};
use floem::style::CursorStyle;
use floem::views::{Decorators, container, empty, h_stack, stack, v_stack};
use floem::{AnyView, IntoView, View};

use crate::theme::*;

/// Where a registered window lives in the workspace.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PanelLocation {
    Main,
    Left,
    Right,
    Bottom,
}

/// Pixel sizes of the resizable panels.
#[derive(Clone, Copy, Default)]
pub struct PanelSizes {
    pub left: f64,
    pub right: f64,
    pub bottom: f64,
}

/// Visibility of the optional panels (Main is always shown).
#[derive(Clone, Copy, Default)]
pub struct PanelVisible {
    pub left: bool,
    pub right: bool,
    pub bottom: bool,
}

/// Builder / owner of the panel layout.
pub struct PanelSystem {
    sizes: RwSignal<PanelSizes>,
    visible: RwSignal<PanelVisible>,
    available: RwSignal<Size>,
    main: Option<AnyView>,
    left: Option<AnyView>,
    right: Option<AnyView>,
    bottom: Option<AnyView>,
}

impl PanelSystem {
    pub fn new() -> Self {
        PanelSystem {
            sizes: create_rw_signal(PanelSizes {
                left: 220.0,
                right: 260.0,
                bottom: 240.0,
            }),
            visible: create_rw_signal(PanelVisible {
                left: true,
                right: true,
                bottom: true,
            }),
            available: create_rw_signal(Size::ZERO),
            main: None,
            left: None,
            right: None,
            bottom: None,
        }
    }

    /// Register the required centre window.
    pub fn with_main(mut self, view: impl IntoView + 'static) -> Self {
        self.main = Some(view.into_any());
        self
    }

    pub fn with_left(mut self, view: impl IntoView + 'static) -> Self {
        self.left = Some(view.into_any());
        self
    }

    pub fn with_right(mut self, view: impl IntoView + 'static) -> Self {
        self.right = Some(view.into_any());
        self
    }

    pub fn with_bottom(mut self, view: impl IntoView + 'static) -> Self {
        self.bottom = Some(view.into_any());
        self
    }

    /// Shared visibility signal so a toolbar can toggle panels.
    pub fn visibility(&self) -> RwSignal<PanelVisible> {
        self.visible
    }

    /// Assemble the full workspace view: toolbar on top, panels in the middle,
    /// status bar at the bottom.
    pub fn build(
        self,
        toolbar: impl IntoView + 'static,
        status_bar: impl IntoView + 'static,
    ) -> impl IntoView {
        let sizes = self.sizes;
        let visible = self.visible;
        let available = self.available;

        let main = self
            .main
            .expect("PanelSystem::build requires a Main window");

        let left_view = self.left.map_or_else(
            || empty().into_any(),
            |v| panel_container(PanelLocation::Left, v, sizes, visible, available).into_any(),
        );
        let right_view = self.right.map_or_else(
            || empty().into_any(),
            |v| panel_container(PanelLocation::Right, v, sizes, visible, available).into_any(),
        );
        let bottom_view = self.bottom.map_or_else(
            || empty().into_any(),
            |v| panel_container(PanelLocation::Bottom, v, sizes, visible, available).into_any(),
        );

        let center_col = v_stack((main, bottom_view))
            .style(|s| s.flex_col().flex_grow(1.0).min_width(0.0).min_height(0.0));

        let center_row = h_stack((left_view, center_col, right_view))
            .style(|s| s.flex_row().flex_grow(1.0).min_height(0.0))
            .on_resize(move |rect| available.set(rect.size()));

        v_stack((toolbar.into_any(), center_row, status_bar.into_any())).style(|s| {
            s.flex_col()
                .width_full()
                .height_full()
                .background(theme().color.bg)
        })
    }
}

/// Build a single panel container (with optional drag handle + collapse).
fn panel_container(
    location: PanelLocation,
    content: impl IntoView + 'static,
    sizes: RwSignal<PanelSizes>,
    visible: RwSignal<PanelVisible>,
    available: RwSignal<Size>,
) -> impl View {
    let current_size = create_rw_signal(Size::ZERO);
    let handle = resize_handle(location, sizes, current_size, available);

    let content = container(content.into_view()).style(|s| s.size_pct(100.0, 100.0).flex_col());

    let inner: AnyView = match location {
        PanelLocation::Left => stack((content, handle))
            .style(|s| s.flex_row().height_pct(100.0))
            .into_any(),
        PanelLocation::Right => stack((handle, content))
            .style(|s| s.flex_row().height_pct(100.0))
            .into_any(),
        PanelLocation::Bottom => stack((handle, content))
            .style(|s| s.flex_col().width_pct(100.0))
            .into_any(),
        PanelLocation::Main => content.style(|s| s.size_pct(100.0, 100.0)).into_any(),
    };

    let is_shown = move || match location {
        PanelLocation::Left => visible.with(|v| v.left),
        PanelLocation::Right => visible.with(|v| v.right),
        PanelLocation::Bottom => visible.with(|v| v.bottom),
        PanelLocation::Main => true,
    };

    inner
        .style(move |s| {
            let s = match location {
                PanelLocation::Left => s
                    .width(sizes.with(|x| x.left as f32))
                    .height_pct(100.0)
                    .border_right(1.0)
                    .border_color(theme().color.border)
                    .background(theme().color.panel),
                PanelLocation::Right => s
                    .width(sizes.with(|x| x.right as f32))
                    .height_pct(100.0)
                    .border_left(1.0)
                    .border_color(theme().color.border)
                    .background(theme().color.panel),
                PanelLocation::Bottom => s
                    .height(sizes.with(|x| x.bottom as f32))
                    .width_pct(100.0)
                    .border_top(1.0)
                    .border_color(theme().color.border)
                    .background(theme().color.panel),
                PanelLocation::Main => s.size_pct(100.0, 100.0),
            };
            s.apply_if(location != PanelLocation::Main && !is_shown(), |s| s.hide())
        })
        .on_resize(move |rect| {
            let size = rect.size();
            if size != current_size.get_untracked() {
                current_size.set(size);
            }
        })
}

/// A thin, in-flow drag handle that resizes its owning panel.
fn resize_handle(
    location: PanelLocation,
    sizes: RwSignal<PanelSizes>,
    current_size: RwSignal<Size>,
    available: RwSignal<Size>,
) -> impl View {
    // (start pointer, panel size captured at drag start)
    let drag = create_rw_signal(None::<(Point, Size)>);

    let view = empty();
    let vid = view.id();
    view.on_event_stop(EventListener::PointerDown, move |event| {
        if let Event::PointerDown(p) = event {
            vid.request_active();
            drag.set(Some((p.pos, current_size.get())));
        }
    })
    .on_event_stop(EventListener::PointerMove, move |event| {
        if let (Event::PointerMove(p), Some((start_pos, start_size))) = (event, drag.get()) {
            let avail = available.get();
            let new = match location {
                PanelLocation::Left => (start_size.width + (p.pos.x - start_pos.x))
                    .clamp(140.0, (avail.width - 140.0).max(140.0)),
                PanelLocation::Right => (start_size.width - (p.pos.x - start_pos.x))
                    .clamp(140.0, (avail.width - 140.0).max(140.0)),
                PanelLocation::Bottom => (start_size.height - (p.pos.y - start_pos.y))
                    .clamp(100.0, (avail.height - 100.0).max(100.0)),
                PanelLocation::Main => start_size.width,
            };
            sizes.update(|s| match location {
                PanelLocation::Left => s.left = new,
                PanelLocation::Right => s.right = new,
                PanelLocation::Bottom => s.bottom = new,
                PanelLocation::Main => {}
            });
        }
    })
    .on_event_stop(EventListener::PointerUp, move |_| {
        drag.set(None);
    })
    .style(move |s| {
        let dragging = drag.get().is_some();
        let is_bottom = location == PanelLocation::Bottom;
        s.apply_if(is_bottom, |s| s.width_pct(100.0).height(4.0))
            .apply_if(!is_bottom, |s| s.width(4.0).height_pct(100.0))
            .apply_if(dragging, |s| {
                s.background(theme().color.accent)
                    .apply_if(is_bottom, |s| s.cursor(CursorStyle::RowResize))
                    .apply_if(!is_bottom, |s| s.cursor(CursorStyle::ColResize))
                    .z_index(5)
            })
            .hover(|s| {
                s.background(theme().color.accent)
                    .apply_if(is_bottom, |s| s.cursor(CursorStyle::RowResize))
                    .apply_if(!is_bottom, |s| s.cursor(CursorStyle::ColResize))
                    .z_index(5)
            })
    })
}
