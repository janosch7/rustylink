//! Icon renderers for the Discrete blocks.
//!
//! Part of the renderer set: one module per Simulink library group.

#![cfg(feature = "egui")]

use super::common::is_external;
use super::continuous::draw_port_pictograms;
use super::ports_subsystems::{ENABLE_PORT, RESET_PORT};
use crate::model::Block;
use crate::simulink_libraries::types::RenderContext;
use eframe::egui::{Painter, Rect};

/// Static renderer for the Delay block: `z` raised to a negative superscript.
///
/// The exponent is the configured `DelayLength` when the length is a dialog
/// parameter, and the symbolic `d` once it comes from the delay-length input
/// port (`DelayLengthSource = Input port`).
pub fn static_delay(
    painter: &Painter,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    draw_port_pictograms(
        painter,
        rect,
        ctx,
        &delay_port_labels(block, ctx.metadata, true),
        ctx.metadata.get("ExternalReset"),
    );
    let exponent = if is_external(ctx, "DelayLengthSource") {
        "d".to_string()
    } else {
        let raw = ctx.metadata.get("DelayLength").unwrap_or("2").trim();
        if raw.is_empty() { "2" } else { raw }.to_string()
    };
    let spec = format!("sup:z^-{exponent}");
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

/// Static renderer for the Discrete-Time Integrator: the icon depends on the
/// integration method (Forward/Backward Euler or Trapezoidal), matching
/// Simulink's mask.
pub fn static_discrete_integrator(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let method = ctx
        .metadata
        .get("IntegratorMethod")
        .unwrap_or("")
        .to_lowercase();
    let spec = if method.contains("backward") {
        "frac:Ts z/z-1"
    } else if method.contains("trapezoidal") {
        "frac:Ts(z+1)/2(z-1)"
    } else {
        // Forward Euler (default).
        "frac:Ts/z-1"
    };
    crate::egui_app::render::draw_math_icon(
        painter,
        rect,
        ctx.font_scale,
        spec,
        ctx.text_color,
        ctx.port_label_widths,
    );
    true
}

/// Input port labels for the Delay block, derived from `InputPortMap`.
///
/// Simulink encodes the enabled optional inputs as a comma-separated token
/// list (`u0,p1,e6,r5,p4`): the signal, the delay length, the enable, the
/// reset and the external initial condition, in the order they appear on the
/// block.  A Delay with only the signal input is left unlabelled.
pub fn delay_port_labels(
    _block: &Block,
    meta: &crate::simulink_libraries::metadata::BlockMetadata,
    is_input: bool,
) -> Vec<String> {
    if !is_input {
        return Vec::new();
    }
    let map = meta.get("InputPortMap").unwrap_or("u0");
    let tokens: Vec<&str> = map
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .collect();
    if tokens.len() <= 1 {
        return Vec::new();
    }
    tokens
        .iter()
        .map(|token| match token.chars().next() {
            Some('u') => "u".to_string(),
            Some('e') => ENABLE_PORT.to_string(),
            Some('r') => RESET_PORT.to_string(),
            // `p1` is the delay length, `p4` the initial condition; Simulink
            // lists the length first.
            Some('p') if *token == "p1" => "d".to_string(),
            Some('p') => "x0".to_string(),
            _ => String::new(),
        })
        .collect()
}
