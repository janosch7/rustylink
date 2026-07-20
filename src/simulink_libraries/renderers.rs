//! Adapter renderers that plug existing drawing routines into the unified
//! catalog's [`StaticRendererFn`] / [`LiveRendererFn`] signatures.
//!
//! These are thin wrappers so the catalog can reuse the well-tested drawing
//! code in `egui_app::render` and `egui_app::dashboard_widgets` without
//! duplicating it.  New libraries can either reuse these adapters or supply
//! their own renderer functions.

#![cfg(feature = "egui")]

use eframe::egui::{Painter, Rect};

use crate::model::Block;

use super::types::RenderContext;

/// Resolved body colors for a self-painting renderer.
fn body_colors(ctx: &RenderContext<'_>) -> crate::egui_app::render::BodyColors {
    crate::egui_app::render::BodyColors {
        fill: ctx.fill_color,
        border: ctx.border_color,
        text: ctx.text_color,
    }
}

/// Static renderer for the Sum block. Reads `IconShape` (round vs rectangular)
/// and `Inputs` (per-port +/- signs) from metadata and paints its own body.
pub fn static_sum(painter: &Painter, _block: &Block, rect: &Rect, ctx: &RenderContext<'_>) -> bool {
    let round = !ctx.metadata.get("IconShape").is_some_and(|s| {
        let s = s.trim();
        s.eq_ignore_ascii_case("rectangular") || s.eq_ignore_ascii_case("rect")
    });
    let ops = crate::egui_app::render::parse_input_operators(
        ctx.metadata.get("Inputs").unwrap_or_default(),
        '+',
    );
    crate::egui_app::render::render_sum_block(
        painter,
        rect,
        ctx.font_scale,
        &ops,
        round,
        body_colors(ctx),
    );
    true
}

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

/// Static renderer for the Product block. Reads `Inputs` (×/÷ per port) and
/// `Multiplication` (element-wise vs matrix). The shared passes draw the body.
pub fn static_product(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let ops = crate::egui_app::render::parse_input_operators(
        ctx.metadata.get("Inputs").unwrap_or_default(),
        '*',
    );
    let matrix = ctx
        .metadata
        .get("Multiplication")
        .is_some_and(|s| s.to_lowercase().contains("matrix"));
    crate::egui_app::render::render_product_block(
        painter,
        rect,
        ctx.font_scale,
        &ops,
        matrix,
        ctx.text_color,
    );
    true
}

/// Static renderer for the Math Function block. Reads `Operator` and paints the
/// matching typeset icon (superscript `eᵘ`/`u²`, overbar conjugate `ū`,
/// fraction `1/u`, …) instead of the flat operator word.
pub fn static_math_function(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let op = ctx.metadata.get("Operator").unwrap_or("exp").trim();
    let spec: std::borrow::Cow<'_, str> = match op {
        "exp" => "sup:e^u".into(),
        "10^u" | "pow10" => "sup:10^u".into(),
        "square" => "sup:u^2".into(),
        "pow" | "power" => "sup:u^v".into(),
        "sqrt" | "signedSqrt" | "rSqrt" => "\u{221A}u".into(),
        "reciprocal" => "frac:1/u".into(),
        "conj" => "over:u".into(),
        "transpose" => "sup:u^T".into(),
        "hermitian" => "sup:u^H".into(),
        "magnitude^2" => "|u|\u{00B2}".into(),
        "log10" => "log\u{2081}\u{2080}(u)".into(),
        "log" => "ln(u)".into(),
        other => other.into(),
    };
    crate::egui_app::render::draw_math_icon(
        painter,
        rect,
        ctx.font_scale,
        &spec,
        ctx.text_color,
        ctx.port_label_widths,
    );
    true
}

/// Static renderer for Goto/From blocks (draws the tag label).
pub fn static_goto_from(
    painter: &Painter,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    crate::egui_app::render::render_goto_from_block(
        painter,
        block,
        rect,
        ctx.font_scale,
        ctx.name_font_factor,
    );
    true
}

/// Static renderer for the ManualSwitch block.
pub fn static_manual_switch(
    painter: &Painter,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    crate::egui_app::render::render_manual_switch(painter, block, rect, ctx.font_scale, ctx.port_y);
    true
}

/// Live renderer for the ManualSwitch block: reflect the live signal value in
/// the drawn switch position.  Non-interactive, so `app` is ignored and drawing
/// goes through `ui.painter()`.
pub fn live_manual_switch(
    _app: &mut crate::egui_app::state::SubsystemApp,
    ui: &mut eframe::egui::Ui,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let Some(value) = ctx.live_value else {
        return false;
    };
    let mut live_block = block.clone();
    live_block.current_setting =
        Some(crate::egui_app::ui::update::manual_switch_setting_from_live_value(value).to_string());
    crate::egui_app::render::render_manual_switch(
        &ui.painter().with_clip_rect(*rect),
        &live_block,
        rect,
        ctx.font_scale,
        ctx.port_y,
    );
    true
}

/// Static renderer for Scope / DashboardScope blocks (waveform glyph).
pub fn static_scope(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    _ctx: &RenderContext<'_>,
) -> bool {
    crate::egui_app::ui::update::paint_scope_glyph(painter, rect);
    true
}

// NOTE: dashboard blocks no longer share a single general renderer hook here.
// Each dashboard block wires its own per-widget static/live renderer in
// `libraries::dashboard`, delegating to the matching `dashboard_widgets`
// drawing routine.
