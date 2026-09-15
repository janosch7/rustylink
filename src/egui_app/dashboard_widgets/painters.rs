//! Painting routines shared by the static and the live widgets.

#![cfg(feature = "egui")]

use super::style::{
    WidgetPalette, color_mix, font_for_rect, inner_rect, safe_clamp_f32,
    should_render_dashboard_icon, switch_labels_visible, widget_palette,
};
use super::values::{format_scale_value, gauge_range};
use crate::model::Block;
use eframe::egui::{self, Align2, Color32, Pos2, Rect, Stroke, Vec2};

pub(super) fn paint_dashboard_widget_icon(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    palette: WidgetPalette,
) {
    // Draw the block's catalog icon (the single source of truth for its
    // small-size representation).  When the definition carries no icon, fall
    // back to a neutral field box.
    let def = crate::simulink_libraries::resolve_definition(block);
    if let Some(icon) = def.icon {
        let spec = crate::simulink_libraries::config::icon_to_spec(icon);
        // Simulink draws the Toggle and Rocker switches vertically; rotate their
        // minimized fallback glyph 90°.  The Slider Switch shares the same glyph
        // but stays upright.
        if matches!(
            block.block_type.as_str(),
            "ToggleSwitchBlock" | "RockerSwitchBlock"
        ) {
            crate::egui_app::render::draw_icon_spec_rotated_quarter(
                painter,
                rect,
                &spec,
                palette.text,
            );
        } else {
            crate::egui_app::render::draw_icon_spec(painter, rect, 1.0, &spec, palette.text, None);
        }
        return;
    }
    let inner = inner_rect(rect, 0.70);
    let body = Rect::from_center_size(
        inner.center(),
        Vec2::new(inner.width() * 0.8, inner.height() * 0.5),
    );
    painter.rect_stroke(
        body,
        3.0,
        Stroke::new(1.1_f32, palette.border),
        egui::StrokeKind::Inside,
    );
}

pub(super) fn paint_slider_visual(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    palette: WidgetPalette,
    font_scale: f32,
    fraction: f32,
) {
    let inner = inner_rect(rect, 0.94);
    let label_space = if inner.height() > 28.0 {
        inner.height() * 0.22
    } else {
        0.0
    };
    let track_center_y = inner.center().y - label_space * 0.12;
    let track_h = safe_clamp_f32(inner.height() * 0.12, 4.0, 10.0);
    let handle_r = safe_clamp_f32(track_h * 0.85, 4.5, inner.height() * 0.20);
    let track = Rect::from_min_max(
        Pos2::new(
            inner.left() + handle_r * 0.75,
            track_center_y - track_h * 0.5,
        ),
        Pos2::new(
            inner.right() - handle_r * 0.75,
            track_center_y + track_h * 0.5,
        ),
    );
    let track_fill = color_mix(palette.border, palette.bg_field, 0.55);
    painter.rect_filled(track, track_h * 0.5, track_fill);
    painter.rect_stroke(
        track,
        track_h * 0.5,
        Stroke::new(1.0_f32, palette.border),
        egui::StrokeKind::Inside,
    );

    let handle_x = egui::lerp(track.left()..=track.right(), fraction.clamp(0.0, 1.0));
    let handle_center = Pos2::new(handle_x, track.center().y);
    painter.circle_filled(handle_center, handle_r, Color32::WHITE);
    painter.circle_stroke(
        handle_center,
        handle_r,
        Stroke::new(1.2_f32, palette.border),
    );

    let tick_top = track.bottom() + 2.0;
    let tick_bottom = tick_top + safe_clamp_f32(inner.height() * 0.10, 2.0, 6.0);
    for index in 0..11 {
        let t = index as f32 / 10.0;
        let x = egui::lerp(track.left()..=track.right(), t);
        let tick_len = if index % 5 == 0 {
            tick_bottom - tick_top
        } else {
            (tick_bottom - tick_top) * 0.55
        };
        painter.line_segment(
            [Pos2::new(x, tick_top), Pos2::new(x, tick_top + tick_len)],
            Stroke::new(1.0_f32, palette.border),
        );
    }

    if label_space > 0.0 {
        let font =
            egui::FontId::proportional((font_for_rect(rect, font_scale) * 0.72).clamp(4.0, 14.0));
        let (min, max) = gauge_range(block);
        let label_y = tick_bottom + 1.5;
        painter.text(
            Pos2::new(track.left(), label_y),
            Align2::LEFT_TOP,
            format_scale_value(min),
            font.clone(),
            palette.text,
        );
        painter.text(
            Pos2::new(track.right(), label_y),
            Align2::RIGHT_TOP,
            format_scale_value(max),
            font,
            palette.text,
        );
    }
}

