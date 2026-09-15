//! Icon renderers for the Logic and Bit Operations blocks.
//!
//! Part of the renderer set: one module per Simulink library group.

#![cfg(feature = "egui")]

use super::common::body_colors;
use crate::model::Block;
use crate::simulink_libraries::types::RenderContext;
use eframe::egui::{Painter, Rect};

/// Static renderer for the Logic (Logical Operator) block. Reads `Operator`
/// (gate kind) and `IconShape` (rectangular text vs distinctive gate).
pub fn static_logic(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let operator = ctx.metadata.get("Operator").unwrap_or("AND");
    let icon_shape = ctx.metadata.get("IconShape").unwrap_or("rectangular");
    crate::egui_app::render::render_logic_block(
        painter,
        rect,
        ctx.font_scale,
        operator,
        icon_shape,
        body_colors(ctx),
    );
    true
}
