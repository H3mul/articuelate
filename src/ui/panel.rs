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
//! │      │                   │         │
//! │      ├───────────────────────┤         │
//! │      │      Bottom       │         │
//! └───────┴───────────────────────┴───────────┘
//! ```

use std::{rc::Rc, time::Duration};

use floem::easing::Linear;
use floem::event::{DragConfig, EventPropagation, listener};
use floem::kurbo::Size;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate, SignalWith};
use floem::style::{AlignItems, CursorStyle, Display};
use floem::views::{Button, Container, Decorators, Empty, Stack};
use floem::{AnyView, IntoView};

use crate::style::theme;

/// Where a registered window lives in the workspace.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PanelLocation {
    Left,
    Right,
    Bottom,
}

/// Pixel sizes of the resizable panels.
#[derive(Clone, Copy)]
pub struct PanelSizes {
    pub left: f64,
    pub right: f64,
    pub bottom: f64,
}

/// Visibility of the optional panels (Main is always shown).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PanelFlags {
    pub left: bool,
    pub right: bool,
    pub bottom: bool,
}

#[derive(Clone, Copy)]
pub struct PanelSystem {
    sizes: RwSignal<PanelSizes>,
    minimum_sizes: RwSignal<PanelSizes>,
    active: RwSignal<PanelFlags>,
    visible: RwSignal<PanelFlags>,
    available_size: RwSignal<Size>,
}

