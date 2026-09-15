//! Live widgets: an interactive control bound to a dashboard value.

#![cfg(feature = "egui")]

use super::painters::{
    dashboard_icon_fallback, paint_rocker_switch_visual, paint_slider_visual, paint_switch_visual,
};
use super::style::{
    ACCENT, ACCENT_DARK, BORDER, NEEDLE_RED, TEXT_DARK, font_for_rect, inner_rect,
    radio_group_metrics, safe_clamp_f32, widget_palette,
};
use super::values::{
    checkbox_label, checkbox_state_from_value, combo_box_label, discrete_live_index,
    format_dashboard_scalar_with_options, format_scale_value, gauge_range, lamp_color_for_value,
    normalized_live_value, option_labels, prop, push_button_visuals,
};
use crate::model::Block;
use eframe::egui::{self, Align2, Color32, Pos2, Rect, Stroke, Vec2};
use std::f32::consts::PI;

pub fn live_push_button(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    live_value: f64,
    _display_options: Option<&crate::live_values::LiveValueDisplayOptions>,
) {
    if dashboard_icon_fallback(painter, block, rect, font_scale, live_value) {
        return;
    }
    let palette = widget_palette(block);
    let inner = inner_rect(rect, 0.85);
    let label = prop(block, "ButtonText", &block.name);
    let (icon_color, off_value) = push_button_visuals(block, Some(live_value));
    let is_on = (live_value - off_value).abs() > f64::EPSILON;
    let fill = if is_on {
        icon_color.linear_multiply(0.2)
    } else {
        palette.bg_field
    };
    painter.rect_filled(inner, 4.0, fill);
    painter.rect_stroke(
        inner,
        4.0,
        Stroke::new(1.0_f32, palette.border),
        egui::StrokeKind::Inside,
    );
    let fsz = font_for_rect(rect, font_scale).min(inner.height() * 0.45);
    painter.text(
        inner.center(),
        Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(fsz),
        palette.text,
    );
}

pub fn live_slider_switch(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    live_value: f64,
    _display_options: Option<&crate::live_values::LiveValueDisplayOptions>,
) {
    if dashboard_icon_fallback(painter, block, rect, font_scale, live_value) {
        return;
    }
    let palette = widget_palette(block);
    paint_switch_visual(painter, rect, palette, live_value >= 0.5, false, font_scale);
}

pub fn live_radio_button_group(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    live_value: f64,
    _display_options: Option<&crate::live_values::LiveValueDisplayOptions>,
) {
    if dashboard_icon_fallback(painter, block, rect, font_scale, live_value) {
        return;
    }
    let palette = widget_palette(block);
    let inner = inner_rect(rect, 0.80);
    let labels = option_labels(block);
    let (fsz, row_h, header_h) = radio_group_metrics(rect, font_scale, labels.len());
    let font = egui::FontId::proportional(fsz);
    let group_name = prop(block, "ButtonGroupName", "Group");
    painter.text(
        Pos2::new(inner.left() + 4.0, inner.top() + 2.0),
        Align2::LEFT_TOP,
        group_name,
        font.clone(),
        palette.text,
    );
    let radio_r = safe_clamp_f32(fsz * 0.32, 3.0, row_h * 0.28);
    let y_start = inner.top() + header_h + 4.0;
    let selected = discrete_live_index(live_value, labels.len());
    for (index, label) in labels.iter().enumerate() {
        let y = y_start + index as f32 * row_h + row_h * 0.4;
        if y + radio_r > inner.bottom() {
            break; // Don't overflow
        }
        let cx = inner.left() + radio_r + 4.0;
        painter.circle_stroke(
            Pos2::new(cx, y),
            radio_r,
            Stroke::new(1.0_f32, palette.border),
        );
        if index == selected {
            painter.circle_filled(Pos2::new(cx, y), radio_r * 0.55, palette.accent);
        }
        painter.text(
            Pos2::new(cx + radio_r + 4.0, y),
            Align2::LEFT_CENTER,
            label,
            font.clone(),
            palette.text,
        );
    }
}

