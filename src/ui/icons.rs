//! Centralized icon module for Articuelate.
//!
//! Wraps `lucide_floem::Icon` calls into a single mapping so the rest of the
//! UI never needs to import lucide-floem directly. Icon sizes are bound to
//! `ThemeDimensions` via the `app_icon` helper.

use floem::IntoView;
use floem::peniko::Color;
use floem::views::Decorators;
use lucide_floem::Icon;

/// All icons used in the application, mapped from the prototype's `AppIconName`.
pub enum AppIcon {
    // Transport
    Play,
    Pause,
    Stop,
    Panic,
    SkipBack,
    #[allow(dead_code)]
    Go,
    // Cue types
    Music,
    Folder,
    ListVideo,
    Network,
    Spline,
    // Actions
    Plus,
    #[allow(dead_code)]
    Trash2,
    #[allow(dead_code)]
    Pencil,
    #[allow(dead_code)]
    Copy,
    FileAudio,
    // UI
    Settings,
    Save,
    Columns2,
    Rows2,
    Clock,
    MousePointerClick,
    #[allow(dead_code)]
    ChevronDown,
    GripVertical,
    EllipsisVertical,
    // Panel
    #[allow(dead_code)]
    PanelBottom,
    #[allow(dead_code)]
    PanelLeft,
    #[allow(dead_code)]
    PanelRight,
}

impl AppIcon {
    fn lucide_icon(&self) -> Icon {
        match self {
            AppIcon::Play => Icon::Play,
            AppIcon::Pause => Icon::Pause,
            AppIcon::Stop => Icon::X,
            AppIcon::Panic => Icon::Ban,
            AppIcon::SkipBack => Icon::SkipBack,
            AppIcon::Go => Icon::Play,
            AppIcon::Music => Icon::Music,
            AppIcon::Folder => Icon::Folder,
            AppIcon::ListVideo => Icon::ListVideo,
            AppIcon::Network => Icon::Network,
            AppIcon::Spline => Icon::Spline,
            AppIcon::Plus => Icon::Plus,
            AppIcon::Trash2 => Icon::Trash2,
            AppIcon::Pencil => Icon::Pencil,
            AppIcon::Copy => Icon::Copy,
            AppIcon::FileAudio => Icon::FileAudio,
            AppIcon::Settings => Icon::Settings,
            AppIcon::Save => Icon::Save,
            AppIcon::Columns2 => Icon::Columns,
            AppIcon::Rows2 => Icon::Rows,
            AppIcon::Clock => Icon::Clock,
            AppIcon::MousePointerClick => Icon::MousePointerClick,
            AppIcon::ChevronDown => Icon::ChevronDown,
            AppIcon::GripVertical => Icon::GripVertical,
            AppIcon::EllipsisVertical => Icon::MoreVertical,
            AppIcon::PanelBottom => Icon::PanelBottom,
            AppIcon::PanelLeft => Icon::PanelLeft,
            AppIcon::PanelRight => Icon::PanelRight,
        }
    }
}

/// Render an app icon with the given size and color.
///
/// `size` matches the `--spacing-icon-sm` (14px) or `--spacing-icon-md` (18px)
/// tokens from the theme.
pub fn app_icon(icon: AppIcon, size: f32, color: Color) -> impl IntoView {
    icon.lucide_icon()
        .into_view()
        .style(move |s| s.size(size, size).color(color).flex_shrink(0.0))
}

/// Render an app icon with a default style (no explicit color, inherits).
#[allow(dead_code)]
pub fn app_icon_default(icon: AppIcon, size: f32) -> impl IntoView {
    icon.lucide_icon()
        .into_view()
        .style(move |s| s.size(size, size).flex_shrink(0.0))
}

/// Render an icon as a `fill` variant (stroke-width 0, used for playhead).
#[allow(dead_code)]
pub fn app_icon_fill(icon: AppIcon, size: f32, color: Color) -> impl IntoView {
    let svg = icon.lucide_icon();
    // lucide-floem icons use stroke; we render them normally and the
    // caller sets fill via style.
    svg.into_view()
        .style(move |s| s.size(size, size).color(color).flex_shrink(0.0))
}
