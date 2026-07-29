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

use floem::event::{EventPropagation, listener};
use floem::kurbo::{Point, Size};
use floem::reactive::{RwSignal, SignalGet, SignalUpdate, SignalWith};
use floem::style::{AlignItems, CursorStyle};
use floem::taffy::Display;
use floem::views::scroll::Scroll;
use floem::views::{Button, Container, Decorators, Empty, Stack};
use floem::{AnyView, IntoView, View};

use crate::style::theme;

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
#[derive(Clone, Copy)]
pub struct PanelSystem {
    sizes: RwSignal<PanelSizes>,
    active: RwSignal<PanelFlags>,
    visible: RwSignal<PanelFlags>,
    available_size: RwSignal<Size>,
}

impl PanelSystem {
    pub fn new() -> Self {
        PanelSystem {
            sizes: RwSignal::new(PanelSizes {
                left: theme().dim.min_panel_size,
                right: theme().dim.min_panel_size,
                bottom: theme().dim.min_panel_size,
            }),
            active: RwSignal::new(PanelFlags::default()),
            visible: RwSignal::new(PanelFlags::default()),
            available_size: RwSignal::new(Size::ZERO),
        }
    }

    pub fn builder(self) -> PanelSystemBuilder {
        PanelSystemBuilder::new(self)
    }
}

pub struct PanelSystemBuilder {
    pub handle: PanelSystem, // Wraps the handle
    main: Option<AnyView>,
    left: Option<AnyView>,
    right: Option<AnyView>,
    bottom: Option<AnyView>,
}

impl PanelSystemBuilder {
    pub fn new(system: PanelSystem) -> Self {
        PanelSystemBuilder {
            handle: system,
            main: None,
            left: None,
            right: None,
            bottom: None,
        }
    }

    /// Register the required centre window.
    #[allow(dead_code)]
    pub fn with_main(mut self, view: impl IntoView + 'static) -> Self {
        self.main = Some(view.into_any());
        self
    }

    /// Register the left panel with an optional initial width.
    ///
    /// When `width` is `None`, the panel starts at `min_panel_size`.
    #[allow(dead_code)]
    pub fn with_left(mut self, view: impl IntoView + 'static, width: Option<f32>) -> Self {
        self.handle.sizes.update(|sizes| {
            sizes.left = width
                .map(f64::from)
                .unwrap_or_else(|| theme().dim.min_panel_size);
        });
        self.left = Some(view.into_any());
        self
    }

    /// Register the right panel with an optional initial width.
    ///
    /// When `width` is `None`, the panel starts at `min_panel_size`.
    #[allow(dead_code)]
    pub fn with_right(mut self, view: impl IntoView + 'static, width: Option<f32>) -> Self {
        self.handle.sizes.update(|sizes| {
            sizes.right = width
                .map(f64::from)
                .unwrap_or_else(|| theme().dim.min_panel_size);
        });
        self.right = Some(view.into_any());
        self
    }

    /// Register the bottom panel with an optional initial height.
    ///
    /// When `height` is `None`, the panel starts at `min_panel_size`.
    #[allow(dead_code)]
    pub fn with_bottom(mut self, view: impl IntoView + 'static, height: Option<f32>) -> Self {
        self.handle.sizes.update(|sizes| {
            sizes.bottom = height
                .map(f64::from)
                .unwrap_or_else(|| theme().dim.min_panel_size);
        });
        self.bottom = Some(view.into_any());
        self
    }

    /// Assemble the full workspace view: toolbar on top, panels in the middle,
    /// status bar at the bottom.
    pub fn build(self) -> impl IntoView {
        let sizes = self.handle.sizes;
        let visible = self.handle.visible;
        let available_size = self.handle.available_size;

        let start_flags = PanelFlags {
            left: self.left.is_some(),
            right: self.right.is_some(),
            bottom: self.bottom.is_some(),
        };

        self.handle.active.update(|a| *a = start_flags);
        self.handle.visible.update(|a| *a = start_flags);

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

        let main_view = Container::new(main.into_view()).style(|s| {
            // A definite zero width makes this a genuine remaining-space flex item.
            // The submitted view must not contribute its intrinsic width to layout.
            s.width_full()
                .height_full()
                .flex_grow(1.0)
                .flex_shrink(1.0)
                .flex_basis(0.0)
                .min_size(0.0, 0.0)
        });

        let left_view = self.left.map_or_else(
            || Empty::new().into_any(),
            |v| panel_container(PanelLocation::Left, v, sizes, visible, available_size).into_any(),
        );
        let right_view = self.right.map_or_else(
            || Empty::new().into_any(),
            |v| panel_container(PanelLocation::Right, v, sizes, visible, available_size).into_any(),
        );
        let bottom_view = self.bottom.map_or_else(
            || Empty::new().into_any(),
            |v| {
                panel_container(PanelLocation::Bottom, v, sizes, visible, available_size).into_any()
            },
        );

        let center_row = Stack::horizontal((left_view, main_view, right_view)).style(|s| {
            s.flex_row()
                .flex_grow(1.0)
                .min_height(0.0)
                .height_full()
                .width_full()
        });

        Stack::vertical((center_row, bottom_view))
            .style(|s| {
                s.flex_col()
                    .flex_grow(1.0)
                    .flex_basis(0.0)
                    .min_height(0.0)
                    .height_full()
                    .width_full()
            })
            .on_event(listener::WindowResized, move |_cx, _size| {
                EventPropagation::Continue
            })
    }
}