pub fn live_combo_box(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    live_value: f64,
    _display_options: Option<&crate::live_values::LiveValueDisplayOptions>,
) {
    if dashboard_icon_fallback(painter, block, rect, font_scale, live_value) {
        return;
    }
    let palette = widget_palette(block);
    let inner = inner_rect(rect, 0.80);
    let field_h = (inner.height() * 0.4).max(8.0);
    let field = Rect::from_min_max(
        Pos2::new(inner.left(), inner.center().y - field_h / 2.0),
        Pos2::new(inner.right(), inner.center().y + field_h / 2.0),
    );
    painter.rect_filled(field, 3.0, palette.bg_field);
    painter.rect_stroke(
        field,
        3.0,
        Stroke::new(1.0_f32, palette.border),
        egui::StrokeKind::Inside,
    );
    let fsz = font_for_rect(rect, font_scale).min(inner.height() * 0.35);
    let labels = option_labels(block);
    let label_count = labels.len().max(1);
    let label = combo_box_label(block, discrete_live_index(live_value, label_count));
    painter.text(
        Pos2::new(field.left() + 4.0, field.center().y),
        Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(fsz),
        palette.text,
    );
    let arrow_sz = (field_h * 0.3).max(3.0);
    let arrow_cx = field.right() - arrow_sz * 2.0;
    let arrow_cy = field.center().y;
    let pts = vec![
        Pos2::new(arrow_cx - arrow_sz, arrow_cy - arrow_sz * 0.5),
        Pos2::new(arrow_cx + arrow_sz, arrow_cy - arrow_sz * 0.5),
        Pos2::new(arrow_cx, arrow_cy + arrow_sz * 0.5),
    ];
    painter.add(egui::Shape::convex_polygon(pts, palette.text, Stroke::NONE));
}

pub fn live_checkbox(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    live_value: f64,
    _display_options: Option<&crate::live_values::LiveValueDisplayOptions>,
) {
    if dashboard_icon_fallback(painter, block, rect, font_scale, live_value) {
        return;
    }
    let palette = widget_palette(block);
    let inner = inner_rect(rect, 0.80);
    let fsz = font_for_rect(rect, font_scale).min(inner.height() * 0.35);
    let font = egui::FontId::proportional(fsz);
    let label = checkbox_label(block);
    let box_sz = (fsz * 1.1).max(6.0);
    let cx = inner.left() + box_sz / 2.0 + 2.0;
    let cy = inner.center().y;
    let check_rect = Rect::from_center_size(Pos2::new(cx, cy), Vec2::splat(box_sz));
    painter.rect_filled(check_rect, 2.0, palette.bg_field);
    painter.rect_stroke(
        check_rect,
        2.0,
        Stroke::new(1.0_f32, palette.border),
        egui::StrokeKind::Inside,
    );
    painter.text(
        Pos2::new(cx + box_sz / 2.0 + 4.0, cy),
        Align2::LEFT_CENTER,
        &label,
        font,
        palette.text,
    );
    if checkbox_state_from_value(block, live_value) {
        let left = cx - box_sz * 0.28;
        let mid = cx - box_sz * 0.05;
        let right = cx + box_sz * 0.30;
        painter.line_segment(
            [Pos2::new(left, cy), Pos2::new(mid, cy + box_sz * 0.22)],
            Stroke::new(1.5_f32, palette.accent_dark),
        );
        painter.line_segment(
            [
                Pos2::new(mid, cy + box_sz * 0.22),
                Pos2::new(right, cy - box_sz * 0.25),
            ],
            Stroke::new(1.5_f32, palette.accent_dark),
        );
    }
}

