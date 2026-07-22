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
//! ┌───────┬───────────────────────┬───────────┐
//! │ Left │       Main        │  Right  │
//! │      ├───────────────────────┤         │
//! │      │      Bottom       │         │
//! └───────┴───────────────────────┴───────────┘
//! ```

use floem::event::{Event, EventListener};
use floem::kurbo::{Point, Size};
use floem::reactive::{RwSignal, SignalGet, SignalUpdate, SignalWith, create_rw_signal};
use floem::style::{AlignItems, CursorStyle};
use floem::views::{Decorators, container, empty, h_stack, scroll, v_stack};
use floem::{AnyView, IntoView, View};

use crate::theme::*;

/// Where a registered window lives in the workspace.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PanelLocation {
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
#[derive(Debug, Clone, Copy, Default)]
pub struct PanelFlags {
    pub left: bool,
    pub right: bool,
    pub bottom: bool,
}

/// Builder / owner of the panel layout.
pub struct PanelSystem {
    sizes: RwSignal<PanelSizes>,
    active: RwSignal<PanelFlags>,
    visible: RwSignal<PanelFlags>,
    available_size: RwSignal<Size>,
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
            active: create_rw_signal(PanelFlags::default()),
            visible: create_rw_signal(PanelFlags::default()),
            available_size: create_rw_signal(Size::ZERO),
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

    #[allow(dead_code)]
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
    pub fn visibility(&self) -> RwSignal<PanelFlags> {
        self.visible
    }

    pub fn active(&self) -> RwSignal<PanelFlags> {
        self.active
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
        let available_size = self.available_size;

        let start_flags = PanelFlags {
            left: self.left.is_some(),
            right: self.right.is_some(),
            bottom: self.bottom.is_some(),
        };

        self.active.update(|a| *a = start_flags);
        self.visible.update(|a| *a = start_flags);

        sizes.update(|s| {
            if self.left.is_none() {
                s.left = 0.0;
            }
            if self.right.is_none() {
                s.right = 0.0;
            }
            if self.bottom.is_none() {
                s.bottom = 0.0;
            }
        });

        let main = self
            .main
            .expect("PanelSystem::build requires a Main window");

        let main_view = container(scroll(main.into_view())).style(|s| {
            s.flex_grow(1.0)
                .flex_shrink(1.0)
                .flex_basis(0.0)
                .min_size(0.0, 0.0)
                .size_full()
        });

        let left_view = self.left.map_or_else(
            || empty().into_any(),
            |v| panel_container(PanelLocation::Left, v, sizes, visible, available_size).into_any(),
        );
        let right_view = self.right.map_or_else(
            || empty().into_any(),
            |v| panel_container(PanelLocation::Right, v, sizes, visible, available_size).into_any(),
        );
        let bottom_view = self.bottom.map_or_else(
            || empty().into_any(),
            |v| {
                panel_container(PanelLocation::Bottom, v, sizes, visible, available_size).into_any()
            },
        );

        let center_row = h_stack((left_view, main_view, right_view)).style(|s| {
            s.flex_row()
                .flex_grow(1.0)
                .min_height(0.0)
                .height_full()
                .width_full()
        });

        let workspace_area = v_stack((center_row, bottom_view))
            .style(|s| {
                s.flex_col()
                    .flex_grow(1.0)
                    .flex_basis(0.0)
                    .min_height(0.0)
                    .height_full()
                    .width_full()
            })
            .on_resize(move |rect| available_size.set(rect.size()));

        v_stack((toolbar.into_any(), workspace_area, status_bar.into_any())).style(|s| {
            s.flex_col()
                .width_full()
                .height_full()
                .background(theme().color.bg_app)
        })
    }
}

