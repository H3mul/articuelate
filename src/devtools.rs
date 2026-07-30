//! Conditional Floem UI Inspector attachment.
//!
//! When the `debug-ui` feature is enabled, the top-level view gets an F11 key
//! handler that toggles the Floem Inspector overlay. In release builds this is
//! a zero-cost no-op.

use floem::prelude::*;

pub trait DebugInspectorExt: IntoView + Sized + 'static {
    fn attach_inspector(self) -> impl IntoView;
}

/// Conditionally attaches an F11 key handler that toggles the Floem Inspector
/// overlay when the `debug-ui` feature flag is enabled.
///
/// Call this on the root view returned from the window closure.
#[cfg(feature = "debug-ui")]
impl<T: IntoView + Sized + 'static> DebugInspectorExt for T {
    fn attach_inspector(self) -> impl IntoView {
        self.on_event_stop(el::KeyUp, move |_cx, KeyboardEvent { key, .. }| {
            if let Key::Named(NamedKey::F12) = key {
                floem::action::inspect();
            }
        })
    }
}

#[cfg(not(feature = "debug-ui"))]
impl<T: IntoView + Sized + 'static> DebugInspectorExt for T {
    #[inline(always)]
    fn attach_inspector(self) -> impl IntoView {
        // In release builds, return the view unchanged with zero overhead
        self
    }
}
