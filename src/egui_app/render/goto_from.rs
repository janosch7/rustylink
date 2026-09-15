//! The Goto/From tag blocks.

#![cfg(feature = "egui")]

use super::icons::fit_font_px;
use crate::model::Block;
use eframe::egui::{self, Color32, Rect, Vec2};

/// Render Goto/From blocks with their GotoTag label instead of port labels.
///
/// Simulink brackets the tag (`[A]`) inside the tag-shaped body.
pub fn render_goto_from_block(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    name_font_factor: f32,
    color: Color32,
) {
    let tag = block
        .properties
        .get("GotoTag")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("A");
    let label = format!("[{tag}]");

    let mut font_size = (rect.height() * 0.6).clamp(10.0, 24.0) * font_scale * name_font_factor;
    let fitted = fit_font_px(painter, &label, rect.size() * Vec2::new(0.72, 0.7));
    font_size = font_size.min(fitted);

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(font_size),
        color,
    );
}
