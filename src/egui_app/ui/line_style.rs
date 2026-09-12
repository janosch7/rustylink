//! Stroke width, label and test-point marker of a resolved signal line.

use eframe::egui::{self, Align2, Color32, Pos2, Rect, Stroke, Vec2};
use std::collections::BTreeSet;

pub(super) fn line_has_testpoint(targets: &[crate::connection_targets::ConnectionTarget]) -> bool {
    targets.iter().any(|target| target.testpoint)
}

pub(super) fn resolved_line_label(
    line: &crate::model::Line,
    targets: &[crate::connection_targets::ConnectionTarget],
) -> Option<String> {
    let explicit_name = line
        .name
        .as_ref()
        .map(|name| name.trim())
        .filter(|name| !name.is_empty())
        .map(str::to_string);
    if explicit_name.is_some() {
        return explicit_name;
    }

    if is_bus_line(targets) || is_mux_line(targets) {
        return None;
    }

    targets
        .iter()
        .filter(|target| target.signals_only)
        .filter(|target| {
            !matches!(
                target.origin,
                crate::connection_targets::ConnectionTargetOrigin::Mux
                    | crate::connection_targets::ConnectionTargetOrigin::BusCreator
            )
        })
        .find_map(|target| target.signal_name.clone())
}

pub(super) fn is_bus_line(targets: &[crate::connection_targets::ConnectionTarget]) -> bool {
    let signal_resolves = targets
        .iter()
        .filter_map(|target| match &target.resolve {
            Some(crate::connection_targets::ConnectionTargetResolve::Signal(signal_name)) => {
                Some(signal_name.clone())
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    signal_resolves.len() > 1
}

pub(super) fn is_mux_line(targets: &[crate::connection_targets::ConnectionTarget]) -> bool {
    targets.iter().any(|target| {
        matches!(
            target.origin,
            crate::connection_targets::ConnectionTargetOrigin::Mux
        )
    })
}

pub(super) fn line_stroke_width(
    targets: &[crate::connection_targets::ConnectionTarget],
    is_selected: bool,
) -> f32 {
    let base_width = if is_bus_line(targets) { 3.25 } else { 2.0 };
    if is_selected {
        base_width + 1.5
    } else {
        base_width
    }
}

pub(super) fn line_testpoint_marker_position(points: &[Pos2]) -> Option<Pos2> {
    let start = *points.first()?;
    for next in points.iter().skip(1) {
        let dx = next.x - start.x;
        let dy = next.y - start.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 1.0 {
            let offset = len.min(18.0);
            return Some(Pos2::new(
                start.x + dx / len * offset,
                start.y + dy / len * offset,
            ));
        }
    }
    None
}

pub(super) fn draw_line_testpoint_marker(painter: &egui::Painter, center: Pos2, color: Color32) {
    let marker_rect = Rect::from_center_size(center, Vec2::new(20.0, 12.0));
    painter.rect_filled(
        marker_rect,
        4.0,
        Color32::from_rgba_unmultiplied(255, 255, 255, 224),
    );
    painter.rect_stroke(
        marker_rect,
        4.0,
        Stroke::new(1.0_f32, color),
        egui::StrokeKind::Inside,
    );
    painter.text(
        marker_rect.center(),
        Align2::CENTER_CENTER,
        "TP",
        egui::FontId::proportional(9.0),
        color,
    );
}
