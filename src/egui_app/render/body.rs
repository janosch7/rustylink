//! Filling and stroking the body shape of a block.

#![cfg(feature = "egui")]

use crate::block_types::{self};
use eframe::egui::{self, Color32, Mesh, Pos2, Rect, Stroke};

/// Fill a block body according to its [`BlockShape`], returning the background
/// color actually painted (commented blocks use a fixed grey).  Shared by the
/// editor and the viewer so both render identical block bodies — there is no
/// per-block-type body-drawing code in either UI.
pub fn fill_block_body(
    painter: &egui::Painter,
    rect: Rect,
    shape: block_types::BlockShape,
    bg: Color32,
    commented: bool,
) -> Color32 {
    use block_types::BlockShape;
    if commented {
        let commented_bg = Color32::from_rgb(230, 230, 230);
        painter.rect_filled(rect, 0.0, commented_bg);
        return commented_bg;
    }
    match shape {
        BlockShape::Triangle => {
            // Gain-style: right-pointing triangle (left-top, right-center, left-bottom).
            let pts = vec![
                egui::pos2(rect.left(), rect.top()),
                egui::pos2(rect.right(), rect.center().y),
                egui::pos2(rect.left(), rect.bottom()),
            ];
            let mut tri = egui::epaint::PathShape::closed_line(pts, Stroke::NONE);
            tri.fill = bg;
            painter.add(egui::Shape::Path(tri));
        }
        BlockShape::Circle => {
            let radius = rect.size().min_elem() / 2.0;
            painter.circle_filled(rect.center(), radius, bg);
        }
        BlockShape::FilledBlack => {
            painter.rect_filled(rect, 0.0, Color32::BLACK);
        }
        BlockShape::Goto => {
            let tab = rect.height() * 0.25;
            let pts = vec![
                egui::pos2(rect.left(), rect.top()),
                egui::pos2(rect.right(), rect.top()),
                egui::pos2(rect.right(), rect.bottom()),
                egui::pos2(rect.left(), rect.bottom()),
                egui::pos2(rect.left() - tab, rect.center().y),
            ];
            let mut path = egui::epaint::PathShape::closed_line(pts, Stroke::NONE);
            path.fill = bg;
            painter.add(egui::Shape::Path(path));
        }
        BlockShape::From => {
            let tab = rect.height() * 0.25;
            let pts = vec![
                egui::pos2(rect.left(), rect.top()),
                egui::pos2(rect.right(), rect.top()),
                egui::pos2(rect.right() + tab, rect.center().y),
                egui::pos2(rect.right(), rect.bottom()),
                egui::pos2(rect.left(), rect.bottom()),
            ];
            let mut path = egui::epaint::PathShape::closed_line(pts, Stroke::NONE);
            path.fill = bg;
            painter.add(egui::Shape::Path(path));
        }
        BlockShape::Rectangle => {
            painter.rect_filled(rect, 6.0, bg);
        }
        BlockShape::Obround => {
            // Fully rounded short ends (egui clamps the corner radius to half the
            // shortest side, giving a stadium/obround).
            painter.rect_filled(rect, rect.height() * 0.5, bg);
        }
        BlockShape::TrapezoidRight => {
            let pts = trapezoid_right_points(rect);
            let mut path = egui::epaint::PathShape::closed_line(pts, Stroke::NONE);
            path.fill = bg;
            painter.add(egui::Shape::Path(path));
        }
        BlockShape::TrapezoidLeft => {
            let pts = trapezoid_left_points(rect);
            let mut path = egui::epaint::PathShape::closed_line(pts, Stroke::NONE);
            path.fill = bg;
            painter.add(egui::Shape::Path(path));
        }
        BlockShape::TrapezoidStemRight => {
            let pts = trapezoid_stem_right_points(rect);
            let mut path = egui::epaint::PathShape::closed_line(pts, Stroke::NONE);
            path.fill = bg;
            painter.add(egui::Shape::Path(path));
        }
        BlockShape::TrapezoidStemLeft => {
            let pts = trapezoid_stem_left_points(rect);
            let mut path = egui::epaint::PathShape::closed_line(pts, Stroke::NONE);
            path.fill = bg;
            painter.add(egui::Shape::Path(path));
        }
        BlockShape::None => {
            // The block's static renderer paints its own body; nothing here.
        }
    }
    bg
}