impl PanelSystem {
    pub fn new() -> Self {
        Self {
            sizes: RwSignal::new(PanelSizes {
                left: theme().dim.space_xl,
                right: theme().dim.space_xl,
                bottom: theme().dim.space_xl,
            }),
            minimum_sizes: RwSignal::new(PanelSizes {
                left: theme().dim.space_xl,
                right: theme().dim.space_xl,
                bottom: theme().dim.space_xl,
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
    pub handle: PanelSystem,
    main: Option<AnyView>,
    left: Option<AnyView>,
    right: Option<AnyView>,
    bottom: Option<AnyView>,
}

impl PanelSystemBuilder {
    pub fn new(system: PanelSystem) -> Self {
        Self {
            handle: system,
            main: None,
            left: None,
            right: None,
            bottom: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_main(mut self, view: impl IntoView + 'static) -> Self {
        self.main = Some(view.into_any());
        self
    }

    #[allow(dead_code)]
    pub fn with_left(mut self, view: impl IntoView + 'static, width: Option<f32>) -> Self {
        let initial = width.map(f64::from).unwrap_or(theme().dim.space_xl);
        self.handle.sizes.update(|sizes| sizes.left = initial);
        self.handle
            .minimum_sizes
            .update(|sizes| sizes.left = initial);
        self.left = Some(view.into_any());
        self
    }

    #[allow(dead_code)]
    pub fn with_right(mut self, view: impl IntoView + 'static, width: Option<f32>) -> Self {
        let initial = width.map(f64::from).unwrap_or(theme().dim.space_xl);
        self.handle.sizes.update(|sizes| sizes.right = initial);
        self.handle
            .minimum_sizes
            .update(|sizes| sizes.right = initial);
        self.right = Some(view.into_any());
        self
    }

    #[allow(dead_code)]
    pub fn with_bottom(mut self, view: impl IntoView + 'static, height: Option<f32>) -> Self {
        let initial = height.map(f64::from).unwrap_or(theme().dim.space_xl);
        self.handle.sizes.update(|sizes| sizes.bottom = initial);
        self.handle
            .minimum_sizes
            .update(|sizes| sizes.bottom = initial);
        self.bottom = Some(view.into_any());
        self
    }

    pub fn build(self) -> impl IntoView {
        let sizes = self.handle.sizes;
        let minimum_sizes = self.handle.minimum_sizes;
        let visible = self.handle.visible;
        let available_size = self.handle.available_size;

        let start_flags = PanelFlags {
            left: self.left.is_some(),
            right: self.right.is_some(),
            bottom: self.bottom.is_some(),
        };
        self.handle.active.update(|flags| *flags = start_flags);
        self.handle.visible.update(|flags| *flags = start_flags);

        let main = Container::new(
            self.main
                .expect("PanelSystem::build requires a Main window")
                .into_view(),
        )
        .style(|s| s.size_full().min_size(0.0, 0.0));

        let left = self.left.map_or_else(
            || Empty::new().into_any(),
            |view| {
                panel_container(
                    PanelLocation::Left,
                    view,
                    sizes,
                    minimum_sizes,
                    visible,
                    available_size,
                )
            },
        );
        let right = self.right.map_or_else(
            || Empty::new().into_any(),
            |view| {
                panel_container(
                    PanelLocation::Right,
                    view,
                    sizes,
                    minimum_sizes,
                    visible,
                    available_size,
                )
            },
        );
        let bottom = self.bottom.map_or_else(
            || Empty::new().into_any(),
            |view| {
                panel_container(
                    PanelLocation::Bottom,
                    view,
                    sizes,
                    minimum_sizes,
                    visible,
                    available_size,
                )
            },
        );

        let center = Stack::horizontal((left, main, right))
            .style(|s| s.width_full().height_full().min_size(0.0, 0.0));

        Stack::vertical((center, bottom))
            .style(|s| s.width_full().height_full().min_size(0.0, 0.0))
            .on_event(listener::WindowResized, move |_cx, size| {
                available_size.set(*size);
                EventPropagation::Continue
            })
    }
}

impl PanelSystem {
    #[allow(dead_code)]
    pub fn visibility(&self) -> RwSignal<PanelFlags> {
        self.visible
    }

    #[allow(dead_code)]
    pub fn active(&self) -> RwSignal<PanelFlags> {
        self.active
    }

    #[allow(dead_code)]
    pub fn panel_toggle_button(
        self,
        child_view: impl IntoView + 'static,
        which: PanelLocation,
    ) -> impl IntoView {
        let active = self.active;
        let visible = self.visible;

        Button::new(child_view)
            .action(move || {
                visible.update(|flags| match which {
                    PanelLocation::Left => flags.left = !flags.left,
                    PanelLocation::Right => flags.right = !flags.right,
                    PanelLocation::Bottom => flags.bottom = !flags.bottom,
                })
            })
            .style(move |s| {
                s.apply_if(
                    !active.with(|flags| match which {
                        PanelLocation::Left => flags.left,
                        PanelLocation::Right => flags.right,
                        PanelLocation::Bottom => flags.bottom,
                    }),
                    |s| s.display(Display::None),
                )
            })
    }
}

fn panel_container(
    location: PanelLocation,
    content: AnyView,
    sizes: RwSignal<PanelSizes>,
    minimum_sizes: RwSignal<PanelSizes>,
    visible: RwSignal<PanelFlags>,
    available_size: RwSignal<Size>,
) -> AnyView {
    let handle = resize_handle(location, sizes, minimum_sizes, available_size);
    let content = Container::new(content).style(|s| {
        s.size_full()
            .min_size(0.0, 0.0)
            .align_items(AlignItems::Stretch)
    });

    let inner = match location {
        PanelLocation::Left => Stack::horizontal((content, handle))
            .style(|s| s.size_full().min_size(0.0, 0.0))
            .into_any(),
        PanelLocation::Right => Stack::horizontal((handle, content))
            .style(|s| s.size_full().min_size(0.0, 0.0))
            .into_any(),
        PanelLocation::Bottom => Stack::vertical((handle, content))
            .style(|s| s.size_full().min_size(0.0, 0.0))
            .into_any(),
    };

    let shown = move || match location {
        PanelLocation::Left => visible.with(|flags| flags.left),
        PanelLocation::Right => visible.with(|flags| flags.right),
        PanelLocation::Bottom => visible.with(|flags| flags.bottom),
    };

    Container::new(inner)
        .style(move |s| {
            let s = s.apply_if(!shown(), |s| s.display(Display::None));
            match location {
                PanelLocation::Left => s
                    .width(sizes.with(|value| value.left))
                    .height_full()
                    .min_height(0.0)
                    .border_right(theme().dim.border_size)
                    .border_color(theme().color.border_subtle)
                    .background(theme().color.bg_surface),
                PanelLocation::Right => s
                    .width(sizes.with(|value| value.right))
                    .height_full()
                    .min_height(0.0)
                    .border_left(theme().dim.border_size)
                    .border_color(theme().color.border_subtle)
                    .background(theme().color.bg_surface),
                PanelLocation::Bottom => s
                    .height(sizes.with(|value| value.bottom))
                    .width_full()
                    .min_width(0.0)
                    .border_top(theme().dim.border_size)
                    .border_color(theme().color.border_subtle)
                    .background(theme().color.bg_surface),
            }
        })
        .into_any()
}

fn resize_handle(
    location: PanelLocation,
    sizes: RwSignal<PanelSizes>,
    minimum_sizes: RwSignal<PanelSizes>,
    available_size: RwSignal<Size>,
) -> AnyView {
    let drag_start = RwSignal::new(None::<floem::kurbo::Point>);
    let handle_size = theme().dim.space_xs;
    let view = Empty::new();

    view.on_event_stop(listener::PointerDown, move |cx, event| {
        drag_start.set(Some(event.state.logical_point()));
        if let Some(pointer_id) = event.pointer.pointer_id {
            cx.request_pointer_capture(pointer_id);
        }
    })
    .on_event_stop(listener::GainedPointerCapture, move |cx, token| {
        cx.start_drag(
            *token,
            DragConfig {
                threshold: 1.0,
                animation_duration: Duration::ZERO,
                easing: Rc::new(Linear),
                custom_data: None,
                track_targets: false,
            },
            false,
        );
    })
    .on_event_stop(listener::DragMove, move |_cx, event| {
        let Some(start) = drag_start.get_untracked() else {
            return;
        };

        let current = sizes.get_untracked();
        let minimum = minimum_sizes.get_untracked();
        let available = available_size.get_untracked();
        let point = event.current_state.logical_point();
        let next = match location {
            PanelLocation::Left => (current.left + point.x - start.x).clamp(
                minimum.left,
                (available.width - current.right).max(minimum.left),
            ),
            PanelLocation::Right => (current.right - point.x + start.x).clamp(
                minimum.right,
                (available.width - current.left).max(minimum.right),
            ),
            PanelLocation::Bottom => (current.bottom - point.y + start.y)
                .clamp(minimum.bottom, available.height.max(minimum.bottom)),
        };

        sizes.update(|value| match location {
            PanelLocation::Left => value.left = next,
            PanelLocation::Right => value.right = next,
            PanelLocation::Bottom => value.bottom = next,
        });
    })
    .on_event_stop(listener::PointerUp, move |_cx, _event| {
        drag_start.set(None);
    })
    .style(move |s| {
        let cursor = match location {
            PanelLocation::Bottom => CursorStyle::RowResize,
            PanelLocation::Left | PanelLocation::Right => CursorStyle::ColResize,
        };
        let s = s.min_size(0.0, 0.0).cursor(cursor);
        let s = match location {
            PanelLocation::Bottom => s.width_full().height(handle_size),
            PanelLocation::Left | PanelLocation::Right => s.width(handle_size).height_full(),
        };
        let s = s.flex_shrink(0.0);
        s.hover(move |s| s.background(theme().color.bg_selection))
            .active(move |s| s.background(theme().color.bg_selection_active))
    })
    .into_any()
}
