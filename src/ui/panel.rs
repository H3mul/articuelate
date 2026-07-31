//! A small, decoupled, Lapce-inspired resizable panel system.
//!
//! The panel layout is rebuilt when panel visibility changes so each
//! `Resizable` contains only the panes that are currently visible.

use floem::reactive::{RwSignal, SignalGet, SignalUpdate, SignalWith};
use floem::style::Display;
use floem::view::View;

use floem::views::resizable::Resizable;
use floem::views::{Container, Decorators, dyn_container};
use floem::{AnyView, IntoView, ViewId};

type ViewFactory = Box<dyn Fn() -> AnyView>;

/// Where a registered window lives in the workspace.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PanelLocation {
    Left,
    Right,
    Bottom,
}

/// Visibility of the optional panels (Main is always shown).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PanelFlags {
    pub left: bool,
    pub right: bool,
    pub bottom: bool,
}

#[derive(Clone, Copy, Default)]
struct PanelSizes {
    left: Option<f64>,
    right: Option<f64>,
    bottom: Option<f64>,
}

/// Builder / owner of the panel layout.
#[derive(Clone, Copy)]
pub struct PanelSystem {
    active: RwSignal<PanelFlags>,
    visible: RwSignal<PanelFlags>,
    sizes: RwSignal<PanelSizes>,
    center_view: RwSignal<Option<ViewId>>,
    bottom_view: RwSignal<Option<ViewId>>,
}

impl PanelSystem {
    pub fn new() -> Self {
        Self {
            active: RwSignal::new(PanelFlags::default()),
            visible: RwSignal::new(PanelFlags::default()),
            sizes: RwSignal::new(PanelSizes::default()),
            center_view: RwSignal::new(None),
            bottom_view: RwSignal::new(None),
        }
    }

    pub fn builder(self) -> PanelSystemBuilder {
        PanelSystemBuilder::new(self)
    }
}

pub struct PanelSystemBuilder {
    pub handle: PanelSystem,
    main: Option<ViewFactory>,
    left: Option<ViewFactory>,
    right: Option<ViewFactory>,
    bottom: Option<ViewFactory>,
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