pub(super) fn paint_rocker_switch_visual(
    painter: &egui::Painter,
    rect: &Rect,
    palette: WidgetPalette,
    is_on: bool,
    font_scale: f32,
) {
    let inner = inner_rect(rect, 0.80);
    let show_labels = switch_labels_visible(rect, true);
    let fsz = font_for_rect(rect, font_scale).min(inner.width().min(inner.height()) * 0.18);
    let font = egui::FontId::proportional(fsz);
    let label_pad = if show_labels { fsz + 4.0 } else { 2.0 };
    let housing_area = Rect::from_min_max(
        Pos2::new(inner.left(), inner.top() + label_pad),
        Pos2::new(inner.right(), inner.bottom() - label_pad),
    );
    let cx = housing_area.center().x;
    let cy = housing_area.center().y;
    let w = (housing_area.width() * 0.30).clamp(10.0, 28.0);
    let h = safe_clamp_f32(housing_area.height() * 0.82, 18.0, housing_area.height());
    let housing = Rect::from_center_size(Pos2::new(cx, cy), Vec2::new(w, h));
    let housing_fill = if is_on {
        palette.accent.linear_multiply(0.16)
    } else {
        palette.bg_field.linear_multiply(0.92)
    };
    painter.rect_filled(housing, w * 0.4, housing_fill);
    painter.rect_stroke(
        housing,
        w * 0.4,
        Stroke::new(1.0_f32, palette.border),
        egui::StrokeKind::Inside,
    );

    let rocker_w = w * 0.85;
    let rocker_h = h * 0.55;
    let rocker = if is_on {
        Rect::from_min_max(
            Pos2::new(cx - rocker_w / 2.0, housing.top() + 1.0),
            Pos2::new(cx + rocker_w / 2.0, housing.top() + rocker_h + 1.0),
        )
    } else {
        Rect::from_min_max(
            Pos2::new(cx - rocker_w / 2.0, housing.bottom() - rocker_h - 1.0),
            Pos2::new(cx + rocker_w / 2.0, housing.bottom() - 1.0),
        )
    };
    let rocker_fill = if is_on {
        palette.accent.linear_multiply(0.42)
    } else {
        Color32::from_rgb(230, 230, 235)
    };
    painter.rect_filled(rocker, 3.0, rocker_fill);
    painter.rect_stroke(
        rocker,
        3.0,
        Stroke::new(1.0_f32, palette.border),
        egui::StrokeKind::Inside,
    );

    if show_labels {
        painter.text(
            Pos2::new(inner.center().x, inner.top() + 1.0),
            Align2::CENTER_TOP,
            "On",
            font.clone(),
            palette.text,
        );
        painter.text(
            Pos2::new(inner.center().x, inner.bottom() - 1.0),
            Align2::CENTER_BOTTOM,
            "Off",
            font,
            palette.text,
        );
    }
}

pub(super) fn paint_switch_visual(
    painter: &egui::Painter,
    rect: &Rect,
    palette: WidgetPalette,
    is_on: bool,
    vertical: bool,
    font_scale: f32,
) {
    let inner = inner_rect(rect, 0.80);
    let show_labels = switch_labels_visible(rect, vertical);
    let fsz = font_for_rect(rect, font_scale).min(inner.width().min(inner.height()) * 0.18);
    let font = egui::FontId::proportional(fsz);
    let track_size = if vertical {
        Vec2::new(
            (inner.width() * 0.30).clamp(14.0, 28.0),
            (inner.height() * if show_labels { 0.40 } else { 0.62 })
                .clamp(18.0, (inner.height() - 4.0).max(18.0)),
        )
    } else {
        Vec2::new(
            (inner.width() * if show_labels { 0.32 } else { 0.62 })
                .clamp(18.0, (inner.width() - 6.0).max(18.0)),
            (inner.height() * 0.28).clamp(10.0, 22.0),
        )
    };
    let track = Rect::from_center_size(inner.center(), track_size);
    let rounding = if vertical {
        track.width() * 0.5
    } else {
        track.height() * 0.5
    };
    let track_fill = if is_on {
        palette.accent.linear_multiply(0.32)
    } else {
        palette.bg_field.linear_multiply(0.94)
    };
    painter.rect_filled(track, rounding, track_fill);
    painter.rect_stroke(
        track,
        rounding,
        Stroke::new(1.0_f32, palette.border),
        egui::StrokeKind::Inside,
    );

    let thumb_r = if vertical {
        safe_clamp_f32(track.width() * 0.32, 4.0, track.height() * 0.2)
    } else {
        safe_clamp_f32(track.height() * 0.40, 4.0, track.width() * 0.2)
    };
    let thumb_center = if vertical {
        let y = if is_on {
            track.top() + rounding
        } else {
            track.bottom() - rounding
        };
        Pos2::new(track.center().x, y)
    } else {
        let x = if is_on {
            track.right() - rounding
        } else {
            track.left() + rounding
        };
        Pos2::new(x, track.center().y)
    };
    painter.circle_filled(thumb_center, thumb_r, Color32::WHITE);
    painter.circle_stroke(thumb_center, thumb_r, Stroke::new(1.0_f32, palette.border));

    if vertical && show_labels {
        painter.text(
            Pos2::new(inner.center().x, inner.top() + 1.0),
            Align2::CENTER_TOP,
            "On",
            font.clone(),
            palette.text,
        );
        painter.text(
            Pos2::new(inner.center().x, inner.bottom() - 1.0),
            Align2::CENTER_BOTTOM,
            "Off",
            font,
            palette.text,
        );
    } else if !vertical && show_labels {
        painter.text(
            Pos2::new(inner.left() + 1.0, inner.center().y),
            Align2::LEFT_CENTER,
            "Off",
            font.clone(),
            palette.text,
        );
        painter.text(
            Pos2::new(inner.right() - 1.0, inner.center().y),
            Align2::RIGHT_CENTER,
            "On",
            font,
            palette.text,
        );
    }
}

/// Draw the small-size icon fallback for dashboard widgets; returns true
/// when it handled drawing (block too small for a full live widget).
pub(super) fn dashboard_icon_fallback(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    _font_scale: f32,
    _live_value: f64,
) -> bool {
    if !should_render_dashboard_icon(rect) {
        return false;
    }
    // No block-type branching: the catalog icon is the single small-size
    // representation for every dashboard widget.
    paint_dashboard_widget_icon(painter, block, rect, widget_palette(block));
    true
}
