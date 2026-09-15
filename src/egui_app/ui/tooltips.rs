//! Pointer hit-testing and the hover tooltips of blocks and signal lines.

use crate::egui_app::state::{LiveTooltipEntry, LiveTooltipKind};
use eframe::egui::{self, Color32, RichText};
use std::collections::HashMap;

pub(super) fn show_pointer_tooltip_text(ui: &egui::Ui, id: egui::Id, tooltip: &str) {
    let _ = egui::Tooltip::always_open(
        ui.ctx().clone(),
        ui.layer_id(),
        id,
        egui::PopupAnchor::Pointer,
    )
    .gap(12.0)
    .show(|ui| {
        ui.label(tooltip);
    });
}

/// Returns true if the pointer is currently hovering over any of the given
/// block screen rects.  Used to skip line tooltips when the pointer is
/// actually over a block (blocks are visually on top of lines).
pub(super) fn pointer_over_any_block(
    ui: &egui::Ui,
    sid_screen_map: &HashMap<String, egui::Rect>,
) -> bool {
    let Some(pos) = ui.ctx().pointer_hover_pos() else {
        return false;
    };
    sid_screen_map.values().any(|rect| rect.contains(pos))
}

/// Returns true if the pointer is within `threshold` pixels of any line
/// segment.  Uses the same 8.0px threshold as the click detection.
pub(super) fn pointer_near_any_segment(
    ui: &egui::Ui,
    segments: &[(egui::Pos2, egui::Pos2)],
) -> bool {
    let Some(cp) = ui.ctx().pointer_hover_pos() else {
        return false;
    };
    let threshold = 8.0;
    segments.iter().any(|(a, b)| {
        let ab_x = b.x - a.x;
        let ab_y = b.y - a.y;
        let len_sq = ab_x * ab_x + ab_y * ab_y;
        let t = if len_sq < 1e-6 {
            0.0
        } else {
            ((cp.x - a.x) * ab_x + (cp.y - a.y) * ab_y) / len_sq
        };
        let t = t.clamp(0.0, 1.0);
        let proj_x = a.x + ab_x * t;
        let proj_y = a.y + ab_y * t;
        let dx = cp.x - proj_x;
        let dy = cp.y - proj_y;
        (dx * dx + dy * dy).sqrt() <= threshold
    })
}

pub(super) fn show_pointer_tooltip_entries(
    ui: &egui::Ui,
    id: egui::Id,
    entries: &[LiveTooltipEntry],
) {
    let _ = egui::Tooltip::always_open(
        ui.ctx().clone(),
        ui.layer_id(),
        id,
        egui::PopupAnchor::Pointer,
    )
    .gap(12.0)
    .show(|ui| {
        for entry in entries {
            let (suffix, suffix_color) = match entry.kind {
                LiveTooltipKind::Signal => ("S", Color32::from_rgb(80, 200, 120)),
                LiveTooltipKind::Parameter => ("P", Color32::from_rgb(64, 140, 255)),
            };
            ui.horizontal(|ui| {
                ui.label(&entry.datafield_name);
                ui.colored_label(suffix_color, RichText::new(suffix).strong());
                ui.label(":");
                ui.monospace(&entry.formatted_value);
            });
        }
    });
}