/// Build a single panel container (with optional drag handle + collapse).
fn panel_container(
    location: PanelLocation,
    content: impl IntoView + 'static,
    sizes: RwSignal<PanelSizes>,
    visible: RwSignal<PanelFlags>,
    available_size: RwSignal<Size>,
) -> impl View {
    let handle = resize_handle(location, sizes, available_size);

    let content = container(scroll(content.into_view())).style(|s| {
        s.flex_grow(1.0)
            .min_size(0.0, 0.0)
            .width_full()
            .height_full()
            .align_items(AlignItems::Stretch)
    });

    let inner: AnyView = match location {
        PanelLocation::Left => h_stack((content, handle))
            .style(|s| s.size_full().min_size(0.0, 0.0))
            .into_any(),
        PanelLocation::Right => h_stack((handle, content))
            .style(|s| s.size_full().min_size(0.0, 0.0))
            .into_any(),
        PanelLocation::Bottom => v_stack((handle, content))
            .style(|s| {
                s.size_full()
                    .min_size(0.0, 0.0)
                    .align_items(AlignItems::Stretch)
            })
            .into_any(),
    };

    let is_shown = move || match location {
        PanelLocation::Left => visible.with(|v| v.left),
        PanelLocation::Right => visible.with(|v| v.right),
        PanelLocation::Bottom => visible.with(|v| v.bottom),
    };

    container(inner).style(move |s| {
        let bw: f32 = 1.0;
        let s = s.apply_if(!is_shown(), |s| s.display(floem::style::Display::None));

        match location {
            PanelLocation::Left => s
                .width(sizes.with(|x| x.left as f32))
                .height_full()
                .min_height(0.0)
                .flex_shrink(1.0)
                .flex_grow(0.0)
                .border_right(bw)
                .border_color(theme().color.border_subtle)
                .background(theme().color.bg_surface),
            PanelLocation::Right => s
                .width(sizes.with(|x| x.right as f32))
                .height_full()
                .min_height(0.0)
                .flex_shrink(1.0)
                .flex_grow(0.0)
                .border_left(bw)
                .border_color(theme().color.border_subtle)
                .background(theme().color.bg_surface),
            PanelLocation::Bottom => s
                .height(sizes.with(|x| x.bottom as f32))
                .width_full()
                .min_width(0.0)
                .flex_shrink(1.0)
                .flex_grow(0.0)
                .align_items(AlignItems::Stretch)
                .border_top(bw)
                .border_color(theme().color.border_subtle)
                .background(theme().color.bg_surface),
        }
    })
}

/// A thin, in-flow drag handle that resizes its owning panel.
fn resize_handle(
    location: PanelLocation,
    sizes: RwSignal<PanelSizes>,
    available_size: RwSignal<Size>,
) -> impl View {
    let drag_start: RwSignal<Option<Point>> = create_rw_signal(None);

    let view = empty();
    let vid = view.id();
    view.on_event_stop(EventListener::PointerDown, move |event| {
        vid.request_active();
        if let Event::PointerDown(pointer_event) = event {
            drag_start.set(Some(pointer_event.pos));
        }
    })
    .on_event_stop(EventListener::PointerMove, move |event| {
        if let Event::PointerMove(pointer_event) = event {
            if let Some(drag_start_point) = drag_start.get_untracked() {
                let available_size = available_size.get_untracked();
                let current_sizes = sizes.get_untracked();

                let new = match location {
                    PanelLocation::Left => {
                        let new_size =
                            current_sizes.left - pointer_event.pos.x + drag_start_point.x;
                        new_size.clamp(
                            140.0,
                            (available_size.width - current_sizes.right).max(140.0),
                        )
                    }
                    PanelLocation::Right => {
                        let new_size =
                            current_sizes.right - pointer_event.pos.x + drag_start_point.x;
                        new_size.clamp(
                            180.0,
                            (available_size.width - current_sizes.left).max(180.0),
                        )
                    }
                    PanelLocation::Bottom => {
                        let new_size =
                            current_sizes.bottom - pointer_event.pos.y + drag_start_point.y;
                        new_size.max(120.0)
                    }
                };
                sizes.update(|s| match location {
                    PanelLocation::Left => s.left = new,
                    PanelLocation::Right => s.right = new,
                    PanelLocation::Bottom => s.bottom = new,
                });
            }
        }
    })
    .on_event_stop(EventListener::PointerUp, move |_| {
        vid.clear_active();
        drag_start.set(None);
    })
    .style(move |s| {
        let dragging = drag_start.get().is_some();
        let is_bottom = location == PanelLocation::Bottom;
        let hw: f32 = 4.0;
        let half_hw = hw / 2.0 - 1.0;

        s.apply_if(is_bottom, |s| {
            s.width_pct(100.0)
                .height(hw)
                .margin_top(-half_hw)
                .margin_bottom(-half_hw)
        })
        .apply_if(!is_bottom, |s| {
            s.width(hw)
                .height_pct(100.0)
                .apply_if(location == PanelLocation::Left, |s| {
                    s.margin_right(-half_hw).margin_left(-half_hw)
                })
                .apply_if(location == PanelLocation::Right, |s| {
                    s.margin_left(-half_hw).margin_right(-half_hw)
                })
        })
        .apply_if(dragging, |s| {
            s.background(theme().color.status_active)
                .apply_if(is_bottom, |s| s.cursor(CursorStyle::RowResize))
                .apply_if(!is_bottom, |s| s.cursor(CursorStyle::ColResize))
        })
        .hover(|s| {
            s.background(theme().color.status_active)
                .apply_if(is_bottom, |s| s.cursor(CursorStyle::RowResize))
                .apply_if(!is_bottom, |s| s.cursor(CursorStyle::ColResize))
        })
        .z_index(10)
    })
}