pub fn live_slider_or_linear_gauge(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    live_value: f64,
    _display_options: Option<&crate::live_values::LiveValueDisplayOptions>,
) {
    if dashboard_icon_fallback(painter, block, rect, font_scale, live_value) {
        return;
    }
    let palette = widget_palette(block);
    let inner = inner_rect(rect, 0.80);
    let cy = inner.center().y;
    let (scale_min, scale_max) = gauge_range(block);
    let track_h = if block.block_type == "SliderBlock" {
        paint_slider_visual(
            painter,
            block,
            rect,
            palette,
            font_scale,
            normalized_live_value(block, live_value),
        );
        0.0
    } else {
        let fsz = font_for_rect(rect, font_scale).min(inner.height() * 0.18);
        let font = egui::FontId::proportional(fsz);
        let bar_h = (inner.height() * 0.15).clamp(3.0, 10.0);
        let bar = Rect::from_min_max(
            Pos2::new(inner.left(), cy - bar_h / 2.0),
            Pos2::new(inner.right(), cy + bar_h / 2.0),
        );
        painter.rect_filled(bar, 2.0, Color32::from_rgb(220, 220, 225));
        painter.rect_stroke(
            bar,
            2.0,
            Stroke::new(1.0_f32, BORDER),
            egui::StrokeKind::Inside,
        );
        let n_ticks = 11;
        for index in 0..n_ticks {
            let tick_t = index as f32 / (n_ticks - 1) as f32;
            let x = inner.left() + tick_t * inner.width();
            let tick_len = if index % 5 == 0 { 4.0 } else { 2.5 };
            painter.line_segment(
                [
                    Pos2::new(x, bar.bottom() + 1.0),
                    Pos2::new(x, bar.bottom() + 1.0 + tick_len),
                ],
                Stroke::new(1.0_f32, TEXT_DARK),
            );
        }
        let label_y = bar.bottom() + 7.0;
        painter.text(
            Pos2::new(inner.left(), label_y),
            Align2::LEFT_TOP,
            format_scale_value(scale_min),
            font.clone(),
            TEXT_DARK,
        );
        painter.text(
            Pos2::new(inner.right(), label_y),
            Align2::RIGHT_TOP,
            format_scale_value(scale_max),
            font,
            TEXT_DARK,
        );
        bar_h
    };
    if block.block_type == "SliderBlock" {
        return;
    }
    let fraction = normalized_live_value(block, live_value);
    let fill_rect = Rect::from_min_max(
        Pos2::new(inner.left(), cy - track_h / 2.0),
        Pos2::new(inner.left() + inner.width() * fraction, cy + track_h / 2.0),
    );
    painter.rect_filled(fill_rect, 2.0, ACCENT);
    let thumb_x = inner.left() + inner.width() * fraction;
    if block.block_type == "SliderBlock" {
        let thumb_w = (inner.width() * 0.04).clamp(4.0, 10.0);
        let thumb =
            Rect::from_center_size(Pos2::new(thumb_x, cy), Vec2::new(thumb_w, track_h * 4.0));
        painter.rect_filled(thumb, 2.0, ACCENT_DARK);
    } else {
        let tri_sz = track_h * 0.8;
        let pts = vec![
            Pos2::new(thumb_x, cy - track_h / 2.0 - 1.0),
            Pos2::new(thumb_x - tri_sz, cy - track_h / 2.0 - 1.0 - tri_sz),
            Pos2::new(thumb_x + tri_sz, cy - track_h / 2.0 - 1.0 - tri_sz),
        ];
        painter.add(egui::Shape::convex_polygon(pts, ACCENT, Stroke::NONE));
    }
}

