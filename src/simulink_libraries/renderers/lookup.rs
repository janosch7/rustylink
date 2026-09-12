//! Icon renderers for the Lookup Tables blocks.
//!
//! Part of the renderer set: one module per Simulink library group.

#![cfg(feature = "egui")]

use crate::model::Block;
use crate::simulink_libraries::types::RenderContext;
use eframe::egui::{Painter, Rect};

/// Static renderer for the n-D Lookup Table: the `<n>-D T(u)` caption Simulink
/// prints above the interpolation curve.
pub fn static_lookup_table(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let dims = ctx
        .metadata
        .get("NumberOfTableDimensions")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("1");
    let spec = format!(
        concat!(
            "t 0.50,0.14,0.26 {dims}-D T(u);",
            "bc 0.08,0.92 0.1467,0.9133 0.2233,0.9200 0.28,0.90 0.3367,0.88 0.38,0.8567 0.42,0.80 0.46,0.7433 0.4867,0.6367 0.52,0.56 0.5533,0.4833 0.58,0.39 0.62,0.34 0.66,0.29 0.7067,0.2767 0.76,0.26 0.8133,0.2433 0.88,0.2467 0.94,0.24"
        ),
        dims = dims
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