    /// Register the required centre window factory.
    #[allow(dead_code)]
    pub fn with_main<V>(mut self, factory: impl Fn() -> V + 'static) -> Self
    where
        V: IntoView + 'static,
    {
        self.main = Some(Box::new(move || factory().into_any()));
        self
    }

    /// Register the left panel factory with an optional initial width.
    #[allow(dead_code)]
    pub fn with_left<V>(mut self, factory: impl Fn() -> V + 'static, width: Option<f32>) -> Self
    where
        V: IntoView + 'static,
    {
        self.handle
            .sizes
            .update(|sizes| sizes.left = width.map(f64::from));
        self.left = Some(Box::new(move || factory().into_any()));
        self
    }

    /// Register the right panel factory with an optional initial width.
    #[allow(dead_code)]
    pub fn with_right<V>(mut self, factory: impl Fn() -> V + 'static, width: Option<f32>) -> Self
    where
        V: IntoView + 'static,
    {
        self.handle
            .sizes
            .update(|sizes| sizes.right = width.map(f64::from));
        self.right = Some(Box::new(move || factory().into_any()));
        self
    }

    /// Register the bottom panel factory with an optional initial height.
    #[allow(dead_code)]
    pub fn with_bottom<V>(mut self, factory: impl Fn() -> V + 'static, height: Option<f32>) -> Self
    where
        V: IntoView + 'static,
    {
        self.handle
            .sizes
            .update(|sizes| sizes.bottom = height.map(f64::from));
        self.bottom = Some(Box::new(move || factory().into_any()));
        self
    }

    /// Assemble the full workspace view.
    pub fn build(self) -> impl IntoView {
        let visible = self.handle.visible;
        let sizes = self.handle.sizes;
        let center_view = self.handle.center_view;
        let bottom_view = self.handle.bottom_view;
        let main = self
            .main
            .expect("PanelSystem::build requires a Main window");
        let left = self.left;
        let right = self.right;
        let bottom = self.bottom;

        let start_flags = PanelFlags {
            left: left.is_some(),
            right: right.is_some(),
            bottom: bottom.is_some(),
        };
        self.handle.active.update(|flags| *flags = start_flags);
        self.handle.visible.update(|flags| *flags = start_flags);

        dyn_container(
            move || visible.get(),
            move |flags| {
                build_panel_layout(
                    flags,
                    &main,
                    left.as_ref(),
                    right.as_ref(),
                    bottom.as_ref(),
                    sizes,
                    center_view,
                    bottom_view,
                )
            },
        )
        .style(|s| s.size_full().min_size(0.0, 0.0))
    }
}

fn build_panel_layout(
    flags: PanelFlags,
    main: &ViewFactory,
    left: Option<&ViewFactory>,
    right: Option<&ViewFactory>,
    bottom: Option<&ViewFactory>,
    sizes: RwSignal<PanelSizes>,
    center_view: RwSignal<Option<ViewId>>,
    bottom_resizable_view: RwSignal<Option<ViewId>>,
) -> AnyView {
    let main_view = panel_content(main());

    let center = match (flags.left && left.is_some(), flags.right && right.is_some()) {
        (true, true) => {
            let left_view = panel_content(left.expect("left factory missing")());
            let right_view = panel_content(right.expect("right factory missing")());
            let resizable = Resizable::new((left_view, main_view, right_view))
                .custom_sizes(move || initial_horizontal_sizes(flags, sizes.get()))
                .style(|s| s.size_full().min_height(0.0));
            center_view.set(Some(resizable.id()));
            resizable.into_any()
        }
        (true, false) => {
            let left_view = panel_content(left.expect("left factory missing")());
            let resizable = Resizable::new((left_view, main_view))
                .custom_sizes(move || initial_horizontal_sizes(flags, sizes.get()))
                .style(|s| s.size_full().min_height(0.0));
            center_view.set(Some(resizable.id()));
            resizable.into_any()
        }
        (false, true) => {
            let right_view = panel_content(right.expect("right factory missing")());
            let resizable = Resizable::new((main_view, right_view))
                .custom_sizes(move || initial_horizontal_sizes(flags, sizes.get()))
                .style(|s| s.size_full().min_height(0.0));
            center_view.set(Some(resizable.id()));
            resizable.into_any()
        }
        (false, false) => main_view,
    };

    if flags.bottom && bottom.is_some() {
        let bottom_view = panel_content(bottom.expect("bottom factory missing")());
        let resizable = Resizable::new((center, bottom_view))
            .custom_sizes(move || initial_bottom_sizes(sizes.get()))
            .style(|s| s.flex_col().size_full().min_height(0.0));
        bottom_resizable_view.set(Some(resizable.id()));
        resizable.into_any()
    } else {
        center
    }
}

fn panel_content(view: AnyView) -> AnyView {
    Container::new(view)
        .style(|s| s.size_full().min_size(0.0, 0.0))
        .into_any()
}

fn initial_horizontal_sizes(flags: PanelFlags, sizes: PanelSizes) -> Vec<(usize, f64)> {
    match (flags.left, flags.right) {
        (true, true) => [
            sizes.left.map(|size| (0, size)),
            sizes.right.map(|size| (2, size)),
        ]
        .into_iter()
        .flatten()
        .collect(),
        (true, false) => sizes.left.into_iter().map(|size| (0, size)).collect(),
        (false, true) => sizes.right.into_iter().map(|size| (1, size)).collect(),
        (false, false) => Vec::new(),
    }
}

fn initial_bottom_sizes(sizes: PanelSizes) -> Vec<(usize, f64)> {
    sizes.bottom.into_iter().map(|size| (1, size)).collect()
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

    fn capture_size(&self, which: PanelLocation) {
        match which {
            PanelLocation::Left | PanelLocation::Right => {
                if let Some(view_id) = self.center_view.get()
                    && let Some(size) = view_id
                        .children()
                        .get(if matches!(which, PanelLocation::Left) {
                            0
                        } else {
                            view_id.children().len().saturating_sub(1)
                        })
                        .map(|child| child.get_layout_rect().width())
                {
                    self.sizes.update(|sizes| match which {
                        PanelLocation::Left => sizes.left = Some(size),
                        PanelLocation::Right => sizes.right = Some(size),
                        PanelLocation::Bottom => {}
                    });
                }
            }
            PanelLocation::Bottom => {
                if let Some(view_id) = self.bottom_view.get()
                    && let Some(child) = view_id.children().last()
                {
                    let size = child.get_layout_rect().height();
                    self.sizes.update(|sizes| sizes.bottom = Some(size));
                }
            }
        }
    }

    /// Toggle an optional panel from a toolbar control.
    #[allow(dead_code)]
    pub fn panel_toggle_button(
        self,
        child_view: impl IntoView + 'static,
        which: PanelLocation,
    ) -> impl IntoView {
        let active = self.active;
        let visible = self.visible;
        Container::new(child_view)
            .action(move || {
                self.capture_size(which);
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