pub fn live_field_or_display(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    live_value: f64,
    display_options: Option<&crate::live_values::LiveValueDisplayOptions>,
) {
    if dashboard_icon_fallback(painter, block, rect, font_scale, live_value) {
        return;
    }
    let inner = inner_rect(
        rect,
        if block.block_type == "DisplayBlock" {
            0.85
        } else {
            0.80
        },
    );
    let field_h = if block.block_type == "DisplayBlock" {
        (inner.height() * 0.55).clamp(10.0, 40.0)
    } else {
        (inner.height() * 0.45).clamp(10.0, 30.0)
    };
    let field = Rect::from_min_max(
        Pos2::new(inner.left(), inner.center().y - field_h / 2.0),
        Pos2::new(inner.right(), inner.center().y + field_h / 2.0),
    );
    let fill = if block.block_type == "DisplayBlock" {
        Color32::from_rgb(240, 245, 240)
    } else {
        Color32::TRANSPARENT
    };
    if fill != Color32::TRANSPARENT {
        painter.rect_filled(field, 3.0, fill);
    }
    painter.rect_stroke(
        field,
        3.0,
        Stroke::new(1.0_f32, BORDER),
        egui::StrokeKind::Inside,
    );
    let fsz = font_for_rect(rect, font_scale).min(inner.height() * 0.42);
    painter.text(
        field.center(),
        Align2::CENTER_CENTER,
        format_dashboard_scalar_with_options(live_value, display_options),
        if block.block_type == "DisplayBlock" {
            egui::FontId::monospace(fsz)
        } else {
            egui::FontId::proportional(fsz)
        },
        TEXT_DARK,
    );
}

pub fn live_toggle_switch(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    live_value: f64,
    _display_options: Option<&crate::live_values::LiveValueDisplayOptions>,
) {
    if dashboard_icon_fallback(painter, block, rect, font_scale, live_value) {
        return;
    }
    let palette = widget_palette(block);
    paint_switch_visual(painter, rect, palette, live_value >= 0.5, true, font_scale);
}

