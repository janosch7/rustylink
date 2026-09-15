//! Logic gates drawn as the classic AND/OR/XOR/NOT outlines.

#![cfg(feature = "egui")]

use super::body::BodyColors;
use eframe::egui::{self, Align2, Pos2, Rect, Stroke};

/// Draw a Logic (Logical Operator) block.
///
/// `icon_shape` selects between the rectangular text box (`"rectangular"`,
/// Simulink's default) and the distinctive IEEE gate symbol (`"distinctive"`).
/// `operator` selects the gate (AND/OR/NOT/NAND/NOR/XOR/NXOR).  The block owns
/// its whole body (shape [`crate::simulink_libraries::types::SimulinkShape::None`]).
pub fn render_logic_block(
    painter: &egui::Painter,
    rect: &Rect,
    font_scale: f32,
    operator: &str,
    icon_shape: &str,
    colors: BodyColors,
) {
    let stroke = Stroke::new((1.6 * font_scale).clamp(1.0, 3.0), colors.border);
    let op = operator.trim().to_uppercase();

    if !icon_shape.eq_ignore_ascii_case("distinctive") {
        painter.rect_filled(*rect, 4.0, colors.fill);
        painter.rect_stroke(*rect, 4.0, stroke, egui::StrokeKind::Inside);
        let label = if op.is_empty() { "AND".to_string() } else { op };
        let font_size = (rect.height() * 0.30).clamp(7.0, 20.0) * font_scale;
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(font_size),
            colors.text,
        );
        return;
    }

    let (base, negated) = match op.as_str() {
        "NAND" => (GateBase::And, true),
        "NOR" => (GateBase::Or, true),
        "NXOR" | "XNOR" => (GateBase::Xor, true),
        "XOR" => (GateBase::Xor, false),
        "OR" => (GateBase::Or, false),
        "NOT" => (GateBase::Not, true),
        _ => (GateBase::And, false),
    };

    let bubble_r = (rect.height() * 0.10).clamp(2.0, 6.0);
    let body = if negated {
        Rect::from_min_max(
            rect.min,
            Pos2::new(rect.right() - 2.0 * bubble_r, rect.bottom()),
        )
    } else {
        *rect
    };

    let (points, extra_arc) = match base {
        GateBase::And => (and_gate_path(&body), None),
        GateBase::Or => (or_gate_path(&body, false), None),
        GateBase::Xor => (or_gate_path(&body, false), Some(xor_back_arc(&body))),
        GateBase::Not => (not_gate_path(&body), None),
    };

    painter.add(egui::Shape::Path(egui::epaint::PathShape {
        points,
        closed: true,
        fill: colors.fill,
        stroke: stroke.into(),
    }));
    if let Some(arc) = extra_arc {
        painter.add(egui::Shape::line(arc, stroke));
    }
    if negated {
        let c = Pos2::new(body.right() + bubble_r, body.center().y);
        painter.circle(c, bubble_r, colors.fill, stroke);
    }
}

#[derive(Clone, Copy)]
enum GateBase {
    And,
    Or,
    Xor,
    Not,
}

fn quad_bezier(p0: Pos2, ctrl: Pos2, p1: Pos2, n: usize) -> Vec<Pos2> {
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            Pos2::new(
                u * u * p0.x + 2.0 * u * t * ctrl.x + t * t * p1.x,
                u * u * p0.y + 2.0 * u * t * ctrl.y + t * t * p1.y,
            )
        })
        .collect()
}

/// D-shaped AND gate outline (flat left/top/bottom, semicircular right).
fn and_gate_path(r: &Rect) -> Vec<Pos2> {
    let (l, rt, t, b, cy, h) = (
        r.left(),
        r.right(),
        r.top(),
        r.bottom(),
        r.center().y,
        r.height(),
    );
    let rad = h * 0.5;
    let flat_x = (rt - rad).max(l + r.width() * 0.15);
    let center = Pos2::new(flat_x, cy);
    let mut pts = vec![Pos2::new(l, t), Pos2::new(flat_x, t)];
    let n = 16;
    for i in 0..=n {
        let th = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * (i as f32 / n as f32);
        pts.push(Pos2::new(
            center.x + rad * th.cos(),
            center.y + rad * th.sin(),
        ));
    }
    pts.push(Pos2::new(l, b));
    pts
}

/// Curved OR gate outline (pointed right, concave left back edge).
fn or_gate_path(r: &Rect, _xor: bool) -> Vec<Pos2> {
    let (l, rt, t, b, cy, w) = (
        r.left(),
        r.right(),
        r.top(),
        r.bottom(),
        r.center().y,
        r.width(),
    );
    let tip = Pos2::new(rt, cy);
    let mut pts = Vec::new();
    // Top edge: top-left → tip.
    pts.extend(quad_bezier(
        Pos2::new(l, t),
        Pos2::new(l + w * 0.60, t),
        tip,
        14,
    ));
    // Bottom edge: tip → bottom-left.
    pts.extend(quad_bezier(
        tip,
        Pos2::new(l + w * 0.60, b),
        Pos2::new(l, b),
        14,
    ));
    // Back (left) concave edge: bottom-left → top-left, bulging right.
    pts.extend(quad_bezier(
        Pos2::new(l, b),
        Pos2::new(l + w * 0.22, cy),
        Pos2::new(l, t),
        12,
    ));
    pts
}

/// The extra concave back stroke drawn to the left of an XOR gate.
fn xor_back_arc(r: &Rect) -> Vec<Pos2> {
    let (l, t, b, cy, w) = (r.left(), r.top(), r.bottom(), r.center().y, r.width());
    let x = l - w * 0.12;
    quad_bezier(
        Pos2::new(x, t),
        Pos2::new(x + w * 0.22, cy),
        Pos2::new(x, b),
        12,
    )
}

/// Right-pointing triangle for the NOT (buffer) gate; the inversion bubble is
/// drawn separately by the caller.
fn not_gate_path(r: &Rect) -> Vec<Pos2> {
    vec![
        Pos2::new(r.left(), r.top()),
        Pos2::new(r.right(), r.center().y),
        Pos2::new(r.left(), r.bottom()),
    ]
}
