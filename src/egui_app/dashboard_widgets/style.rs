//! Colors, palettes and the shared geometry of a dashboard widget.

#![cfg(feature = "egui")]

use crate::model::Block;
#[cfg(feature = "dashboard")]
use eframe::egui::Vec2;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke};

/// Standard widget colours matching Simulink's Dashboard palette.
pub(super) const BG_FIELD: Color32 = Color32::from_rgb(255, 255, 255);

pub(super) const BORDER: Color32 = Color32::from_rgb(180, 180, 180);

pub(super) const TEXT_DARK: Color32 = Color32::from_rgb(40, 40, 40);

pub(super) const ACCENT: Color32 = Color32::from_rgb(60, 120, 215);

pub(super) const ACCENT_DARK: Color32 = Color32::from_rgb(40, 80, 180);

pub(super) const NEEDLE_RED: Color32 = Color32::from_rgb(200, 40, 40);

pub(super) const SCOPE_BG: Color32 = Color32::from_rgb(250, 250, 250);

pub(super) const SCOPE_GRID: Color32 = Color32::from_rgb(220, 220, 220);

pub(super) const SCOPE_LINE: Color32 = Color32::from_rgb(30, 100, 200);

#[derive(Clone, Copy)]
pub(super) struct WidgetPalette {
    pub(super) bg_field: Color32,
    pub(super) border: Color32,
    pub(super) text: Color32,
    pub(super) accent: Color32,
    pub(super) accent_dark: Color32,
}

pub(super) fn clamp_u8(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

pub(super) fn color_mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    Color32::from_rgb(
        clamp_u8(a.r() as f32 * inv + b.r() as f32 * t),
        clamp_u8(a.g() as f32 * inv + b.g() as f32 * t),
        clamp_u8(a.b() as f32 * inv + b.b() as f32 * t),
    )
}

fn parse_color_string(raw: &str) -> Option<Color32> {
    let cleaned = raw
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .replace(';', ",");
    let parts: Vec<f32> = cleaned
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<f32>().ok())
        .collect();
    if parts.len() != 3 {
        return None;
    }
    let scale = if parts.iter().all(|v| *v <= 1.0) {
        255.0
    } else {
        1.0
    };
    Some(Color32::from_rgb(
        clamp_u8(parts[0] * scale),
        clamp_u8(parts[1] * scale),
        clamp_u8(parts[2] * scale),
    ))
}

pub(super) fn parse_block_background_color(block: &Block) -> Option<Color32> {
    block
        .background_color
        .as_deref()
        .and_then(parse_color_string)
}

pub(super) fn parse_color_property(block: &Block, keys: &[&str]) -> Option<Color32> {
    keys.iter().find_map(|key| {
        block
            .properties
            .get(*key)
            .and_then(|raw| parse_color_string(raw))
    })
}

pub(super) fn widget_palette(block: &Block) -> WidgetPalette {
    let bg = parse_color_property(block, &["BackgroundColor", "Background"])
        .or_else(|| parse_block_background_color(block))
        .unwrap_or(BG_FIELD);
    let fg = parse_color_property(block, &["ForegroundColor", "Foreground", "TextColor"])
        .unwrap_or(TEXT_DARK);
    let border = color_mix(fg, bg, 0.45);
    let accent = color_mix(ACCENT, fg, 0.35);
    let accent_dark = color_mix(ACCENT_DARK, fg, 0.45);
    WidgetPalette {
        bg_field: bg,
        border,
        text: fg,
        accent,
        accent_dark,
    }
}

/// Paint a thin rounded-rect border (the "widget frame").
pub(super) fn widget_frame(painter: &egui::Painter, rect: Rect, rounding: f32) {
    painter.rect_stroke(
        rect,
        rounding,
        Stroke::new(1.0_f32, BORDER),
        egui::StrokeKind::Inside,
    );
}

/// A small helper: clamp-shrink the rect and compute a font size that fits.
pub(super) fn inner_rect(rect: &Rect, frac: f32) -> Rect {
    let inset_x = rect.width() * (1.0 - frac) * 0.5;
    let inset_y = rect.height() * (1.0 - frac) * 0.5;
    Rect::from_min_max(
        Pos2::new(rect.left() + inset_x, rect.top() + inset_y),
        Pos2::new(rect.right() - inset_x, rect.bottom() - inset_y),
    )
}

pub(super) fn font_for_rect(rect: &Rect, scale: f32) -> f32 {
    (rect.height() * 0.25 * scale).max(4.0)
}

pub(super) fn safe_clamp_f32(value: f32, min: f32, max: f32) -> f32 {
    let low = min.min(max);
    let high = min.max(max);
    if !low.is_finite() || !high.is_finite() {
        return if value.is_finite() { value } else { 0.0 };
    }
    if !value.is_finite() {
        return low;
    }
    value.clamp(low, high)
}

