use floem::{
    IntoView,
    views::{Decorators, h_stack, text, v_stack},
};
use lucide_floem::Icon;

use crate::{
    style::{style::StatusBarButton, theme},
    ui::panel::{PanelLocation, PanelSystem},
};

pub fn view(panel_system: PanelSystem) -> impl IntoView {
    let _left = text("STATUS: Connected (ASIO: Focusrite USB)").style(|s| {
        s.color(theme().color.status_active)
            .font_size(11.0)
            .font_family(theme().font.mono_sm.family)
    });
    let _right = text("CPU: 4%   DSP: 12%").style(|s| {
        s.color(theme().color.text_secondary)
            .font_size(11.0)
            .font_family(theme().font.mono_sm.family)
    });
    let spacer = text("").style(|s| s.flex_grow(1.0));

    let bottom_toggle = panel_system
        .panel_toggle_button(Icon::PanelBottom, PanelLocation::Bottom)
        .class(StatusBarButton);
    let left_toggle = panel_system
        .panel_toggle_button(Icon::PanelLeft, PanelLocation::Left)
        .class(StatusBarButton);
    let right_toggle = panel_system
        .panel_toggle_button(Icon::PanelRight, PanelLocation::Right)
        .class(StatusBarButton);

    let panel_toggles = h_stack((bottom_toggle, left_toggle, right_toggle));

    v_stack((h_stack((spacer, panel_toggles)).style(|s| {
        s.items_center()
            .gap(theme().dim.space_xs / 2.0)
            .padding_horiz(theme().dim.space_md)
            .background(theme().color.bg_app)
            .border_top(theme().dim.border_size)
            .border_color(theme().color.border_subtle)
            .height(theme().dim.status_bar_height)
    }),))
    .style(|s| s.width_full())
}
