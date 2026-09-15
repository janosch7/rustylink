//! The static (non-interactive) drawing of every dashboard widget.

#![cfg(feature = "egui")]

use super::painters::{
    paint_dashboard_widget_icon, paint_rocker_switch_visual, paint_slider_visual,
    paint_switch_visual,
};
use super::style::{
    ACCENT, ACCENT_DARK, BG_FIELD, BORDER, NEEDLE_RED, SCOPE_BG, SCOPE_GRID, SCOPE_LINE, TEXT_DARK,
    color_mix, font_for_rect, inner_rect, radio_group_metrics, safe_clamp_f32,
    should_render_dashboard_icon, widget_frame, widget_palette,
};
use super::values::{
    checkbox_label, combo_box_label, configured_dashboard_value, format_scale_value, gauge_range,
    lamp_color_for_value, option_labels, prop,
};
use crate::model::Block;
use eframe::egui::{self, Align2, Color32, Pos2, Rect, Stroke, Vec2};
use std::f32::consts::PI;

// ─── PushButton ─────────────────────────────────────────────────────────

/// Draws a push button like Simulink's Dashboard PushButton.
pub fn render_push_button(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    _name_font_factor: f32,
) {
    if should_render_dashboard_icon(rect) {
        paint_dashboard_widget_icon(painter, block, rect, widget_palette(block));
        return;
    }
    let palette = widget_palette(block);
    let inner = inner_rect(rect, 0.85);
    let label = prop(block, "ButtonText", &block.name);
    painter.rect_filled(inner, 4.0, palette.bg_field);
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

// ─── SliderSwitch ───────────────────────────────────────────────────────

/// Draws a vertical slider switch with Off/On labels.
pub fn render_slider_switch(
    painter: &egui::Painter,
    _block: &Block,
    rect: &Rect,
    font_scale: f32,
    _name_font_factor: f32,
) {
    if should_render_dashboard_icon(rect) {
        paint_dashboard_widget_icon(painter, _block, rect, widget_palette(_block));
        return;
    }
    paint_switch_visual(
        painter,
        rect,
        widget_palette(_block),
        false,
        false,
        font_scale,
    );
}

// ─── RadioButton ────────────────────────────────────────────────────────

/// Draws a radio button group with 3 labelled options.
pub fn render_radio_button(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    _name_font_factor: f32,
) {
    if should_render_dashboard_icon(rect) {
        paint_dashboard_widget_icon(painter, block, rect, widget_palette(block));
        return;
    }
    let inner = inner_rect(rect, 0.80);
    let palette = widget_palette(block);
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
    for (i, lbl) in labels.iter().enumerate() {
        let y = y_start + i as f32 * row_h + row_h * 0.4;
        if y + radio_r > inner.bottom() {
            break; // Don't overflow the rect
        }
        let cx = inner.left() + radio_r + 4.0;
        painter.circle_stroke(
            Pos2::new(cx, y),
            radio_r,
            Stroke::new(1.0_f32, palette.border),
        );
        if i == 0 {
            painter.circle_filled(Pos2::new(cx, y), radio_r * 0.55, palette.accent);
        }
        painter.text(
            Pos2::new(cx + radio_r + 6.0, y),
            Align2::LEFT_CENTER,
            lbl,
            font.clone(),
            palette.text,
        );
    }
}

// ─── ComboBox ───────────────────────────────────────────────────────────

/// Draws a combo box / dropdown with a triangle indicator.
pub fn render_combo_box(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    _name_font_factor: f32,
) {
    if should_render_dashboard_icon(rect) {
        paint_dashboard_widget_icon(painter, block, rect, widget_palette(block));
        return;
    }
    let palette = widget_palette(block);
    let inner = inner_rect(rect, 0.80);
    let fsz = font_for_rect(rect, font_scale).min(inner.height() * 0.35);
    let font = egui::FontId::proportional(fsz);

    // Dropdown field
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

    // Label text
    painter.text(
        Pos2::new(field.left() + 4.0, field.center().y),
        Align2::LEFT_CENTER,
        combo_box_label(block, 0),
        font,
        palette.text,
    );

    // Dropdown arrow (triangle)
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

// ─── CheckBox ───────────────────────────────────────────────────────────

/// Draws a checkbox with a label.
pub fn render_checkbox(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    _name_font_factor: f32,
) {
    if should_render_dashboard_icon(rect) {
        paint_dashboard_widget_icon(painter, block, rect, widget_palette(block));
        return;
    }
    let inner = inner_rect(rect, 0.80);
    let fsz = font_for_rect(rect, font_scale).min(inner.height() * 0.35);
    let font = egui::FontId::proportional(fsz);
    let label = checkbox_label(block);

    // Checkbox square
    let box_sz = (fsz * 1.1).max(6.0);
    let cx = inner.left() + box_sz / 2.0 + 2.0;
    let cy = inner.center().y;
    let check_rect = Rect::from_center_size(Pos2::new(cx, cy), Vec2::splat(box_sz));
    painter.rect_filled(check_rect, 2.0, BG_FIELD);
    painter.rect_stroke(
        check_rect,
        2.0,
        Stroke::new(1.0_f32, BORDER),
        egui::StrokeKind::Inside,
    );

    // Label
    painter.text(
        Pos2::new(cx + box_sz / 2.0 + 4.0, cy),
        Align2::LEFT_CENTER,
        &label,
        font,
        TEXT_DARK,
    );
}

// ─── Slider ─────────────────────────────────────────────────────────────

/// Draws a horizontal slider with tick marks and scale.
pub fn render_slider(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    _name_font_factor: f32,
) {
    if should_render_dashboard_icon(rect) {
        paint_dashboard_widget_icon(painter, block, rect, widget_palette(block));
        return;
    }
    paint_slider_visual(painter, block, rect, widget_palette(block), font_scale, 0.5);
}

// ─── EditField ──────────────────────────────────────────────────────────

/// Draws a text edit field.
pub fn render_edit_field(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    _name_font_factor: f32,
) {
    if should_render_dashboard_icon(rect) {
        paint_dashboard_widget_icon(painter, block, rect, widget_palette(block));
        return;
    }
    let palette = widget_palette(block);
    let inner = inner_rect(rect, 0.80);
    let fsz = safe_clamp_f32(
        (inner.height() * 0.48 * font_scale).min(inner.width() * 0.24),
        5.0,
        inner.height() * 0.62,
    );

    // Field rectangle
    let field_h = (inner.height() * 0.45).max(8.0);
    let field = Rect::from_min_max(
        Pos2::new(inner.left(), inner.center().y - field_h / 2.0),
        Pos2::new(inner.right(), inner.center().y + field_h / 2.0),
    );
    painter.rect_stroke(
        field,
        3.0,
        Stroke::new(1.0_f32, palette.border),
        egui::StrokeKind::Inside,
    );

    // Blinking cursor indicator
    let cursor_x = field.left() + 6.0;
    let cursor_top = field.top() + 3.0;
    let cursor_bot = field.bottom() - 3.0;
    painter.line_segment(
        [
            Pos2::new(cursor_x, cursor_top),
            Pos2::new(cursor_x, cursor_bot),
        ],
        Stroke::new(1.0_f32, palette.text),
    );

    if block
        .properties
        .get("ShowInitialText")
        .is_some_and(|value| value.eq_ignore_ascii_case("on"))
    {
        painter.text(
            Pos2::new(field.center().x, field.center().y),
            match block
                .properties
                .get("Alignment")
                .map(|value| value.to_ascii_lowercase())
                .as_deref()
            {
                Some("left") => Align2::LEFT_CENTER,
                Some("right") => Align2::RIGHT_CENTER,
                _ => Align2::CENTER_CENTER,
            },
            "0",
            egui::FontId::proportional(fsz),
            color_mix(palette.text, palette.bg_field, 0.45),
        );
    }
}

// ─── ToggleSwitch ───────────────────────────────────────────────────────

/// Draws a horizontal toggle switch (Off / On).
pub fn render_toggle_switch(
    painter: &egui::Painter,
    _block: &Block,
    rect: &Rect,
    font_scale: f32,
    _name_font_factor: f32,
) {
    if should_render_dashboard_icon(rect) {
        paint_dashboard_widget_icon(painter, _block, rect, widget_palette(_block));
        return;
    }
    paint_switch_visual(
        painter,
        rect,
        widget_palette(_block),
        false,
        true,
        font_scale,
    );
}

// ─── Knob ───────────────────────────────────────────────────────────────

/// Draws a circular knob with tick marks (like Simulink's Knob).
pub fn render_knob(
    painter: &egui::Painter,
    _block: &Block,
    rect: &Rect,
    font_scale: f32,
    _name_font_factor: f32,
) {
    if should_render_dashboard_icon(rect) {
        paint_dashboard_widget_icon(painter, _block, rect, widget_palette(_block));
        return;
    }
    let inner = inner_rect(rect, 0.80);
    let fsz = font_for_rect(rect, font_scale).min(inner.height() * 0.12);
    let font = egui::FontId::proportional(fsz);

    let cx = inner.center().x;
    let cy = inner.center().y + inner.height() * 0.05;
    let radius = (inner.width().min(inner.height()) * 0.35).max(8.0);

    // Knob body (outer ring)
    painter.circle_filled(Pos2::new(cx, cy), radius, Color32::from_rgb(220, 220, 225));
    painter.circle_stroke(Pos2::new(cx, cy), radius, Stroke::new(1.5_f32, BORDER));
    // Inner circle
    painter.circle_filled(
        Pos2::new(cx, cy),
        radius * 0.7,
        Color32::from_rgb(235, 235, 238),
    );

    // Scale ticks (arc from ~225° to ~315° going clockwise = 225° to -45° in standard)
    let start_angle = 5.0 * PI / 4.0; // 225 degrees
    let end_angle = -PI / 4.0; // -45 degrees
    let n_ticks = 11;
    let tick_r_outer = radius + 4.0;
    let tick_r_inner = radius + 1.0;
    for i in 0..n_ticks {
        let t = i as f32 / (n_ticks - 1) as f32;
        let angle = start_angle + t * (end_angle - start_angle);
        let outer = Pos2::new(
            cx + tick_r_outer * angle.cos(),
            cy - tick_r_outer * angle.sin(),
        );
        let inner_p = Pos2::new(
            cx + tick_r_inner * angle.cos(),
            cy - tick_r_inner * angle.sin(),
        );
        painter.line_segment([inner_p, outer], Stroke::new(1.0_f32, BORDER));
    }

    // Needle indicator pointing at ~180° position (left = 0)
    let needle_angle = start_angle; // pointing to "0" at the start
    let needle_end = Pos2::new(
        cx + (radius * 0.6) * needle_angle.cos(),
        cy - (radius * 0.6) * needle_angle.sin(),
    );
    painter.line_segment(
        [Pos2::new(cx, cy), needle_end],
        Stroke::new(2.0_f32, ACCENT_DARK),
    );

    // Scale labels
    let label_r = tick_r_outer + fsz;
    painter.text(
        Pos2::new(
            cx + label_r * start_angle.cos(),
            cy - label_r * start_angle.sin(),
        ),
        Align2::CENTER_CENTER,
        "0",
        font.clone(),
        TEXT_DARK,
    );
    painter.text(
        Pos2::new(
            cx + label_r * end_angle.cos(),
            cy - label_r * end_angle.sin(),
        ),
        Align2::CENTER_CENTER,
        "100",
        font,
        TEXT_DARK,
    );
}

// ─── RockerSwitch ───────────────────────────────────────────────────────

/// Draws a rocker switch (On/Off toggle with a rocker shape).
pub fn render_rocker_switch(
    painter: &egui::Painter,
    _block: &Block,
    rect: &Rect,
    font_scale: f32,
    _name_font_factor: f32,
) {
    if should_render_dashboard_icon(rect) {
        paint_dashboard_widget_icon(painter, _block, rect, widget_palette(_block));
        return;
    }
    paint_rocker_switch_visual(painter, rect, widget_palette(_block), false, font_scale);
}

// ─── RotarySwitch ───────────────────────────────────────────────────────

/// Draws a rotary switch with discrete positions.
pub fn render_rotary_switch(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    _name_font_factor: f32,
) {
    if should_render_dashboard_icon(rect) {
        paint_dashboard_widget_icon(painter, block, rect, widget_palette(block));
        return;
    }
    let inner = inner_rect(rect, 0.80);
    let fsz = font_for_rect(rect, font_scale).min(inner.height() * 0.12);
    let font = egui::FontId::proportional(fsz);

    let cx = inner.center().x;
    let cy = inner.center().y + inner.height() * 0.05;
    let radius = (inner.width().min(inner.height()) * 0.30).max(8.0);

    // Body
    painter.circle_filled(Pos2::new(cx, cy), radius, Color32::from_rgb(210, 215, 220));
    painter.circle_stroke(Pos2::new(cx, cy), radius, Stroke::new(1.5_f32, BORDER));

    // Position marks
    let labels = option_labels(block);
    let mark_r = radius + 4.0;
    let label_r = radius + fsz * 1.2 + 4.0;
    let steps = labels.len().saturating_sub(1).max(1) as f32;
    for (i, lbl) in labels.iter().enumerate() {
        let tick_t = i as f32 / steps;
        let angle = 5.0 * PI / 4.0 + tick_t * (-3.0 * PI / 2.0);
        let mark_end = Pos2::new(cx + mark_r * angle.cos(), cy - mark_r * angle.sin());
        let mark_start = Pos2::new(
            cx + (mark_r - 3.0) * angle.cos(),
            cy - (mark_r - 3.0) * angle.sin(),
        );
        let col = if i == 0 { ACCENT_DARK } else { BORDER };
        painter.line_segment([mark_start, mark_end], Stroke::new(1.5_f32, col));
        painter.text(
            Pos2::new(cx + label_r * angle.cos(), cy - label_r * angle.sin()),
            Align2::CENTER_CENTER,
            lbl,
            font.clone(),
            TEXT_DARK,
        );
    }

    // Pointer at position 0 (Low)
    let pointer_angle = 5.0 * PI / 4.0;
    let pointer_end = Pos2::new(
        cx + (radius * 0.7) * pointer_angle.cos(),
        cy - (radius * 0.7) * pointer_angle.sin(),
    );
    painter.line_segment(
        [Pos2::new(cx, cy), pointer_end],
        Stroke::new(2.5_f32, ACCENT_DARK),
    );
    painter.circle_filled(Pos2::new(cx, cy), radius * 0.15, ACCENT_DARK);
}

// ─── Circular Gauge (full 270°) ─────────────────────────────────────────

/// Draws a full circular gauge (≈270° arc) like Simulink's Gauge block.
pub fn render_circular_gauge(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    _name_font_factor: f32,
) {
    if should_render_dashboard_icon(rect) {
        paint_dashboard_widget_icon(painter, block, rect, widget_palette(block));
        return;
    }
    let (min, max) = gauge_range(block);
    let inner = inner_rect(rect, 0.80);
    let fsz = font_for_rect(rect, font_scale).min(inner.height() * 0.10);
    let font = egui::FontId::proportional(fsz);

    let cx = inner.center().x;
    let cy = inner.center().y + inner.height() * 0.05;
    let radius = (inner.width().min(inner.height()) * 0.40).max(10.0);

    // Arc background
    painter.circle_stroke(Pos2::new(cx, cy), radius, Stroke::new(2.0_f32, BORDER));

    // Scale ticks around the 270° arc (from 225° counter-clockwise to -45°)
    let start_angle = 5.0 * PI / 4.0;
    let end_angle = -PI / 4.0;
    let n_ticks = 11;
    for i in 0..n_ticks {
        let t = i as f32 / (n_ticks - 1) as f32;
        let angle = start_angle + t * (end_angle - start_angle);
        let is_major = i % 2 == 0;
        let r_out = radius;
        let r_in = if is_major { radius - 4.0 } else { radius - 2.5 };
        let p1 = Pos2::new(cx + r_in * angle.cos(), cy - r_in * angle.sin());
        let p2 = Pos2::new(cx + r_out * angle.cos(), cy - r_out * angle.sin());
        painter.line_segment(
            [p1, p2],
            Stroke::new(if is_major { 1.5_f32 } else { 1.0_f32 }, TEXT_DARK),
        );

        // Scale numbers for major ticks
        if is_major {
            let val = min + (max - min) * t as f64;
            let lr = radius + fsz * 0.8;
            painter.text(
                Pos2::new(cx + lr * angle.cos(), cy - lr * angle.sin()),
                Align2::CENTER_CENTER,
                format_scale_value(val),
                font.clone(),
                TEXT_DARK,
            );
        }
    }

    // Needle (pointing to ~40)
    let needle_t = 0.4;
    let needle_angle = start_angle + needle_t * (end_angle - start_angle);
    let needle_end = Pos2::new(
        cx + (radius * 0.85) * needle_angle.cos(),
        cy - (radius * 0.85) * needle_angle.sin(),
    );
    painter.line_segment(
        [Pos2::new(cx, cy), needle_end],
        Stroke::new(2.0_f32, NEEDLE_RED),
    );
    painter.circle_filled(Pos2::new(cx, cy), radius * 0.08, NEEDLE_RED);
}

// ─── SemiCircular Gauge (half gauge) ────────────────────────────────────

/// Draws a semi-circular (180°) gauge.
pub fn render_semi_circular_gauge(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    _name_font_factor: f32,
) {
    if should_render_dashboard_icon(rect) {
        paint_dashboard_widget_icon(painter, block, rect, widget_palette(block));
        return;
    }
    let (min, max) = gauge_range(block);
    let inner = inner_rect(rect, 0.80);
    let fsz = font_for_rect(rect, font_scale).min(inner.height() * 0.12);
    let font = egui::FontId::proportional(fsz);

    let cx = inner.center().x;
    let cy = inner.bottom() - inner.height() * 0.15;
    let radius = (inner.width() * 0.40).min(inner.height() * 0.7).max(10.0);

    // Semi-arc from 180° to 0°
    let start_angle = PI;
    let end_angle = 0.0;
    let n_ticks = 11;
    for i in 0..n_ticks {
        let t = i as f32 / (n_ticks - 1) as f32;
        let angle = start_angle + t * (end_angle - start_angle);
        let is_major = i % 2 == 0;
        let r_out = radius;
        let r_in = if is_major { radius - 4.0 } else { radius - 2.5 };
        let p1 = Pos2::new(cx + r_in * angle.cos(), cy - r_in * angle.sin());
        let p2 = Pos2::new(cx + r_out * angle.cos(), cy - r_out * angle.sin());
        painter.line_segment(
            [p1, p2],
            Stroke::new(if is_major { 1.5_f32 } else { 1.0_f32 }, TEXT_DARK),
        );

        if is_major {
            let val = min + (max - min) * t as f64;
            let lr = radius + fsz * 0.8;
            painter.text(
                Pos2::new(cx + lr * angle.cos(), cy - lr * angle.sin()),
                Align2::CENTER_CENTER,
                format_scale_value(val),
                font.clone(),
                TEXT_DARK,
            );
        }
    }

    // Base line
    painter.line_segment(
        [Pos2::new(cx - radius, cy), Pos2::new(cx + radius, cy)],
        Stroke::new(1.0_f32, BORDER),
    );

    // Needle
    let needle_t = 0.5;
    let needle_angle = start_angle + needle_t * (end_angle - start_angle);
    let needle_end = Pos2::new(
        cx + (radius * 0.85) * needle_angle.cos(),
        cy - (radius * 0.85) * needle_angle.sin(),
    );
    painter.line_segment(
        [Pos2::new(cx, cy), needle_end],
        Stroke::new(2.0_f32, NEEDLE_RED),
    );
    painter.circle_filled(Pos2::new(cx, cy), radius * 0.08, NEEDLE_RED);
}

// ─── Quarter Gauge ──────────────────────────────────────────────────────

/// Draws a quarter-circle (90°) gauge.
pub fn render_quarter_gauge(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    _name_font_factor: f32,
) {
    if should_render_dashboard_icon(rect) {
        paint_dashboard_widget_icon(painter, block, rect, widget_palette(block));
        return;
    }
    let (min, max) = gauge_range(block);
    let inner = inner_rect(rect, 0.80);
    let fsz = font_for_rect(rect, font_scale).min(inner.height() * 0.12);
    let font = egui::FontId::proportional(fsz);

    // Origin at bottom-left of inner rect
    let cx = inner.left() + inner.width() * 0.1;
    let cy = inner.bottom() - inner.height() * 0.1;
    let radius = (inner.width() * 0.7).min(inner.height() * 0.7).max(10.0);

    // Quarter arc from 90° to 0°
    let start_angle = PI / 2.0;
    let end_angle = 0.0;
    let n_ticks = 6;
    for i in 0..n_ticks {
        let t = i as f32 / (n_ticks - 1) as f32;
        let angle = start_angle + t * (end_angle - start_angle);
        let r_out = radius;
        let r_in = radius - 3.5;
        let p1 = Pos2::new(cx + r_in * angle.cos(), cy - r_in * angle.sin());
        let p2 = Pos2::new(cx + r_out * angle.cos(), cy - r_out * angle.sin());
        painter.line_segment([p1, p2], Stroke::new(1.5_f32, TEXT_DARK));

        let val = min + (max - min) * t as f64;
        let lr = radius + fsz * 0.8;
        painter.text(
            Pos2::new(cx + lr * angle.cos(), cy - lr * angle.sin()),
            Align2::CENTER_CENTER,
            format_scale_value(val),
            font.clone(),
            TEXT_DARK,
        );
    }

    // Needle
    let needle_t = 0.3;
    let needle_angle = start_angle + needle_t * (end_angle - start_angle);
    let needle_end = Pos2::new(
        cx + (radius * 0.85) * needle_angle.cos(),
        cy - (radius * 0.85) * needle_angle.sin(),
    );
    painter.line_segment(
        [Pos2::new(cx, cy), needle_end],
        Stroke::new(2.0_f32, NEEDLE_RED),
    );
    painter.circle_filled(Pos2::new(cx, cy), radius * 0.06, NEEDLE_RED);
}

// ─── Linear Gauge ───────────────────────────────────────────────────────

/// Draws a horizontal linear gauge (bar-style).
pub fn render_linear_gauge(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    _name_font_factor: f32,
) {
    if should_render_dashboard_icon(rect) {
        paint_dashboard_widget_icon(painter, block, rect, widget_palette(block));
        return;
    }
    let (min, max) = gauge_range(block);
    let inner = inner_rect(rect, 0.80);
    let fsz = font_for_rect(rect, font_scale).min(inner.height() * 0.18);
    let font = egui::FontId::proportional(fsz);

    // Bar track
    let bar_h = (inner.height() * 0.15).clamp(3.0, 10.0);
    let cy = inner.center().y;
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

    // Scale ticks below bar
    let n_ticks = 11;
    for i in 0..n_ticks {
        let t = i as f32 / (n_ticks - 1) as f32;
        let x = inner.left() + t * inner.width();
        let tick_len = if i % 5 == 0 { 4.0 } else { 2.5 };
        painter.line_segment(
            [
                Pos2::new(x, bar.bottom() + 1.0),
                Pos2::new(x, bar.bottom() + 1.0 + tick_len),
            ],
            Stroke::new(1.0_f32, TEXT_DARK),
        );
    }

    // Scale labels
    let label_y = bar.bottom() + 7.0;
    painter.text(
        Pos2::new(inner.left(), label_y),
        Align2::LEFT_TOP,
        format_scale_value(min),
        font.clone(),
        TEXT_DARK,
    );
    painter.text(
        Pos2::new(inner.right(), label_y),
        Align2::RIGHT_TOP,
        format_scale_value(max),
        font,
        TEXT_DARK,
    );

    // Filled portion (indicator at ~50%)
    let fill_frac = 0.5;
    let fill_rect = Rect::from_min_max(
        Pos2::new(inner.left(), cy - bar_h / 2.0),
        Pos2::new(inner.left() + inner.width() * fill_frac, cy + bar_h / 2.0),
    );
    painter.rect_filled(fill_rect, 2.0, ACCENT);

    // Indicator triangle above bar
    let tri_x = inner.left() + inner.width() * fill_frac;
    let tri_sz = bar_h * 0.8;
    let pts = vec![
        Pos2::new(tri_x, bar.top() - 1.0),
        Pos2::new(tri_x - tri_sz, bar.top() - 1.0 - tri_sz),
        Pos2::new(tri_x + tri_sz, bar.top() - 1.0 - tri_sz),
    ];
    painter.add(egui::Shape::convex_polygon(pts, ACCENT, Stroke::NONE));
}

// ─── Dashboard Scope ────────────────────────────────────────────────────

/// Draws a mini oscilloscope / waveform chart.
pub fn render_dashboard_scope(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    _name_font_factor: f32,
) {
    if should_render_dashboard_icon(rect) {
        paint_dashboard_widget_icon(painter, block, rect, widget_palette(block));
        return;
    }
    let inner = inner_rect(rect, 0.85);
    let fsz = font_for_rect(rect, font_scale).min(inner.height() * 0.10);
    let font = egui::FontId::proportional(fsz);

    // Background
    painter.rect_filled(inner, 2.0, SCOPE_BG);
    widget_frame(painter, inner, 2.0);

    // Grid lines
    let n_h = 4; // horizontal grid lines
    let n_v = 5; // vertical grid lines
    for i in 1..n_h {
        let t = i as f32 / n_h as f32;
        let y = inner.top() + t * inner.height();
        painter.line_segment(
            [Pos2::new(inner.left(), y), Pos2::new(inner.right(), y)],
            Stroke::new(0.5_f32, SCOPE_GRID),
        );
    }
    for i in 1..n_v {
        let t = i as f32 / n_v as f32;
        let x = inner.left() + t * inner.width();
        painter.line_segment(
            [Pos2::new(x, inner.top()), Pos2::new(x, inner.bottom())],
            Stroke::new(0.5_f32, SCOPE_GRID),
        );
    }

    // Axes
    painter.line_segment(
        [
            Pos2::new(inner.left(), inner.bottom()),
            Pos2::new(inner.right(), inner.bottom()),
        ],
        Stroke::new(1.0_f32, TEXT_DARK),
    );
    painter.line_segment(
        [
            Pos2::new(inner.left(), inner.top()),
            Pos2::new(inner.left(), inner.bottom()),
        ],
        Stroke::new(1.0_f32, TEXT_DARK),
    );

    // Y-axis labels
    painter.text(
        Pos2::new(inner.left() - 2.0, inner.top()),
        Align2::RIGHT_TOP,
        "1",
        font.clone(),
        TEXT_DARK,
    );
    painter.text(
        Pos2::new(inner.left() - 2.0, inner.bottom()),
        Align2::RIGHT_BOTTOM,
        "0",
        font.clone(),
        TEXT_DARK,
    );

    // Sine wave trace
    let n_pts = 60;
    let mut points: Vec<Pos2> = Vec::with_capacity(n_pts);
    for i in 0..n_pts {
        let t = i as f32 / (n_pts - 1) as f32;
        let x = inner.left() + t * inner.width();
        let y_val = 0.5 + 0.4 * (t * 4.0 * PI).sin();
        let y = inner.bottom() - y_val * inner.height();
        points.push(Pos2::new(x, y));
    }
    for seg in points.windows(2) {
        painter.line_segment([seg[0], seg[1]], Stroke::new(1.5_f32, SCOPE_LINE));
    }

    // X-axis labels
    let x_label_y = inner.bottom() + 2.0;
    painter.text(
        Pos2::new(inner.left(), x_label_y),
        Align2::LEFT_TOP,
        "0",
        font.clone(),
        TEXT_DARK,
    );
    let x_max = ((n_pts as f32) * 0.8).round() as i32;
    painter.text(
        Pos2::new(inner.right(), x_label_y),
        Align2::RIGHT_TOP,
        format!("{}", x_max),
        font,
        TEXT_DARK,
    );
}

// ─── Display (Dashboard) ────────────────────────────────────────────────

/// Draws a digital display block (value readout).
pub fn render_display_block(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    _name_font_factor: f32,
) {
    if should_render_dashboard_icon(rect) {
        paint_dashboard_widget_icon(painter, block, rect, widget_palette(block));
        return;
    }
    let inner = inner_rect(rect, 0.85);
    let fsz = font_for_rect(rect, font_scale).min(inner.height() * 0.50);

    // Display field (dark background, LCD-like)
    let field_h = (inner.height() * 0.55).clamp(10.0, 40.0);
    let field = Rect::from_min_max(
        Pos2::new(inner.left(), inner.center().y - field_h / 2.0),
        Pos2::new(inner.right(), inner.center().y + field_h / 2.0),
    );
    painter.rect_filled(field, 3.0, Color32::from_rgb(240, 245, 240));
    painter.rect_stroke(
        field,
        3.0,
        Stroke::new(1.0_f32, BORDER),
        egui::StrokeKind::Inside,
    );

    // Value text
    painter.text(
        field.center(),
        Align2::CENTER_CENTER,
        "0",
        egui::FontId::monospace(fsz),
        TEXT_DARK,
    );
}

// ─── Lamp ───────────────────────────────────────────────────────────────

/// Draws a circular lamp indicator (green by default).
pub fn render_lamp(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    _font_scale: f32,
    _name_font_factor: f32,
) {
    if should_render_dashboard_icon(rect) {
        paint_dashboard_widget_icon(painter, block, rect, widget_palette(block));
        return;
    }
    let inner = inner_rect(rect, 0.80);
    let radius = (inner.width().min(inner.height()) * 0.35).max(6.0);
    let cx = inner.center().x;
    let cy = inner.center().y;

    // Lamp body (glowing circle)
    painter.circle_filled(
        Pos2::new(cx, cy),
        radius,
        lamp_color_for_value(block, configured_dashboard_value(block)),
    );
    painter.circle_stroke(Pos2::new(cx, cy), radius, Stroke::new(1.5_f32, BORDER));

    // Highlight (light reflection)
    let highlight_r = radius * 0.3;
    let hx = cx - radius * 0.2;
    let hy = cy - radius * 0.2;
    painter.circle_filled(
        Pos2::new(hx, hy),
        highlight_r,
        Color32::from_rgba_premultiplied(255, 255, 255, 100),
    );
}