impl PanelSystem {
    /// Shared visibility signal so a toolbar can toggle panels.
    #[allow(dead_code)]
    pub fn visibility(&self) -> RwSignal<PanelFlags> {
        self.visible
    }

    #[allow(dead_code)]
    pub fn active(&self) -> RwSignal<PanelFlags> {
        self.active
    }

    /// A small chevron/panel icon that toggles an optional panel.
    #[allow(dead_code)]
    pub fn panel_toggle_button(
        self,
        child_view: impl IntoView + 'static,
        which: PanelLocation,
    ) -> impl IntoView {
        let active = self.active;
        let visible = self.visible;

        let hide = move || {
            active.with(|v| match which {
                PanelLocation::Left => !v.left,
                PanelLocation::Right => !v.right,
                PanelLocation::Bottom => !v.bottom,
            })
        };

        let button_active = move || {
            visible.with(|v| match which {
                PanelLocation::Left => v.left,
                PanelLocation::Right => v.right,
                PanelLocation::Bottom => v.bottom,
            })
        };

        Button::new(child_view)
            .action(move || {
                visible.update(|v| match which {
                    PanelLocation::Left => v.left = !v.left,
                    PanelLocation::Right => v.right = !v.right,
                    PanelLocation::Bottom => v.bottom = !v.bottom,
                })
            })
            .style(move |s| {
                s.apply_if(hide(), |s| s.display(Display::None))
                    .apply_if(button_active(), |s| s.color(theme().color.status_playhead))
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

    let content = Container::new(Scroll::new(content.into_view())).style(|s| {
        s.flex_grow(1.0)
            .min_size(0.0, 0.0)
            .width_full()
            .height_full()
            .align_items(AlignItems::Stretch)
    });

    let inner: AnyView = match location {
        PanelLocation::Left => Stack::horizontal((content, handle))
            .style(|s| s.size_full().min_size(0.0, 0.0))
            .into_any(),
        PanelLocation::Right => Stack::horizontal((handle, content))
            .style(|s| s.size_full().min_size(0.0, 0.0))
            .into_any(),
        PanelLocation::Bottom => Stack::vertical((handle, content))
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

    Container::new(inner).style(move |s| {
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
    let drag_start: RwSignal<Option<Point>> = RwSignal::new(None);

    let view = Empty::new();
    let _view_id = view.id();
    view.on_event_stop(listener::PointerDown, move |_cx, event| {
        drag_start.set(Some(event.state.logical_point()));
    })
    .on_event_stop(listener::PointerMove, move |_cx, event| {
        if let Some(drag_start_point) = drag_start.get_untracked() {
            let available_size = available_size.get_untracked();
            let current_sizes = sizes.get_untracked();
            let pos = event.current.logical_point();

            let new = match location {
                PanelLocation::Left => {
                    let new_size = current_sizes.left - pos.x + drag_start_point.x;
                    new_size.clamp(
                        theme().dim.min_panel_size,
                        (available_size.width - current_sizes.right)
                            .max(theme().dim.min_panel_size),
                    )
                }
                PanelLocation::Right => {
                    let new_size = current_sizes.right - pos.x + drag_start_point.x;
                    new_size.clamp(
                        theme().dim.min_panel_size,
                        (available_size.width - current_sizes.left).max(theme().dim.min_panel_size),
                    )
                }
                PanelLocation::Bottom => {
                    let new_size = current_sizes.bottom - pos.y + drag_start_point.y;
                    new_size.clamp(
                        theme().dim.min_panel_size,
                        (available_size.height - 0.0).max(theme().dim.min_panel_size),
                    )
                }
            };
            sizes.update(|sizes| match location {
                PanelLocation::Left => sizes.left = new,
                PanelLocation::Right => sizes.right = new,
                PanelLocation::Bottom => sizes.bottom = new,
            });
        }
    })
    .on_event_stop(listener::PointerUp, move |_cx, _event| {
        drag_start.set(None);
    })
    .style(move |s| {
        s.cursor(CursorStyle::ColResize)
            .apply_if(drag_start.get_untracked().is_some(), |s| {
                s.selectable(false)
            })
            .min_size(0.0, 0.0)
            .flex_shrink(0.0)
            .hover(|s| s.background(theme().color.border_focus))
    })
}
