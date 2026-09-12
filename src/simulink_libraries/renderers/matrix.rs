//! Icon renderers for the matrix and concatenation blocks.
//!
//! Part of the renderer set: one module per Simulink library group.

#![cfg(feature = "egui")]

use crate::model::Block;
use crate::simulink_libraries::types::RenderContext;
use eframe::egui::{Painter, Rect};

/// Static renderer for the Matrix Concatenate block: two cuboids offset
/// diagonally (one back-right, one front-left) with shaded faces and the
/// `ConcatenateDimension` they are joined along printed in the front cuboid.
pub fn static_matrix_concatenate(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let raw = ctx
        .metadata
        .get("ConcatenateDimension")
        .unwrap_or("2")
        .trim();
    let dim = if raw.is_empty() { "2" } else { raw };
    // Two cuboids seen from the front-left, the back one offset up-right and
    // the front one offset down-left, joined where they overlap.  Faces are
    // filled with the SVG's semi-transparent greys; the L-shaped front face of
    // the back cuboid is split into two convex rectangles.
    let spec = format!(
        concat!(
            // ── Back cuboid (drawn first, behind) ──────────────────────────
            // Top face (light grey).
            "pg 235,235,235,128 0.265,0.167 0.412,0 1.0,0 0.853,0.167;",
            // Right side face (darker grey).
            "pg 180,180,180,128 0.853,0.7 0.853,0.167 1.0,0 1.0,0.533;",
            // Front face (white) – L-shape split into two convex rectangles,
            // filled without stroke (`pf`) so the internal seam is invisible;
            // the external boundary is traced separately by the `p` command.
            "pf 255,255,255,128 0.265,0.167 0.853,0.167 0.853,0.3 0.265,0.3;",
            "pf 255,255,255,128 0.735,0.3 0.853,0.3 0.853,0.7 0.735,0.7;",
            // L-shape external outline (closed polyline).
            "p 0.265,0.167 0.853,0.167 0.853,0.7 0.735,0.7 0.735,0.3 0.265,0.3 0.265,0.167;",
            // ── Front cuboid (drawn second, in front) ───────────────────────
            // Top face (light grey).
            "pg 235,235,235,128 0,0.467 0.147,0.3 0.735,0.3 0.588,0.467;",
            // Right side face (darker grey).
            "pg 180,180,180,128 0.588,1.0 0.588,0.467 0.735,0.3 0.735,0.833;",
            // Front face (white).
            "pg 255,255,255,128 0,0.467 0.588,0.467 0.588,1.0 0,1.0;",
            // ── Dashed edge connectors (faint) ──────────────────────────────
            "a 0.147,0.3 0.265,0.167;",
            "a 0.735,0.833 0.853,0.7;",
            "a 0.735,0.3 0.853,0.167;",
            // ── Concatenation dimension in the front cuboid's face ───────────
            "t 0.29,0.73,0.22 {dim}"
        ),
        dim = dim
    );
    crate::egui_app::render::draw_plot_icon(
        painter,
        rect,
        ctx.font_scale,
        &spec,
        ctx.text_color,
        ctx.port_label_widths,
    );
    true
}

/// Static renderer for the Is Triangular block: the diagonal of a square with
/// the tested triangularity beside it (`Upper` → `U`, `Lower` → `L`).
pub fn static_is_triangular(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let lower = ctx
        .metadata
        .get("Mode")
        .is_some_and(|v| v.trim().eq_ignore_ascii_case("lower"));
    let spec = if lower {
        "r 0.10,0.10 0.90,0.90; p 0.10,0.10 0.90,0.90; t 0.32,0.68,0.34 L"
    } else {
        "r 0.10,0.10 0.90,0.90; p 0.10,0.10 0.90,0.90; t 0.68,0.32,0.34 U"
    };
    crate::egui_app::render::draw_plot_icon(
        painter,
        rect,
        ctx.font_scale,
        spec,
        ctx.text_color,
        ctx.port_label_widths,
    );
    true
}

/// Static renderer for the Concatenate block.
///
/// `Mode` picks the pictogram: multidimensional-array concatenation is drawn
/// as two joined cuboids labelled with `ConcatenateDimension`, while vector
/// and matrix concatenation are the stacked slabs of the incoming signals.
pub fn static_concatenate(
    painter: &Painter,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let multidimensional = ctx
        .metadata
        .get("Mode")
        .is_some_and(|m| m.to_lowercase().contains("multidimensional"));
    if multidimensional {
        return static_matrix_concatenate(painter, block, rect, ctx);
    }
    let inputs = block
        .port_counts
        .as_ref()
        .and_then(|c| c.ins)
        .unwrap_or(2)
        .clamp(2, 6);
    let mut spec = String::new();
    for i in 1..inputs {
        let y = i as f32 / inputs as f32;
        spec.push_str(&format!("p 0.0,{y:.3} 1.0,{y:.3};"));
    }
    crate::egui_app::render::draw_plot_icon(
        painter,
        rect,
        ctx.font_scale,
        &spec,
        ctx.text_color,
        None,
    );
    true
}
