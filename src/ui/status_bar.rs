use floem::{
    IntoView,
    views::{Decorators, h_stack, text, v_stack},
};
use lucide_floem::Icon;

use crate::{
    theme::theme,
    ui::panel::{PanelLocation, PanelSystem},
};

pub fn view(panel_system: PanelSystem) -> impl IntoView {
    let left = text("STATUS: Connected (ASIO: Focusrite USB)").style(|s| {
        s.color(theme().color.status_active)
            .font_size(11.0)
            .font_family(theme().font.mono_sm.family)
    });
    let right = text("CPU: 4%   DSP: 12%").style(|s| {
        s.color(theme().color.text_secondary)
            .font_size(11.0)
            .font_family(theme().font.mono_sm.family)
    });
    let spacer = text("").style(|s| s.flex_grow(1.0));

    let bottom_toggle = panel_system.panel_toggle_button(Icon::PanelBottom, PanelLocation::Bottom);
    let left_toggle = panel_system.panel_toggle_button(Icon::PanelLeft, PanelLocation::Left);
    let right_toggle = panel_system.panel_toggle_button(Icon::PanelRight, PanelLocation::Right);

    let panel_toggles = h_stack((bottom_toggle, left_toggle, right_toggle));

    v_stack((h_stack((panel_toggles, spacer, right)).style(|s| {
        s.items_center()
            .gap(10.0)
            .padding_horiz(12.0)
            .padding_vert(4.0)
            .background(theme().color.bg_surface)
            .border_top(1.0)
            .border_color(theme().color.border_subtle)
            .height(24.0)
    }),))
    .style(|s| s.width_full())
}
