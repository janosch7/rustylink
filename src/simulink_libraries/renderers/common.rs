//! Helpers shared by several renderer groups.
//!
//! Part of the renderer set: one module per Simulink library group.

#![cfg(feature = "egui")]

use crate::model::Block;
use crate::simulink_libraries::types::RenderContext;
use eframe::egui::{Painter, Rect};

/// Resolved body colors for a self-painting renderer.
pub(super) fn body_colors(ctx: &RenderContext<'_>) -> crate::egui_app::render::BodyColors {
    crate::egui_app::render::BodyColors {
        fill: ctx.fill_color,
        border: ctx.border_color,
        text: ctx.text_color,
    }
}

/// Whether a Simulink on/off property is enabled.
pub(super) fn is_on(ctx: &RenderContext<'_>, key: &str) -> bool {
    ctx.metadata
        .get(key)
        .is_some_and(|v| v.trim().eq_ignore_ascii_case("on"))
}

/// Whether a property selects an external source (`external` / `Input port`).
pub(super) fn is_external(ctx: &RenderContext<'_>, key: &str) -> bool {
    ctx.metadata.get(key).is_some_and(|v| {
        let v = v.trim();
        v.eq_ignore_ascii_case("external") || v.eq_ignore_ascii_case("input port")
    })
}

/// A block Simulink draws empty: its identity comes from the port labels
/// alone, so claiming the interior keeps the `?` placeholder away.
pub fn static_nothing(
    _painter: &Painter,
    _block: &Block,
    _rect: &Rect,
    _ctx: &RenderContext<'_>,
) -> bool {
    true
}
