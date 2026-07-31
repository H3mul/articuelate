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

use floem::reactive::{RwSignal, SignalUpdate, SignalWith};
use floem::style::Display;
use floem::views::resizable::Resizable;
use floem::views::{Container, Decorators, Empty};
use floem::{AnyView, IntoView};

/// Where a registered window lives in the workspace.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PanelLocation {
    Left,
    Right,
    Bottom,
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
    active: RwSignal<PanelFlags>,
    visible: RwSignal<PanelFlags>,
}

impl PanelSystem {
    pub fn new() -> Self {
        PanelSystem {
            active: RwSignal::new(PanelFlags::default()),
            visible: RwSignal::new(PanelFlags::default()),
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
    left_width: Option<f64>,
    right_width: Option<f64>,
    bottom_height: Option<f64>,
}

impl PanelSystemBuilder {
    pub fn new(system: PanelSystem) -> Self {
        PanelSystemBuilder {
            handle: system,
            main: None,
            left: None,
            right: None,
            bottom: None,
            left_width: None,
            right_width: None,
            bottom_height: None,
        }
    }

    /// Register the required centre window.
    #[allow(dead_code)]
    pub fn with_main(mut self, view: impl IntoView + 'static) -> Self {
        self.main = Some(view.into_any());
        self
    }

    /// Register the left panel with an optional initial width.
    #[allow(dead_code)]
    pub fn with_left(mut self, view: impl IntoView + 'static, width: Option<f32>) -> Self {
        self.left_width = width.map(f64::from);
        self.left = Some(view.into_any());
        self
    }

    /// Register the right panel with an optional initial width.
    #[allow(dead_code)]
    pub fn with_right(mut self, view: impl IntoView + 'static, width: Option<f32>) -> Self {
        self.right_width = width.map(f64::from);
        self.right = Some(view.into_any());
        self
    }

    /// Register the bottom panel with an optional initial height.
    #[allow(dead_code)]
    pub fn with_bottom(mut self, view: impl IntoView + 'static, height: Option<f32>) -> Self {
        self.bottom_height = height.map(f64::from);
        self.bottom = Some(view.into_any());
        self
    }

    /// Assemble the full workspace view.
    pub fn build(self) -> impl IntoView {
        let visible = self.handle.visible;
        let _left_width = self.left_width;
        let _right_width = self.right_width;
        let _bottom_height = self.bottom_height;

        let start_flags = PanelFlags {
            left: self.left.is_some(),
            right: self.right.is_some(),
            bottom: self.bottom.is_some(),
        };

        self.handle.active.update(|a| *a = start_flags);
        self.handle.visible.update(|a| *a = start_flags);

        let main = self
            .main
            .expect("PanelSystem::build requires a Main window");

        let main_view = Container::new(main.into_view())
            .style(|s| s.width_full().height_full().min_size(0.0, 0.0));

        // Build the centre row: left panel | main | right panel
        let center_row = if self.left.is_some() || self.right.is_some() {
            let left_view = panel_view(self.left, PanelLocation::Left, visible);
            let right_view = panel_view(self.right, PanelLocation::Right, visible);

            Resizable::new((left_view, main_view, right_view))
                .style(|s| s.width_full().height_full().min_height(0.0))
                .into_any()
        } else {
            main_view.into_any()
        };

        // if let Some(bottom) = self.bottom {
        //     let bottom_view = panel_view(Some(bottom), PanelLocation::Bottom, visible);
        //     Resizable::new((center_row, bottom_view))
        //         .style(|s| s.flex_col().width_full().height_full().min_height(0.0))
        //         .into_any()
        // } else {
        //     center_row.into_any()
        // }
        center_row.into_any()
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
}

/// Wrap an optional panel view with scroll + visibility toggle.
fn panel_view(
    view: Option<AnyView>,
    location: PanelLocation,
    visible: RwSignal<PanelFlags>,
) -> AnyView {
    let is_shown = move || match location {
        PanelLocation::Left => visible.with(|v| v.left),
        PanelLocation::Right => visible.with(|v| v.right),
        PanelLocation::Bottom => visible.with(|v| v.bottom),
    };

    if let Some(view) = view {
        view.style(move |s| s.apply_if(!is_shown(), |s| s.display(Display::None)))
    } else {
        Empty::new().into_any()
    }
}
