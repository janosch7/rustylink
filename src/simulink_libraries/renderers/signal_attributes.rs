//! Icon renderers for the Signal Attributes blocks.
//!
//! Part of the renderer set: one module per Simulink library group.

#![cfg(feature = "egui")]

use crate::model::Block;
use crate::simulink_libraries::types::RenderContext;
use eframe::egui::{Painter, Rect};

/// Static renderer for the Data Type Conversion block: the target type, or
/// `convert` when the type is inherited.
pub fn static_data_type_conversion(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let out_type = ctx.metadata.get("OutDataTypeStr").unwrap_or("").trim();
    let label = if out_type.is_empty() || out_type.starts_with("Inherit") {
        "convert"
    } else {
        out_type
    };
    crate::egui_app::render::draw_plot_icon(
        painter,
        rect,
        ctx.font_scale,
        &format!("t 0.50,0.50,0.44 {label}"),
        ctx.text_color,
        ctx.port_label_widths,
    );
    true
}

/// Static renderer for the Signal Conversion block.
///
/// A plain signal copy fans the individual elements into a bus; the bus
/// conversions draw three signal lines through the conversion bar with the
/// virtual side dashed.
pub fn static_signal_conversion(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let output = ctx.metadata.get("ConversionOutput").unwrap_or("").trim();
    let spec: String = match output {
        "Virtual bus" | "Nonvirtual bus" => {
            // The dashed middle line marks the virtual side of the conversion.
            let (left, right) = if output == "Virtual bus" {
                (
                    "p 0.06,0.50 0.16,0.50; p 0.22,0.50 0.32,0.50;",
                    "p 0.62,0.50 0.94,0.50;",
                )
            } else {
                (
                    "p 0.06,0.50 0.38,0.50;",
                    "p 0.62,0.50 0.72,0.50; p 0.78,0.50 0.94,0.50;",
                )
            };
            format!(
                "p 0.06,0.28 0.94,0.28; p 0.06,0.72 0.94,0.72; {left} {right} b 0.40,0.10 0.60,0.90; r 0.40,0.10 0.60,0.90"
            )
        }
        _ => concat!(
            "b 0.10,0.10 0.30,0.28; r 0.10,0.10 0.30,0.28;",
            "b 0.10,0.41 0.30,0.59; r 0.10,0.41 0.30,0.59;",
            "b 0.10,0.72 0.30,0.90; r 0.10,0.72 0.30,0.90;",
            "b 0.68,0.10 0.88,0.36; r 0.68,0.10 0.88,0.36;",
            "b 0.68,0.37 0.88,0.63; r 0.68,0.37 0.88,0.63;",
            "b 0.68,0.64 0.88,0.90; r 0.68,0.64 0.88,0.90;",
            "p 0.30,0.19 0.68,0.23; p 0.30,0.50 0.68,0.50; p 0.30,0.81 0.68,0.77"
        )
        .to_string(),
    };
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
