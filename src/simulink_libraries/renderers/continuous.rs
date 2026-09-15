//! Icon renderers for the Continuous blocks.
//!
//! Part of the renderer set: one module per Simulink library group.

#![cfg(feature = "egui")]

use super::common::is_on;
use super::discontinuities::SATURATION_CURVE;
use super::ports_subsystems::{EITHER_EDGE, ENABLE_PORT, FALLING_EDGE, RESET_PORT, RISING_EDGE};
use crate::model::Block;
use crate::simulink_libraries::types::RenderContext;
use eframe::egui::{Painter, Rect};

/// An open circular arrow with an arrow head on either end – the marker
/// Simulink draws around (or beside) a state that wraps.
const WRAP_ARROW: &str = "sb 0.50,0.50,0.48,0.06,0.94";

/// Static renderer for the continuous Integrator block.
///
/// The `1/s` core is constant; the configuration decorates it: `LimitOutput`
/// adds the saturation curve, `WrapState` encircles the fraction with the wrap
/// arrow, and `ExternalReset` prints its trigger pictogram at the reset port.
pub fn static_integrator(
    painter: &Painter,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let limited = is_on(ctx, "LimitOutput");
    let wrapped = is_on(ctx, "WrapState");
    let reset = draw_port_pictograms(
        painter,
        rect,
        ctx,
        &integrator_port_labels(block, ctx.metadata, true),
        ctx.metadata.get("ExternalReset"),
    );
    if !limited && !wrapped && !reset {
        return false; // fall back to the definition's plain `frac:1/s` icon
    }
    let split = rect.left() + rect.width() * if limited { 0.60 } else { 1.0 };
    let frac_rect = Rect::from_min_max(rect.min, eframe::egui::pos2(split, rect.bottom()));
    crate::egui_app::render::draw_math_icon(
        painter,
        &frac_rect,
        ctx.font_scale,
        "frac:1/s",
        ctx.text_color,
        ctx.port_label_widths,
    );
    if wrapped {
        crate::egui_app::render::draw_plot_icon(
            painter,
            &frac_rect,
            ctx.font_scale,
            WRAP_ARROW,
            ctx.text_color,
            None,
        );
    }
    if limited {
        let curve_rect = Rect::from_min_max(eframe::egui::pos2(split, rect.top()), rect.max);
        crate::egui_app::render::draw_plot_icon(
            painter,
            &curve_rect,
            ctx.font_scale,
            SATURATION_CURVE,
            ctx.text_color,
            None,
        );
    }
    true
}

/// Static renderer for the Second-Order Integrator: `1/s²` decorated per state.
///
/// The `x` stage marker (upper right) is a saturation curve when `LimitX` is
/// on and a circle when `WrapX` is; the `dx` stage marker (lower right) is a
/// saturation curve when `LimitDXDT` is on.
pub fn static_second_order_integrator(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let split = rect.left() + rect.width() * 0.66;
    let frac_rect = Rect::from_min_max(rect.min, eframe::egui::pos2(split, rect.bottom()));
    crate::egui_app::render::draw_math_icon(
        painter,
        &frac_rect,
        ctx.font_scale,
        "frac:1/s\u{00B2}",
        ctx.text_color,
        ctx.port_label_widths,
    );

    let marks = Rect::from_min_max(eframe::egui::pos2(split, rect.top()), rect.max);
    let mut spec = String::new();
    if is_on(ctx, "LimitX") {
        spec.push_str("p 0.06,0.44 0.34,0.44 0.70,0.10 0.94,0.10;");
    } else if is_on(ctx, "WrapX") {
        // The `x` state wraps: an arc open towards the output port, with an
        // arrow head on both ends, in the upper half of the marker column.
        spec.push_str("sb 0.50,0.26,0.34,0.13,0.87;");
    }
    if is_on(ctx, "LimitDXDT") {
        spec.push_str("p 0.06,0.92 0.34,0.92 0.70,0.58 0.94,0.58;");
    }
    if !spec.is_empty() {
        crate::egui_app::render::draw_plot_icon(
            painter,
            &marks,
            ctx.font_scale,
            &spec,
            ctx.text_color,
            None,
        );
    }
    true
}

/// Input port labels for the continuous Integrator.
///
/// Simulink adds one port per enabled source: the external reset (labelled
/// with the edge pictogram it triggers on) and the external initial condition
/// (`x₀`).  The signal input itself is never labelled.
pub fn integrator_port_labels(
    _block: &Block,
    meta: &crate::simulink_libraries::metadata::BlockMetadata,
    is_input: bool,
) -> Vec<String> {
    if !is_input {
        return Vec::new();
    }
    let mut labels = vec![String::new()];
    if reset_spec(meta.get("ExternalReset")).is_some() {
        // The reset pictogram is line art drawn by the renderer, so the port
        // itself carries no text label.
        labels.push(RESET_PORT.to_string());
    }
    if matches!(meta.get("InitialConditionSource"), Some(s) if s.trim().eq_ignore_ascii_case("external"))
    {
        labels.push("x\u{2080}".to_string());
    }
    labels
}