pub fn live_radial_gauge(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    live_value: f64,
    _display_options: Option<&crate::live_values::LiveValueDisplayOptions>,
) {
    if dashboard_icon_fallback(painter, block, rect, font_scale, live_value) {
        return;
    }
    let inner = inner_rect(rect, 0.80);
    let fraction = if block.block_type == "RotarySwitchBlock" {
        let labels = option_labels(block);
        let idx = discrete_live_index(live_value, labels.len());
        let denom = labels.len().saturating_sub(1).max(1) as f32;
        idx as f32 / denom
    } else {
        normalized_live_value(block, live_value)
    };
    let (cx, cy, radius, start_angle, end_angle, stroke, color) = match block.block_type.as_str() {
        "KnobBlock" | "RotarySwitchBlock" => (
            inner.center().x,
            inner.center().y + inner.height() * 0.05,
            (inner.width().min(inner.height()) * 0.35).max(8.0),
            5.0 * PI / 4.0,
            -PI / 4.0,
            2.5_f32,
            ACCENT_DARK,
        ),
        "CircularGaugeBlock" => (
            inner.center().x,
            inner.center().y + inner.height() * 0.05,
            (inner.width().min(inner.height()) * 0.40).max(10.0),
            5.0 * PI / 4.0,
            -PI / 4.0,
            2.0_f32,
            NEEDLE_RED,
        ),
        "SemiCircularGaugeBlock" => (
            inner.center().x,
            inner.bottom() - inner.height() * 0.15,
            (inner.width() * 0.40).min(inner.height() * 0.7).max(10.0),
            PI,
            0.0,
            2.0_f32,
            NEEDLE_RED,
        ),
        _ => (
            inner.left() + inner.width() * 0.1,
            inner.bottom() - inner.height() * 0.1,
            (inner.width() * 0.7).min(inner.height() * 0.7).max(10.0),
            PI / 2.0,
            0.0,
            2.0_f32,
            NEEDLE_RED,
        ),
    };
    let fsz = font_for_rect(rect, font_scale).min(inner.height() * 0.12);
    let font = egui::FontId::proportional(fsz);
    let (scale_min, scale_max) = gauge_range(block);
    match block.block_type.as_str() {
        "KnobBlock" => {
            painter.circle_filled(Pos2::new(cx, cy), radius, Color32::from_rgb(220, 220, 225));
            painter.circle_stroke(Pos2::new(cx, cy), radius, Stroke::new(1.5_f32, BORDER));
            painter.circle_filled(
                Pos2::new(cx, cy),
                radius * 0.7,
                Color32::from_rgb(235, 235, 238),
            );
            let tick_r_outer = radius + 4.0;
            let tick_r_inner = radius + 1.0;
            let n_ticks = 11;
            for index in 0..n_ticks {
                let tick_t = index as f32 / (n_ticks - 1) as f32;
                let tick_angle = start_angle + tick_t * (end_angle - start_angle);
                let outer = Pos2::new(
                    cx + tick_r_outer * tick_angle.cos(),
                    cy - tick_r_outer * tick_angle.sin(),
                );
                let inner_p = Pos2::new(
                    cx + tick_r_inner * tick_angle.cos(),
                    cy - tick_r_inner * tick_angle.sin(),
                );
                painter.line_segment([inner_p, outer], Stroke::new(1.0_f32, BORDER));
            }
            let label_r = tick_r_outer + fsz;
            painter.text(
                Pos2::new(
                    cx + label_r * start_angle.cos(),
                    cy - label_r * start_angle.sin(),
                ),
                Align2::CENTER_CENTER,
                format_scale_value(scale_min),
                font.clone(),
                TEXT_DARK,
            );
            painter.text(
                Pos2::new(
                    cx + label_r * end_angle.cos(),
                    cy - label_r * end_angle.sin(),
                ),
                Align2::CENTER_CENTER,
                format_scale_value(scale_max),
                font.clone(),
                TEXT_DARK,
            );
        }
        "RotarySwitchBlock" => {
            painter.circle_filled(Pos2::new(cx, cy), radius, Color32::from_rgb(210, 215, 220));
            painter.circle_stroke(Pos2::new(cx, cy), radius, Stroke::new(1.5_f32, BORDER));
            let labels = option_labels(block);
            let selected = discrete_live_index(live_value, labels.len());
            let mark_r = radius + 4.0;
            let label_r = radius + fsz * 1.2 + 4.0;
            let steps = labels.len().saturating_sub(1).max(1) as f32;
            for (index, label) in labels.iter().enumerate() {
                let tick_t = index as f32 / steps;
                let angle = start_angle + tick_t * (end_angle - start_angle);
                let mark_end = Pos2::new(cx + mark_r * angle.cos(), cy - mark_r * angle.sin());
                let mark_start = Pos2::new(
                    cx + (mark_r - 3.0) * angle.cos(),
                    cy - (mark_r - 3.0) * angle.sin(),
                );
                let mark_color = if index == selected {
                    ACCENT_DARK
                } else {
                    BORDER
                };
                painter.line_segment([mark_start, mark_end], Stroke::new(1.5_f32, mark_color));
                painter.text(
                    Pos2::new(cx + label_r * angle.cos(), cy - label_r * angle.sin()),
                    Align2::CENTER_CENTER,
                    label,
                    font.clone(),
                    TEXT_DARK,
                );
            }
        }
        "CircularGaugeBlock" => {
            painter.circle_stroke(Pos2::new(cx, cy), radius, Stroke::new(2.0_f32, BORDER));
            let n_ticks = 11;
            for index in 0..n_ticks {
                let tick_t = index as f32 / (n_ticks - 1) as f32;
                let tick_angle = start_angle + tick_t * (end_angle - start_angle);
                let is_major = index % 2 == 0;
                let r_in = if is_major { radius - 4.0 } else { radius - 2.5 };
                let p1 = Pos2::new(cx + r_in * tick_angle.cos(), cy - r_in * tick_angle.sin());
                let p2 = Pos2::new(
                    cx + radius * tick_angle.cos(),
                    cy - radius * tick_angle.sin(),
                );
                painter.line_segment(
                    [p1, p2],
                    Stroke::new(if is_major { 1.5_f32 } else { 1.0_f32 }, TEXT_DARK),
                );
                if is_major {
                    let val = scale_min + (scale_max - scale_min) * tick_t as f64;
                    let label_r = radius + fsz * 0.8;
                    painter.text(
                        Pos2::new(
                            cx + label_r * tick_angle.cos(),
                            cy - label_r * tick_angle.sin(),
                        ),
                        Align2::CENTER_CENTER,
                        format_scale_value(val),
                        font.clone(),
                        TEXT_DARK,
                    );
                }
            }
        }
        "SemiCircularGaugeBlock" => {
            let n_ticks = 11;
            for index in 0..n_ticks {
                let tick_t = index as f32 / (n_ticks - 1) as f32;
                let tick_angle = start_angle + tick_t * (end_angle - start_angle);
                let is_major = index % 2 == 0;
                let r_in = if is_major { radius - 4.0 } else { radius - 2.5 };
                let p1 = Pos2::new(cx + r_in * tick_angle.cos(), cy - r_in * tick_angle.sin());
                let p2 = Pos2::new(
                    cx + radius * tick_angle.cos(),
                    cy - radius * tick_angle.sin(),
                );
                painter.line_segment(
                    [p1, p2],
                    Stroke::new(if is_major { 1.5_f32 } else { 1.0_f32 }, TEXT_DARK),
                );
                if is_major {
                    let val = scale_min + (scale_max - scale_min) * tick_t as f64;
                    let label_r = radius + fsz * 0.8;
                    painter.text(
                        Pos2::new(
                            cx + label_r * tick_angle.cos(),
                            cy - label_r * tick_angle.sin(),
                        ),
                        Align2::CENTER_CENTER,
                        format_scale_value(val),
                        font.clone(),
                        TEXT_DARK,
                    );
                }
            }
            painter.line_segment(
                [Pos2::new(cx - radius, cy), Pos2::new(cx + radius, cy)],
                Stroke::new(1.0_f32, BORDER),
            );
        }
        "QuarterGaugeBlock" => {
            let n_ticks = 6;
            for index in 0..n_ticks {
                let tick_t = index as f32 / (n_ticks - 1) as f32;
                let tick_angle = start_angle + tick_t * (end_angle - start_angle);
                let p1 = Pos2::new(
                    cx + (radius - 3.5) * tick_angle.cos(),
                    cy - (radius - 3.5) * tick_angle.sin(),
                );
                let p2 = Pos2::new(
                    cx + radius * tick_angle.cos(),
                    cy - radius * tick_angle.sin(),
                );
                painter.line_segment([p1, p2], Stroke::new(1.5_f32, TEXT_DARK));
                let label_r = radius + fsz * 0.8;
                let val = scale_min + (scale_max - scale_min) * tick_t as f64;
                painter.text(
                    Pos2::new(
                        cx + label_r * tick_angle.cos(),
                        cy - label_r * tick_angle.sin(),
                    ),
                    Align2::CENTER_CENTER,
                    format_scale_value(val),
                    font.clone(),
                    TEXT_DARK,
                );
            }
        }
        _ => {}
    }
    let angle = start_angle + fraction * (end_angle - start_angle);
    let needle_end = Pos2::new(
        cx + (radius * 0.85) * angle.cos(),
        cy - (radius * 0.85) * angle.sin(),
    );
    painter.line_segment([Pos2::new(cx, cy), needle_end], Stroke::new(stroke, color));
    painter.circle_filled(Pos2::new(cx, cy), radius * 0.08, color);
}

pub fn live_rocker_switch(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    live_value: f64,
    _display_options: Option<&crate::live_values::LiveValueDisplayOptions>,
) {
    if dashboard_icon_fallback(painter, block, rect, font_scale, live_value) {
        return;
    }
    let palette = widget_palette(block);
    paint_rocker_switch_visual(painter, rect, palette, live_value >= 0.5, font_scale);
}

pub fn live_lamp(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    live_value: f64,
    _display_options: Option<&crate::live_values::LiveValueDisplayOptions>,
) {
    if dashboard_icon_fallback(painter, block, rect, font_scale, live_value) {
        return;
    }
    let inner = inner_rect(rect, 0.80);
    let radius = (inner.width().min(inner.height()) * 0.35).max(6.0);
    let center = inner.center();
    let color = lamp_color_for_value(block, Some(live_value));
    painter.circle_filled(center, radius, color);
    painter.circle_stroke(center, radius, Stroke::new(1.5_f32, BORDER));
}