/// Stroke a block body's outline according to its [`BlockShape`].  Shared by the
/// editor and the viewer.
pub fn stroke_block_body(
    painter: &egui::Painter,
    rect: Rect,
    shape: block_types::BlockShape,
    stroke: Stroke,
) {
    use block_types::BlockShape;
    match shape {
        BlockShape::Triangle => {
            let pts = vec![
                egui::pos2(rect.left(), rect.top()),
                egui::pos2(rect.right(), rect.center().y),
                egui::pos2(rect.left(), rect.bottom()),
            ];
            painter.add(egui::Shape::Path(egui::epaint::PathShape::closed_line(
                pts, stroke,
            )));
        }
        BlockShape::Circle => {
            let radius = rect.size().min_elem() / 2.0;
            painter.circle_stroke(rect.center(), radius, stroke);
        }
        BlockShape::FilledBlack => {}
        BlockShape::Goto => {
            let tab = rect.height() * 0.25;
            let pts = vec![
                egui::pos2(rect.left(), rect.top()),
                egui::pos2(rect.right(), rect.top()),
                egui::pos2(rect.right(), rect.bottom()),
                egui::pos2(rect.left(), rect.bottom()),
                egui::pos2(rect.left() - tab, rect.center().y),
            ];
            painter.add(egui::Shape::Path(egui::epaint::PathShape::closed_line(
                pts, stroke,
            )));
        }
        BlockShape::From => {
            let tab = rect.height() * 0.25;
            let pts = vec![
                egui::pos2(rect.left(), rect.top()),
                egui::pos2(rect.right(), rect.top()),
                egui::pos2(rect.right() + tab, rect.center().y),
                egui::pos2(rect.right(), rect.bottom()),
                egui::pos2(rect.left(), rect.bottom()),
            ];
            painter.add(egui::Shape::Path(egui::epaint::PathShape::closed_line(
                pts, stroke,
            )));
        }
        BlockShape::Rectangle => {
            painter.rect_stroke(rect, 4.0, stroke, egui::StrokeKind::Inside);
        }
        BlockShape::Obround => {
            painter.rect_stroke(rect, rect.height() * 0.5, stroke, egui::StrokeKind::Inside);
        }
        BlockShape::TrapezoidRight => {
            let pts = trapezoid_right_points(rect);
            painter.add(egui::Shape::Path(egui::epaint::PathShape::closed_line(
                pts, stroke,
            )));
        }
        BlockShape::TrapezoidLeft => {
            let pts = trapezoid_left_points(rect);
            painter.add(egui::Shape::Path(egui::epaint::PathShape::closed_line(
                pts, stroke,
            )));
        }
        BlockShape::TrapezoidStemRight => {
            let pts = trapezoid_stem_right_points(rect);
            painter.add(egui::Shape::Path(egui::epaint::PathShape::closed_line(
                pts, stroke,
            )));
        }
        BlockShape::TrapezoidStemLeft => {
            let pts = trapezoid_stem_left_points(rect);
            painter.add(egui::Shape::Path(egui::epaint::PathShape::closed_line(
                pts, stroke,
            )));
        }
        BlockShape::None => {
            // The block's static renderer paints its own outline; nothing here.
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Trapezoid geometry helpers for VariantStart/End/Sink/Source blocks.
//
// TrapezoidRight/Left use 30° sloped sides (shallower than 45°).
// The narrow side height is `H - 2*W*tan(30°)`, clamped to ≥ 0.
//
// TrapezoidStemRight/Left have a rectangular section on the WIDE side
// (top + bottom horizontal bars + vertical bar), then 45° taper to the
// narrow side.  The rectangle takes 1/3 of the width.
// ────────────────────────────────────────────────────────────────────────────

/// tan(30°) — shallower slope for VariantStart/End trapezoids.
const TAN_30: f32 = 0.57735;

/// Trapezoid wide on the right, narrow on the left (VariantStart).
/// 30° sloped sides.
fn trapezoid_right_points(rect: Rect) -> Vec<egui::Pos2> {
    let w = rect.width();
    let h = rect.height();
    let cy = rect.center().y;
    let narrow_half = ((h - 2.0 * w * TAN_30) / 2.0).max(0.0);
    vec![
        egui::pos2(rect.left(), cy - narrow_half),
        egui::pos2(rect.right(), rect.top()),
        egui::pos2(rect.right(), rect.bottom()),
        egui::pos2(rect.left(), cy + narrow_half),
    ]
}

/// Trapezoid wide on the left, narrow on the right (VariantEnd).
/// 30° sloped sides.
fn trapezoid_left_points(rect: Rect) -> Vec<egui::Pos2> {
    let w = rect.width();
    let h = rect.height();
    let cy = rect.center().y;
    let narrow_half = ((h - 2.0 * w * TAN_30) / 2.0).max(0.0);
    vec![
        egui::pos2(rect.left(), rect.top()),
        egui::pos2(rect.right(), cy - narrow_half),
        egui::pos2(rect.right(), cy + narrow_half),
        egui::pos2(rect.left(), rect.bottom()),
    ]
}

/// Trapezoid with rectangular section on the wide right side, then 45°
/// taper to narrow left (VariantSink).
///
/// ```text
///    _____
///   /     |
///         |
///         |
///   \_____|
/// ```
fn trapezoid_stem_right_points(rect: Rect) -> Vec<egui::Pos2> {
    let w = rect.width();
    let h = rect.height();
    let cy = rect.center().y;
    let stem_w = w / 3.0;
    let taper_w = w - stem_w;
    let narrow_half = ((h - 2.0 * taper_w) / 2.0).max(0.0);
    vec![
        egui::pos2(rect.left(), cy - narrow_half),
        egui::pos2(rect.right() - stem_w, rect.top()),
        egui::pos2(rect.right(), rect.top()),
        egui::pos2(rect.right(), rect.bottom()),
        egui::pos2(rect.right() - stem_w, rect.bottom()),
        egui::pos2(rect.left(), cy + narrow_half),
    ]
}

/// Trapezoid with rectangular section on the wide left side, then 45°
/// taper to narrow right (VariantSource).
///
/// ```text
/// _____
/// |       \
/// |        |
/// |        |
/// |____/
/// ```
fn trapezoid_stem_left_points(rect: Rect) -> Vec<egui::Pos2> {
    let w = rect.width();
    let h = rect.height();
    let cy = rect.center().y;
    let stem_w = w / 3.0;
    let taper_w = w - stem_w;
    let narrow_half = ((h - 2.0 * taper_w) / 2.0).max(0.0);
    vec![
        egui::pos2(rect.left(), rect.top()),
        egui::pos2(rect.left() + stem_w, rect.top()),
        egui::pos2(rect.right(), cy - narrow_half),
        egui::pos2(rect.right(), cy + narrow_half),
        egui::pos2(rect.left() + stem_w, rect.bottom()),
        egui::pos2(rect.left(), rect.bottom()),
    ]
}

/// Triangulate a (possibly concave) simple polygon via ear-clipping and
/// return a filled [`Mesh`].  Used by the `pf` command for the stepped band
/// shapes of the Check Dynamic block icons, which are concave and therefore
/// cannot use `Shape::convex_polygon`.
pub(super) fn concave_polygon_mesh(pts: &[Pos2], fill: Color32) -> Mesh {
    let mut mesh = Mesh::default();
    // Ensure CCW winding (in screen coords, y-down).  The ear-clipper below
    // expects CCW so that `cross > 0` identifies convex ("ear") vertices.
    // Signed area: positive = CW (screen coords), negative = CCW.
    let signed_area: f32 = pts
        .windows(2)
        .map(|w| (w[1].x - w[0].x) * (w[1].y + w[0].y))
        .sum::<f32>()
        + (pts[0].x - pts[pts.len() - 1].x) * (pts[0].y + pts[pts.len() - 1].y);
    let pts: Vec<Pos2> = if signed_area > 0.0 {
        pts.iter().rev().copied().collect()
    } else {
        pts.to_vec()
    };
    for &p in &pts {
        mesh.vertices.push(egui::epaint::Vertex {
            pos: p,
            uv: (0.0, 0.0).into(),
            color: fill,
        });
    }
    // Ear-clipping: repeatedly find an "ear" (a triangle whose interior is
    // inside the polygon and contains no other vertex) and clip it.
    let mut indices: Vec<usize> = (0..pts.len()).collect();
    let mut guard = 0;
    while indices.len() > 2 && guard < 10_000 {
        guard += 1;
        let n = indices.len();
        let mut clipped = false;
        for i in 0..n {
            let ia = indices[(i + n - 1) % n];
            let ib = indices[i];
            let ic = indices[(i + 1) % n];
            let a = pts[ia];
            let b = pts[ib];
            let c = pts[ic];
            // Cross product of (b-a) x (c-b); positive = left turn (CCW ear).
            let cross = (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x);
            if cross <= 0.0 {
                continue; // reflex or degenerate vertex
            }
            // Check no other vertex is inside this triangle.
            let inside = |p: Pos2| -> bool {
                let d1 = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
                let d2 = (c.x - b.x) * (p.y - b.y) - (c.y - b.y) * (p.x - b.x);
                let d3 = (a.x - c.x) * (p.y - c.y) - (a.y - c.y) * (p.x - c.x);
                d1 >= 0.0 && d2 >= 0.0 && d3 >= 0.0
            };
            let mut ok = true;
            for &j in &indices {
                if j == ia || j == ib || j == ic {
                    continue;
                }
                if inside(pts[j]) {
                    ok = false;
                    break;
                }
            }
            if ok {
                mesh.indices.extend([ia as u32, ib as u32, ic as u32]);
                indices.remove(i);
                clipped = true;
                break;
            }
        }
        if !clipped {
            // Fallback: fan triangulation from vertex 0.
            for i in 1..indices.len() - 1 {
                mesh.indices
                    .extend([indices[0] as u32, indices[i] as u32, indices[i + 1] as u32]);
            }
            break;
        }
    }
    mesh
}

/// Resolved body colors (fill / outline / interior text) passed to the
/// self-painting metadata-aware renderers (Sum, Logic, Product …).
#[derive(Clone, Copy)]
pub struct BodyColors {
    pub fill: Color32,
    pub border: Color32,
    pub text: Color32,
}
