//! Global stylesheet for the Articuelate application.

use floem::style::{CursorStyle, Style};
use floem::style_class;
use floem::views::ButtonClass;
use floem::views::scroll::{ScrollClass, ScrollCustomStyle};

use crate::style::theme::theme;

// ─── Cuelist / Grid Classes ───────────────────────────────────────────────

style_class!(pub CueRowGrid);
style_class!(pub CueRowGridEven);
style_class!(pub CueRowGridOdd);
style_class!(pub CueRowGridSelected);
style_class!(pub CueRowGridPlaying);
style_class!(pub CueRowGridDisabled);
style_class!(pub CueRowGridGroup);
style_class!(pub CueRowGridError);
style_class!(pub CueRowGridStandby);
style_class!(pub CuelistHeader);
style_class!(pub LabelDragHandle);
style_class!(pub BtnAddCueEnd);

// ─── Buttons & Controls ───────────────────────────────────────────────────

style_class!(pub BtnPunchDown);
style_class!(pub BtnPanic);
style_class!(pub BtnGo);
style_class!(pub BtnIconSm);
style_class!(pub BtnIconXs);
style_class!(pub BtnGlobal);
style_class!(pub BtnGlobalActive);
style_class!(pub BtnDanger);
style_class!(pub FieldInput);
style_class!(pub FieldInputXs);
style_class!(pub FieldTextarea);
style_class!(pub DeviceChip);
style_class!(pub DeviceDot);

// ─── Status & LED Graphics ────────────────────────────────────────────────

style_class!(pub LedDot);
style_class!(pub LedDotLit);
style_class!(pub MeterTrackSm);
style_class!(pub MeterTrackMd);
style_class!(pub PanelSurface);

// ─── Typography ───────────────────────────────────────────────────────────

style_class!(pub LabelMonoSm);
style_class!(pub LabelMonoXl);
style_class!(pub LabelBody);
style_class!(pub LabelHeading);
style_class!(pub FieldLabel);
style_class!(pub TextMonoSm);
style_class!(pub TextMonoXl);
style_class!(pub TextBody);
style_class!(pub TextHeading);

// ─── Conductor Classes ────────────────────────────────────────────────────

style_class!(pub ConductorCurrent);
style_class!(pub ConductorNext);

// ─── Tab Classes ──────────────────────────────────────────────────────────

style_class!(pub TabBtn);
style_class!(pub TabBtnActive);
style_class!(pub TabBtnInactive);

// ─── Badge Classes ────────────────────────────────────────────────────────

style_class!(pub BadgeSm);
style_class!(pub BadgeNext);
style_class!(pub BadgeRunning);
style_class!(pub BadgeChip);

// ─── Time Cell Classes ────────────────────────────────────────────────────

style_class!(pub TimeCell);
style_class!(pub TimeCellFill);
style_class!(pub TimeCellText);
style_class!(pub TimeCellTextEmphasis);
style_class!(pub TimeCellTextMuted);

// ─── Active Cue Classes ───────────────────────────────────────────────────

style_class!(pub ActiveCueRow);
style_class!(pub ActiveCueNumber);
style_class!(pub ActiveCueName);

// ─── Misc ─────────────────────────────────────────────────────────────────

style_class!(pub DividerVert);
style_class!(pub InspectorSectionHeader);
style_class!(pub StatusBarButton);

// ─── Global Stylesheet ────────────────────────────────────────────────────