/// Port labels for the Second-Order Integrator: `u` plus the external initial
/// conditions that are enabled, and the two integrated states as outputs.
pub fn second_order_integrator_port_labels(
    _block: &Block,
    meta: &crate::simulink_libraries::metadata::BlockMetadata,
    is_input: bool,
) -> Vec<String> {
    if !is_input {
        return vec!["x".to_string(), "dx".to_string()];
    }
    let external =
        |key: &str| matches!(meta.get(key), Some(s) if s.trim().eq_ignore_ascii_case("external"));
    let mut labels = vec!["u".to_string()];
    if external("ICSourceX") {
        labels.push("x\u{2080}".to_string());
    }
    if external("ICSourceDXDT") {
        labels.push("dx\u{2080}".to_string());
    }
    labels
}

/// The square pulse Simulink draws at an enable port and at a level-triggered
/// reset port.
pub(super) const LEVEL_PULSE: &str =
    "p 0.05,0.95 0.30,0.95 0.30,0.15 0.70,0.15 0.70,0.95 0.95,0.95";

/// The pictogram Simulink prints beside a reset port, per trigger edge: a step
/// with an arrow head on the triggering edge, or a plain square pulse for the
/// level-triggered modes.
pub(super) fn reset_spec(external_reset: Option<&str>) -> Option<&'static str> {
    match external_reset?.trim().to_ascii_lowercase().as_str() {
        "" | "none" => None,
        "rising" => Some(RISING_EDGE),
        "falling" => Some(FALLING_EDGE),
        "either" => Some(EITHER_EDGE),
        // `level` and `level hold` share the square-pulse pictogram.
        _ => Some(LEVEL_PULSE),
    }
}

/// Draw the pictograms of the marked input ports inside `rect`.
///
/// `labels` is the block's input-port label list: entries equal to
/// [`RESET_PORT`] / [`ENABLE_PORT`] stand for line art rather than text, and
/// the list length is the port count the ports are distributed over.  Returns
/// whether anything was drawn.
pub(super) fn draw_port_pictograms(
    painter: &Painter,
    rect: &Rect,
    ctx: &RenderContext<'_>,
    labels: &[String],
    external_reset: Option<&str>,
) -> bool {
    let count = labels.len().max(1) as f32;
    let size = (rect.height() / (count + 1.0))
        .min(rect.width() * 0.30)
        .min(16.0 * ctx.font_scale)
        .max(4.0);
    let mut drawn = false;
    for (index, label) in labels.iter().enumerate() {
        let spec = match label.as_str() {
            RESET_PORT => reset_spec(external_reset),
            ENABLE_PORT => Some(LEVEL_PULSE),
            _ => None,
        };
        let Some(spec) = spec else { continue };
        // Same distribution as `geometry::port_anchor_pos`.
        let y =
            rect.top() + (2.0 * (index as f32 + 1.0) - 0.5) / (2.0 * count + 1.0) * rect.height();
        let glyph = Rect::from_min_size(
            eframe::egui::pos2(rect.left() + size * 0.1, y - size * 0.5),
            eframe::egui::vec2(size, size),
        );
        crate::egui_app::render::draw_plot_icon(
            painter,
            &glyph,
            ctx.font_scale,
            spec,
            ctx.text_color,
            None,
        );
        drawn = true;
    }
    drawn
}

/// Static renderer for the continuous Transfer Fcn block: the numerator
/// polynomial over the denominator polynomial (in `s`), typeset with a real
/// fraction bar.  Reads the `Numerator`/`Denominator` coefficient vectors.
pub fn static_transfer_fcn(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let num = format_polynomial(ctx.metadata.get("Numerator").unwrap_or("[1]"), 's');
    let den = format_polynomial(ctx.metadata.get("Denominator").unwrap_or("[1 1]"), 's');
    let spec = format!("frac:{num}/{den}");
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

/// Format a MATLAB coefficient row-vector (e.g. `"[1 2 1]"`, `"1,2,1"`) as a
/// polynomial string in `var`, highest power first (e.g. `"s^2+2s+1"`).
pub(super) fn format_polynomial(raw: &str, var: char) -> String {
    let coeffs: Vec<f64> = raw
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split([',', ' ', '\t', ';'])
        .filter(|t| !t.is_empty())
        .filter_map(|t| t.trim().parse::<f64>().ok())
        .collect();
    if coeffs.is_empty() {
        return "1".to_string();
    }
    let degree = coeffs.len() - 1;
    let mut out = String::new();
    for (i, &c) in coeffs.iter().enumerate() {
        if c == 0.0 {
            continue;
        }
        let power = degree - i;
        let mag = c.abs();
        let unit_mag = (mag - 1.0).abs() < 1e-9;
        let coeff_str = if unit_mag && power != 0 {
            String::new()
        } else {
            format_coeff(mag)
        };
        let var_str = match power {
            0 => String::new(),
            1 => var.to_string(),
            _ => format!("{var}^{power}"),
        };
        let mut term = format!("{coeff_str}{var_str}");
        if term.is_empty() {
            term.push('1');
        }
        if out.is_empty() {
            if c < 0.0 {
                out.push('-');
            }
        } else {
            out.push(if c < 0.0 { '-' } else { '+' });
        }
        out.push_str(&term);
    }
    if out.is_empty() { "0".to_string() } else { out }
}

/// Format a non-negative coefficient magnitude without a trailing `.0`.
pub(super) fn format_coeff(mag: f64) -> String {
    if (mag.fract()).abs() < 1e-9 {
        format!("{}", mag.round() as i64)
    } else {
        let s = format!("{mag:.3}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}
