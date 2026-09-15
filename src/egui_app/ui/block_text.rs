//! Fitting, ellipsizing and formatting the text drawn inside a block.

use crate::egui_app::render::wrap_text_to_max_width;
use eframe::egui::{self, Align2, Color32, Pos2, Rect, Vec2};

pub(crate) fn format_live_scalar_csv(value: f64) -> String {
    let mut text = format!("{value:.6}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    if text == "-0" { "0".to_string() } else { text }
}

pub(super) fn format_constant_value_for_display(raw: &str) -> String {
    let trimmed = raw.trim();
    let Ok(value) = trimmed.parse::<f64>() else {
        return trimmed.to_string();
    };
    if value != 0.0 && value.abs() < 0.1 {
        format!("{value:.2e}")
    } else {
        format!("{value:.2}")
    }
}

pub(super) const MIN_BLOCK_VALUE_FONT_FACTOR: f32 = 0.45;

pub(super) fn ellipsize_text_to_width(
    painter: &egui::Painter,
    text: &str,
    font_id: &egui::FontId,
    max_width: f32,
    color: Color32,
) -> String {
    let measure = |value: &str| {
        painter
            .layout_no_wrap(value.to_string(), font_id.clone(), color)
            .size()
            .x
    };

    if measure(text) <= max_width {
        return text.to_string();
    }

    let ellipsis = "...";
    let mut trimmed = text.to_string();
    while !trimmed.is_empty() {
        trimmed.pop();
        let candidate = format!("{}{}", trimmed.trim_end(), ellipsis);
        if measure(&candidate) <= max_width {
            return candidate;
        }
    }

    ellipsis.to_string()
}

pub(super) fn paint_fitted_centered_text(
    painter: &egui::Painter,
    rect: Rect,
    text: &str,
    color: Color32,
    desired_font_px: f32,
    min_font_px: f32,
    monospace: bool,
) {
    let inner = rect.shrink2(Vec2::new(4.0, 4.0));
    if inner.width() <= 1.0 || inner.height() <= 1.0 || text.trim().is_empty() {
        return;
    }

    let mut current_font_px = desired_font_px.max(min_font_px).max(1.0);

    let (best_lines, best_font_px) = loop {
        let font_id = if monospace {
            egui::FontId::monospace(current_font_px)
        } else {
            egui::FontId::proportional(current_font_px)
        };
        let line_height = (current_font_px * 1.15).max(1.0);
        let max_lines = ((inner.height() / line_height).floor() as usize).max(1);
        let mut lines = wrap_text_to_max_width(painter, text, font_id.clone(), inner.width());
        if lines.is_empty() {
            return;
        }
        if lines.len() > max_lines {
            lines.truncate(max_lines);
        }
        for line in &mut lines {
            *line = ellipsize_text_to_width(painter, line, &font_id, inner.width(), color);
        }

        let total_h = lines.len() as f32 * line_height;
        if total_h <= inner.height() + 0.5 || current_font_px <= min_font_px + f32::EPSILON {
            break (lines, current_font_px);
        }

        let next_font_px = (current_font_px * 0.9).max(min_font_px);
        if (next_font_px - current_font_px).abs() < f32::EPSILON {
            break (lines, current_font_px);
        }
        current_font_px = next_font_px;
    };

    let font_id = if monospace {
        egui::FontId::monospace(best_font_px)
    } else {
        egui::FontId::proportional(best_font_px)
    };
    let line_height = (best_font_px * 1.15).max(1.0);
    let total_h = best_lines.len() as f32 * line_height;
    let mut y = inner.center().y - total_h * 0.5 + line_height * 0.5;
    for line in &best_lines {
        painter.text(
            Pos2::new(inner.center().x, y),
            Align2::CENTER_CENTER,
            line,
            font_id.clone(),
            color,
        );
        y += line_height;
    }
}

pub(super) fn format_block_value_for_display(block: &crate::model::Block, raw: &str) -> String {
    if crate::simulink_libraries::traits::has_editable_value(&block.block_type) {
        raw.trim().to_string()
    } else {
        raw.to_string()
    }
}