pub(super) fn switch_labels_visible(rect: &Rect, vertical: bool) -> bool {
    if vertical {
        rect.height() >= 56.0 && rect.width() >= 20.0
    } else {
        rect.width() >= 62.0 && rect.height() >= 18.0
    }
}

pub(super) fn should_render_dashboard_icon(rect: &Rect) -> bool {
    rect.width() < 34.0 || rect.height() < 22.0 || rect.area() < 950.0
}

pub(super) fn radio_group_metrics(
    rect: &Rect,
    font_scale: f32,
    option_count: usize,
) -> (f32, f32, f32) {
    let inner = inner_rect(rect, 0.80);
    let rows = option_count.max(1) as f32;
    let row_h = safe_clamp_f32((inner.height() - 6.0) / (rows + 1.05), 6.0, inner.height());
    let width_limited = (inner.width() / 8.5).max(5.0);
    let font_size = safe_clamp_f32(
        (row_h * 0.62).min(width_limited),
        5.0,
        18.0 * font_scale.max(0.7),
    );
    let header_h = safe_clamp_f32(row_h * 0.95, 8.0, row_h + 4.0);
    (font_size, row_h, header_h)
}

#[cfg(feature = "dashboard")]
pub(super) fn apply_dashboard_widget_style_with_body_size(
    ui: &mut egui::Ui,
    body_size: f32,
    palette: WidgetPalette,
) {
    let mut style: egui::Style = ui.style().as_ref().clone();
    style.visuals.override_text_color = Some(palette.text);
    style.visuals.widgets.noninteractive.fg_stroke.color = palette.text;
    style.visuals.widgets.inactive.fg_stroke.color = palette.text;
    style.visuals.widgets.hovered.fg_stroke.color = palette.text;
    style.visuals.widgets.active.fg_stroke.color = palette.text;
    style.visuals.widgets.inactive.bg_fill = palette.bg_field;
    style.visuals.widgets.hovered.bg_fill = palette.bg_field;
    style.visuals.widgets.active.bg_fill = palette.bg_field;
    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::proportional(body_size));
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::proportional(body_size),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::proportional((body_size * 0.85).max(7.0)),
    );
    ui.set_style(style);
}

#[cfg(feature = "dashboard")]
pub(super) fn apply_dashboard_widget_style(
    ui: &mut egui::Ui,
    rect: Rect,
    font_scale: f32,
    palette: WidgetPalette,
) {
    let body_size = (rect.height() * 0.35 * font_scale).clamp(8.0, 40.0);
    apply_dashboard_widget_style_with_body_size(ui, body_size, palette);
}

#[cfg(feature = "dashboard")]
pub(super) fn paint_dashboard_widget_background(
    ui: &mut egui::Ui,
    rect: Rect,
    palette: WidgetPalette,
) {
    if palette.bg_field != BG_FIELD {
        ui.painter()
            .rect_filled(rect.shrink(2.0), 4.0, palette.bg_field);
    }
}

#[cfg(feature = "dashboard")]
pub(super) fn dashboard_knob_geometry(rect: Rect) -> (Pos2, f32) {
    let inner = rect.shrink2(Vec2::new(rect.width() * 0.1, rect.height() * 0.1));
    let center = Pos2::new(inner.center().x, inner.center().y + inner.height() * 0.05);
    let radius = (inner.width().min(inner.height()) * 0.35).max(8.0);
    (center, radius)
}

#[cfg(feature = "dashboard")]
pub(super) fn dashboard_rotary_geometry(rect: Rect) -> (Pos2, f32) {
    let inner = rect.shrink2(Vec2::new(rect.width() * 0.1, rect.height() * 0.1));
    let center = Pos2::new(inner.center().x, inner.center().y + inner.height() * 0.05);
    let radius = (inner.width().min(inner.height()) * 0.30).max(8.0);
    (center, radius)
}

#[cfg(feature = "dashboard")]
pub(super) fn dashboard_arc_fraction(pointer: Pos2, center: Pos2) -> f64 {
    let dx = pointer.x - center.x;
    let dy = center.y - pointer.y;
    let mut angle_deg = dy.atan2(dx).to_degrees();
    if angle_deg < 0.0 {
        angle_deg += 360.0;
    }
    let start_deg = 225.0;
    let clockwise = (start_deg - angle_deg).rem_euclid(360.0);
    if clockwise <= 270.0 {
        (clockwise / 270.0).clamp(0.0, 1.0) as f64
    } else if clockwise < 315.0 {
        1.0
    } else {
        0.0
    }
}