/// Apply global class-based styles to the base view.
///
/// These styles are applied once and cascade to all matching views.
pub fn global_stylesheet(s: Style) -> Style {
    s// ─── Base Application Styles ───────────────────────────────────────
        .background(theme().color.bg_app)
        .color(theme().color.text_primary)
        .font_size(theme().font.body.size)
        .font_family(theme().font.body.family.clone())
        .selectable(false)
        .size_full()
        .min_size(0.0, 0.0)

        // ─── Scrollbar ─────────────────────────────────────────────────
        .class(ScrollClass, |s| {
            s.size_full()
                .min_size(0.0, 0.0)
                .apply_custom(ScrollCustomStyle::new().handle_thickness(theme().dim.space_xs))
        })

        // ─── Cuelist Grid ──────────────────────────────────────────────
        .class(CueRowGrid, |s| {
            s.grid()
                .width_full()
                .items_center()
                .gap(4.0)
                .border_bottom(1.0)
                .border_color(theme().color.border_row_divider)
                .padding_vert(4.0)
                .padding_horiz(theme().dim.space_sm)
                .padding_left(theme().dim.space_xs)
                .font_size(theme().font.body.size)
                .outline(0.0)
                .min_size(0.0, 0.0)
                .focus_visible(|s| {
                    s.border(1.0).border_color(theme().color.border_focus)
                })
        })
        .class(CueRowGridEven, |s| {
            s.background(theme().color.bg_surface)
        })
        .class(CueRowGridOdd, |s| {
            s.background(theme().color.bg_surface_raised)
        })
        .class(CueRowGridSelected, |s| {
            s.background(theme().color.bg_selection)
                .color(theme().color.text_primary)
        })
        .class(CueRowGridPlaying, |s| {
            s.background(theme().color.status_running_bg_20)
                .color(theme().color.text_primary)
        })
        .class(CueRowGridDisabled, |s| {
            s
        })
        .class(CueRowGridGroup, |s| {
            s.border_left(3.0).border_color(theme().color.status_group)
        })
        .class(CueRowGridError, |s| {
            s.background(theme().color.status_error_bg_12)
        })
        .class(CueRowGridStandby, |s| {
            s.border_left(3.0).border_color(theme().color.status_standby)
        })
        .class(CuelistHeader, |s| {
            s.grid()
                .width_full()
                .items_center()
                .gap(4.0)
                .border_bottom(1.0)
                .border_color(theme().color.element_border)
                .padding_vert(6.0)
                .padding_horiz(theme().dim.space_sm)
                .padding_left(theme().dim.space_xs)
                .font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
                .font_weight(theme().font.mono_sm.weight)
                .color(theme().color.text_secondary)
        })
        .class(LabelDragHandle, |s| {
            s.flex()
                .items_center()
                .justify_center()
                .color(theme().color.text_disabled)
                .hover(|s| s.color(theme().color.text_secondary))
        })

        // ─── Panels & Environment ──────────────────────────────────────
        .class(PanelSurface, |s| {
            s.background(theme().color.bg_surface)
                .border(1.0)
                .border_color(theme().color.border_subtle)
        })

        // ─── Punch-Down Button ─────────────────────────────────────────
        .class(BtnPunchDown, |s| {
            s.flex()
                .items_center()
                .justify_center()
                .background(theme().color.element_bg)
                .border(1.0)
                .border_color(theme().color.element_border)
                .border_radius(theme().dim.radius_sm)
                .color(theme().color.text_primary)
                .cursor(CursorStyle::Pointer)
                .hover(|s| s.background(theme().color.element_bg_hover))
                .active(|s| {
                    s.background(theme().color.element_bg_active)
                        .border_color(theme().color.border_focus)
                })
                .focus_visible(|s| s.border_color(theme().color.border_focus))
        })

        // ─── Icon Buttons ──────────────────────────────────────────────
        .class(BtnIconSm, |s| {
            s.flex()
                .items_center()
                .justify_center()
                .background(theme().color.element_bg)
                .border(1.0)
                .border_color(theme().color.element_border)
                .border_radius(theme().dim.radius_sm)
                .color(theme().color.text_secondary)
                .cursor(CursorStyle::Pointer)
                .height(theme().dim.control_md)
                .width(theme().dim.control_md)
                .padding(theme().dim.space_xs)
                .hover(|s| {
                    s.color(theme().color.text_primary)
                        .background(theme().color.element_bg_hover)
                })
                .active(|s| {
                    s.background(theme().color.element_bg_active)
                        .border_color(theme().color.border_focus)
                })
                .focus_visible(|s| s.border_color(theme().color.border_focus))
        })
        .class(BtnIconXs, |s| {
            s.flex()
                .items_center()
                .justify_center()
                .background(theme().color.element_bg)
                .border(1.0)
                .border_color(theme().color.element_border)
                .border_radius(theme().dim.radius_sm)
                .color(theme().color.text_secondary)
                .cursor(CursorStyle::Pointer)
                .height(theme().dim.space_xl)
                .width(theme().dim.space_xl)
                .padding(theme().dim.space_xs)
                .hover(|s| {
                    s.color(theme().color.text_primary)
                        .background(theme().color.element_bg_hover)
                })
                .active(|s| {
                    s.background(theme().color.element_bg_active)
                        .border_color(theme().color.border_focus)
                })
                .focus_visible(|s| s.border_color(theme().color.border_focus))
        })

        // ─── Panic Button ──────────────────────────────────────────────
        .class(BtnPanic, |s| {
            s.flex()
                .items_center()
                .justify_center()
                .background(theme().color.element_bg)
                .border(2.0)
                .border_color(theme().color.status_error)
                .border_radius(theme().dim.radius_sm)
                .color(theme().color.status_error)
                .cursor(CursorStyle::Pointer)
                .height(theme().dim.control_md)
                .width(theme().dim.btn_panic_width)
                .font_weight(floem::text::Weight::SEMIBOLD)
                .font_size(12.0)
                .hover(|s| s.background(theme().color.element_bg_hover))
                .active(|s| {
                    s.background(theme().color.element_bg_active)
                        .border_color(theme().color.status_error)
                })
                .focus_visible(|s| s.border_color(theme().color.status_error))
        })

        // ─── Go Button ─────────────────────────────────────────────────
        .class(BtnGo, |s| {
            s.flex()
                .items_center()
                .justify_center()
                .background(theme().color.element_bg)
                .border(2.0)
                .border_color(theme().color.status_running)
                .border_radius(theme().dim.radius_sm)
                .color(theme().color.status_running)
                .cursor(CursorStyle::Pointer)
                .width(theme().dim.btn_go_width)
                .flex_grow(1.0)
                .font_size(16.0)
                .font_weight(floem::text::Weight::BOLD)
                .hover(|s| s.background(theme().color.status_running_bg))
                .active(|s| {
                    s.background(theme().color.element_bg_active)
                        .border_color(theme().color.status_running)
                })
                .focus_visible(|s| s.border_color(theme().color.status_running))
        })

        // ─── Conductor Panel ───────────────────────────────────────────
        .class(ConductorCurrent, |s| {
            s.flex()
                .items_center()
                .height(theme().dim.control_md)
                .gap(8.0)
                .border_radius(theme().dim.radius_sm)
                .border(1.0)
                .border_color(theme().color.element_border)
                .background(theme().color.element_bg_hover)
                .padding_vert(6.0)
                .padding_horiz(theme().dim.space_md)
        })
        .class(ConductorNext, |s| {
            s.flex()
                .items_center()
                .min_size(0.0, 0.0)
                .flex_grow(1.0)
                .gap(8.0)
                .border_radius(theme().dim.radius_sm)
                .border(1.0)
                .border_color(theme().color.border_emphasized)
                .background(theme().color.element_bg)
                .padding_vert(6.0)
                .padding_horiz(theme().dim.space_md)
        })

        // ─── Labels & Typography ───────────────────────────────────────
        .class(LabelMonoSm, |s| {
            s.font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
                .color(theme().color.text_secondary)
                .min_width(0.0)
        })
        .class(LabelMonoXl, |s| {
            s.font_family(theme().font.mono_xl.family.clone())
                .font_size(theme().font.mono_xl.size)
                .font_weight(theme().font.mono_xl.weight)
                .line_height(theme().font.mono_xl.line_height as f32)
                .color(theme().color.text_primary)
                .min_width(0.0)
        })
        .class(LabelBody, |s| {
            s.font_family(theme().font.body.family.clone())
                .font_size(theme().font.body.size)
                .line_height(theme().font.body.line_height as f32)
                .color(theme().color.text_primary)
                .min_width(0.0)
        })
        .class(LabelHeading, |s| {
            s.font_family(theme().font.heading.family.clone())
                .font_size(theme().font.heading.size)
                .font_weight(theme().font.heading.weight)
                .line_height(theme().font.heading.line_height as f32)
                .color(theme().color.text_primary)
                .min_width(0.0)
        })
        .class(FieldLabel, |s| {
            s.font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
                .font_weight(floem::text::Weight::SEMIBOLD)
                .color(theme().color.text_disabled)
        })
        .class(TextMonoSm, |s| {
            s.font_size(theme().font.mono_sm.size)
        })
        .class(TextMonoXl, |s| {
            s.font_size(theme().font.mono_xl.size)
        })
        .class(TextBody, |s| {
            s.font_size(theme().font.body.size)
        })
        .class(TextHeading, |s| {
            s.font_size(theme().font.heading.size)
        })

        // ─── Form Elements ─────────────────────────────────────────────
        .class(FieldInput, |s| {
            s.height(theme().dim.control_sm)
                .font_family(theme().font.body.family.clone())
                .font_size(theme().font.body.size)
                .padding_horiz(theme().dim.space_sm)
                .background(theme().color.element_bg)
                .border(1.0)
                .border_color(theme().color.element_border)
                .border_radius(theme().dim.radius_sm)
                .color(theme().color.text_primary)
                .outline(0.0)
                .min_width(0.0)
                .focus_visible(|s| s.border_color(theme().color.border_focus))
        })
        .class(FieldInputXs, |s| {
            s.height(20.0)
                .font_family(theme().font.mono_sm.family.clone())
                .font_size(11.0)
                .padding_horiz(4.0)
                .background(theme().color.element_bg)
                .border(1.0)
                .border_color(theme().color.element_border)
                .border_radius(theme().dim.radius_sm)
                .color(theme().color.text_primary)
                .outline(0.0)
                .min_width(0.0)
                .focus_visible(|s| s.border_color(theme().color.border_focus))
        })
        .class(FieldTextarea, |s| {
            s.height(theme().dim.textarea_height)
                .border_radius(theme().dim.radius_sm)
                .border(1.0)
                .border_color(theme().color.element_border)
                .background(theme().color.element_bg)
                .padding_vert(6.0)
                .padding_horiz(theme().dim.space_sm)
                .font_family(theme().font.body.family.clone())
                .font_size(theme().font.body.size)
                .color(theme().color.text_primary)
                .outline(0.0)
                .min_width(0.0)
                .focus_visible(|s| s.border_color(theme().color.border_focus))
        })
        .class(InspectorSectionHeader, |s| {
            s.font_family(theme().font.mono_sm.family.clone())
                .font_size(10.0)
                .font_weight(floem::text::Weight::BOLD)
                .color(theme().color.text_disabled)
                .margin_bottom(theme().dim.space_xs)
        })

        // ─── Tab Classes ───────────────────────────────────────────────
        .class(TabBtn, |s| {
            s.flex()
                .items_center()
                .justify_center()
                .height(theme().dim.control_sm)
                .padding_horiz(theme().dim.space_md)
                .padding_vert(theme().dim.space_sm)
                .font_family(theme().font.body.family.clone())
                .font_size(12.0)
                .font_weight(floem::text::Weight::MEDIUM)
                .outline(0.0)
                .focus_visible(|s| s.border_color(theme().color.border_focus))
        })
        .class(TabBtnActive, |s| {
            s.flex()
                .items_center()
                .justify_center()
                .height(theme().dim.control_sm)
                .padding_horiz(theme().dim.space_md)
                .padding_vert(theme().dim.space_sm)
                .font_family(theme().font.body.family.clone())
                .font_size(12.0)
                .font_weight(floem::text::Weight::MEDIUM)
                .outline(0.0)
                .background(theme().color.bg_selection)
                .color(theme().color.text_primary)
        })
        .class(TabBtnInactive, |s| {
            s.flex()
                .items_center()
                .justify_center()
                .height(theme().dim.control_sm)
                .padding_horiz(theme().dim.space_md)
                .padding_vert(theme().dim.space_sm)
                .font_family(theme().font.body.family.clone())
                .font_size(12.0)
                .font_weight(floem::text::Weight::MEDIUM)
                .outline(0.0)
                .background(theme().color.element_bg)
                .color(theme().color.text_disabled)
                .hover(|s| {
                    s.background(theme().color.element_bg_hover)
                        .color(theme().color.text_primary)
                })
                .focus_visible(|s| s.border_color(theme().color.border_focus))
        })

        // ─── Badges ────────────────────────────────────────────────────
        .class(BadgeSm, |s| {
            s.border_radius(theme().dim.radius_sm)
                .padding_vert(2.0)
                .padding_horiz(theme().dim.space_sm)
                .font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
                .font_weight(floem::text::Weight::SEMIBOLD)
        })
        .class(BadgeNext, |s| {
            s.background(theme().color.status_playhead_bg)
                .color(theme().color.status_playhead)
        })
        .class(BadgeRunning, |s| {
            s.background(theme().color.status_running_bg)
                .color(theme().color.status_running)
        })
        .class(BadgeChip, |s| {
            s.min_width(68.0)
        })

        // ─── Time Cell ─────────────────────────────────────────────────
        .class(TimeCell, |s| {
            s.flex()
                .items_center()
                .justify_end()
                .height(theme().dim.time_cell)
                .border_radius(theme().dim.radius_sm)
                .padding_horiz(6.0)
        })
        .class(TimeCellFill, |s| {
            s.background(theme().color.status_running_bg_30)
        })
        .class(TimeCellText, |s| {
            s.font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
                .min_width(0.0)
        })
        .class(TimeCellTextEmphasis, |s| {
            s.font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.body.size)
                .color(theme().color.text_primary)
                .min_width(0.0)
        })
        .class(TimeCellTextMuted, |s| {
            s.font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.body.size)
                .color(theme().color.text_disabled)
                .min_width(0.0)
        })

        // ─── Active Cue Row ────────────────────────────────────────────
        .class(ActiveCueRow, |s| {
            s.flex()
                .items_start()
                .height(theme().dim.active_card_height)
                .gap(6.0)
                .border_radius(theme().dim.radius_sm)
                .border(1.0)
                .border_color(theme().color.element_border)
                .background(theme().color.element_bg_hover)
                .padding_horiz(theme().dim.space_sm)
        })
        .class(ActiveCueNumber, |s| {
            s.width(theme().dim.space_xl)
                .flex_shrink(0.0)
                .font_family(theme().font.mono_sm.family.clone())
                .font_size(theme().font.mono_sm.size)
                .color(theme().color.status_playhead)
        })
        .class(ActiveCueName, |s| {
            s.font_family(theme().font.body.family.clone())
                .font_size(12.0)
                .font_weight(floem::text::Weight::MEDIUM)
                .color(theme().color.text_primary)
                .min_width(0.0)
        })

        // ─── Global Runtime Buttons ────────────────────────────────────
        .class(BtnGlobal, |s| {
            s.flex()
                .items_center()
                .justify_center()
                .height(theme().dim.space_xl)
                .width(theme().dim.space_xl)
                .border_radius(theme().dim.radius_md)
                .border(1.0)
                .border_color(theme().color.element_border)
                .background(theme().color.element_bg)
                .color(theme().color.text_secondary)
                .outline(0.0)
                .hover(|s| {
                    s.background(theme().color.element_bg_hover)
                        .color(theme().color.text_primary)
                })
                .focus_visible(|s| s.border_color(theme().color.border_focus))
        })
        .class(BtnGlobalActive, |s| {
            s.border(1.0)
                .border_color(theme().color.status_group)
                .background(theme().color.status_group_bg)
                .color(theme().color.status_group)
                .hover(|s| s.background(theme().color.status_group_bg_25))
        })
        .class(BtnDanger, |s| {
            s.color(theme().color.status_error)
                .hover(|s| {
                    s.background(theme().color.status_error_bg)
                        .color(theme().color.status_error)
                })
                .focus_visible(|s| s.border_color(theme().color.status_error))
        })

        // ─── LED Meter & Level Indicators ──────────────────────────────
        .class(LedDot, |s| {
            s.height(theme().dim.led_dot)
                .width(theme().dim.led_dot)
                .border_radius(theme().dim.radius_full)
                .border(1.0)
                .border_color(theme().color.element_border)
                .background(theme().color.element_bg)
        })
        .class(LedDotLit, |s| {
            s
        })
        .class(MeterTrackSm, |s| {
            s.width(theme().dim.meter_width_sm)
        })
        .class(MeterTrackMd, |s| {
            s.width(theme().dim.meter_width_md)
        })

        // ─── Device Status Chip ────────────────────────────────────────
        .class(DeviceChip, |s| {
            s.flex()
                .width_full()
                .items_center()
                .gap(8.0)
                .border_radius(theme().dim.radius_full)
                .border(1.0)
                .border_color(theme().color.element_border)
                .background(theme().color.element_bg)
                .padding_vert(4.0)
                .padding_horiz(theme().dim.space_md)
                .outline(0.0)
                .hover(|s| s.background(theme().color.element_bg_hover))
                .focus_visible(|s| s.border_color(theme().color.border_focus))
        })
        .class(DeviceDot, |s| {
            s.height(theme().dim.dot_sm)
                .width(theme().dim.dot_sm)
                .flex_shrink(0.0)
                .border_radius(theme().dim.radius_full)
        })

        // ─── Divider ───────────────────────────────────────────────────
        .class(DividerVert, |s| {
            s.width(1.0).background(theme().color.element_border)
        })

        // ─── Status Bar Button ─────────────────────────────────────────
        .class(StatusBarButton, |s| {
            s.color(theme().color.text_secondary)
                .background(theme().color.element_bg)
                .border(1.0)
                .border_color(theme().color.element_border)
                .font_size(theme().dim.status_icon_size)
                .border_radius(theme().dim.radius_sm)
                .padding_horiz(theme().dim.space_xs)
                .hover(|s| s.background(theme().color.element_bg_hover))
                .active(|s| s.background(theme().color.element_bg_active))
        })

        // ─── Default Button ────────────────────────────────────────────
        .class(ButtonClass, |s| apply_interactable_base_styles(s))

        // ─── BtnAddCueEnd ──────────────────────────────────────────────
        .class(BtnAddCueEnd, |s| {
            apply_interactable_base_styles(s)
                .width_full()
                .justify_center()
                .padding_vert(theme().dim.space_xs)
        })
}

fn apply_interactable_base_styles(s: Style) -> Style {
    s.background(theme().color.element_bg)
        .border(1.0)
        .border_color(theme().color.element_border)
        .border_radius(theme().dim.radius_sm)
        .hover(|s| s.background(theme().color.element_bg_hover))
        .active(|s| s.background(theme().color.element_bg_active))
        .focus_visible(|s| s.border_color(theme().color.border_focus))
}