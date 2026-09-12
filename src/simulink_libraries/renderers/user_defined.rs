//! Icon renderers for the User-Defined Functions blocks.
//!
//! Part of the renderer set: one module per Simulink library group.

#![cfg(feature = "egui")]

use crate::model::Block;
use crate::simulink_libraries::types::RenderContext;
use eframe::egui::{Painter, Rect};

/// Static renderer for the C Function block: a bold `C` with the two raised
/// plus signs of the C++ logo.
pub fn static_c_function(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    crate::egui_app::render::draw_plot_icon(
        painter,
        rect,
        ctx.font_scale,
        concat!(
            "t 0.38,0.52,0.60 C;",
            "p 0.62,0.30 0.78,0.30; p 0.70,0.22 0.70,0.38;",
            "p 0.62,0.60 0.78,0.60; p 0.70,0.52 0.70,0.68"
        ),
        ctx.text_color,
        ctx.port_label_widths,
    );
    true
}

/// Static renderer for the MATLAB Function block: a bold `M`, with the name of
/// the function the block runs beneath it (`fcn`, `test`, … – taken from the
/// block's MATLAB source, not from its name).
pub fn static_matlab_function(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let name = ctx
        .metadata
        .get(crate::simulink_libraries::labels::MATLAB_FUNCTION_NAME_PROPERTY)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or("fcn");
    // A bold `M` as line art: strokes keep their weight at any block size,
    // where a glyph would have to be squeezed into the block's aspect ratio.
    let side = (rect.width() * 0.42).min(rect.height() * 0.42);
    let center = eframe::egui::pos2(rect.center().x, rect.top() + rect.height() * 0.36);
    let x = |u: f32| center.x + (u - 0.5) * side;
    let y = |v: f32| center.y + (v - 0.5) * side;
    painter.add(eframe::egui::Shape::line(
        vec![
            eframe::egui::pos2(x(0.05), y(1.0)),
            eframe::egui::pos2(x(0.05), y(0.0)),
            eframe::egui::pos2(x(0.50), y(0.62)),
            eframe::egui::pos2(x(0.95), y(0.0)),
            eframe::egui::pos2(x(0.95), y(1.0)),
        ],
        eframe::egui::Stroke::new((side * 0.16).max(1.0), ctx.text_color),
    ));
    painter.text(
        eframe::egui::pos2(rect.center().x, rect.top() + rect.height() * 0.78),
        eframe::egui::Align2::CENTER_CENTER,
        name,
        eframe::egui::FontId::proportional(
            (rect.height() * 0.28).min(16.0 * ctx.font_scale).max(1.0),
        ),
        ctx.text_color,
    );
    true
}
