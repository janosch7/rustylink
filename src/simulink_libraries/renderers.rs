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

/// Static renderer for the continuous Integrator block.
///
/// Simulink draws `1/s` on its own, but adds the saturation curve beside it
/// once output limiting is enabled (`LimitOutput = on`), which is the only
/// thing distinguishing an "Integrator Limited" instance from a plain one.
pub fn static_integrator(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let limited = ctx
        .metadata
        .get("LimitOutput")
        .is_some_and(|v| v.trim().eq_ignore_ascii_case("on"));
    if !limited {
        return false; // fall back to the definition's plain `frac:1/s` icon
    }
    // Fraction on the left two-thirds, saturation curve on the right third.
    let split = rect.left() + rect.width() * 0.60;
    let frac_rect = Rect::from_min_max(rect.min, eframe::egui::pos2(split, rect.bottom()));
    let curve_rect = Rect::from_min_max(eframe::egui::pos2(split, rect.top()), rect.max);
    crate::egui_app::render::draw_math_icon(
        painter,
        &frac_rect,
        ctx.font_scale,
        "frac:1/s",
        ctx.text_color,
        ctx.port_label_widths,
    );
    crate::egui_app::render::draw_plot_icon(
        painter,
        &curve_rect,
        ctx.font_scale,
        "p 0.0,0.86 0.30,0.86 0.72,0.14 1.0,0.14",
        ctx.text_color,
        None,
    );
    true
}

/// Static renderer for the Second-Order Integrator: `1/s²` with the two
/// integral signs Simulink prints beside its `x` and `dx` outputs.
pub fn static_second_order_integrator(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let split = rect.left() + rect.width() * 0.68;
    let frac_rect = Rect::from_min_max(rect.min, eframe::egui::pos2(split, rect.bottom()));
    crate::egui_app::render::draw_math_icon(
        painter,
        &frac_rect,
        ctx.font_scale,
        "frac:1/s\u{00B2}",
        ctx.text_color,
        ctx.port_label_widths,
    );
    let signs_rect = Rect::from_min_max(eframe::egui::pos2(split, rect.top()), rect.max);
    crate::egui_app::render::draw_plot_icon(
        painter,
        &signs_rect,
        ctx.font_scale,
        "t 0.5,0.28,0.42 \u{222B}; t 0.5,0.74,0.42 \u{222B}",
        ctx.text_color,
        None,
    );
    true
}

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
            "p 0.08,0.92 0.28,0.90 0.42,0.80 0.52,0.56 0.62,0.34 0.76,0.26 0.94,0.24"
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

/// Static renderer for the Switch block: the pass-through lever with the
/// control criterion (`Criteria` against `Threshold`, e.g. `> 0`) beside it.
pub fn static_switch(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let criteria = ctx.metadata.get("Criteria").unwrap_or("u2 >= Threshold");
    let op = criteria
        .split_whitespace()
        .find(|t| t.starts_with('>') || t.starts_with('~') || t.starts_with('='))
        .unwrap_or(">=");
    let threshold = ctx.metadata.get("Threshold").unwrap_or("0").trim();
    let threshold = if threshold.is_empty() { "0" } else { threshold };
    let spec = format!(
        concat!(
            "p 0.04,0.18 0.24,0.18; d 0.28,0.18 0.05;",
            "p 0.04,0.50 0.20,0.50; p 0.14,0.44 0.20,0.50 0.14,0.56;",
            "p 0.04,0.82 0.24,0.82; d 0.28,0.82 0.05;",
            "p 0.28,0.18 0.86,0.50; p 0.86,0.50 0.97,0.50;",
            "t 0.62,0.80,0.26 {op} {threshold}"
        ),
        op = op,
        threshold = threshold
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

/// Static renderer for a SubSystem: a miniature of its contents.
///
/// Simulink previews a subsystem by drawing the blocks it contains at reduced
/// scale; for the common case that is one In port wired across to one Out port
/// per signal, plus the enable/trigger badges on the top edge.
pub fn static_subsystem(
    painter: &Painter,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let mut ins = 0usize;
    let mut outs = 0usize;
    let mut enabled = false;
    let mut triggered = false;
    if let Some(system) = block.subsystem.as_deref() {
        for child in &system.blocks {
            match child.block_type.as_str() {
                "Inport" => ins += 1,
                "Outport" => outs += 1,
                "EnablePort" => enabled = true,
                "TriggerPort" => triggered = true,
                _ => {}
            }
        }
    }
    if let Some(counts) = block.port_counts.as_ref() {
        ins = ins.max(counts.ins.unwrap_or(0) as usize);
        outs = outs.max(counts.outs.unwrap_or(0) as usize);
    }
    let rows = ins.max(outs).clamp(1, 4);
    let mut spec = String::new();
    for row in 0..rows {
        let y = (row as f32 + 0.5) / rows as f32;
        let y = 0.20 + y * 0.60;
        if row < ins.max(1) {
            spec.push_str(&format!("o 0.16,{y:.3},0.13,0.09;"));
        }
        if row < outs.max(1) {
            spec.push_str(&format!("o 0.84,{y:.3},0.13,0.09;"));
        }
        spec.push_str(&format!("p 0.24,{y:.3} 0.76,{y:.3};"));
    }
    if enabled {
        spec.push_str("p 0.40,0.14 0.46,0.14 0.46,0.06 0.54,0.06 0.54,0.14 0.60,0.14;");
    }
    if triggered {
        spec.push_str("p 0.66,0.14 0.72,0.06 0.78,0.14;");
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

/// Static renderer for the Matrix Concatenate block: stacked blocks with the
/// `ConcatenateDimension` they are joined along printed in the corner.
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
    let spec = format!(
        concat!(
            "r 0.06,0.30 0.44,0.78; p 0.06,0.30 0.20,0.14 0.58,0.14 0.44,0.30;",
            "p 0.58,0.14 0.58,0.62 0.44,0.78;",
            "r 0.50,0.38 0.82,0.80; p 0.50,0.38 0.62,0.24 0.94,0.24 0.82,0.38;",
            "p 0.94,0.24 0.94,0.66 0.82,0.80;",
            "t 0.90,0.90,0.26 {dim}"
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
        .get("Triangularity")
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

/// Static renderer for the Delay block: `z` raised to a negative superscript
/// equal to the configured delay length (Simulink shows e.g. `z⁻²` for the
/// default `DelayLength = 2`).
pub fn static_delay(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let raw = ctx.metadata.get("DelayLength").unwrap_or("2").trim();
    let n = if raw.is_empty() { "2" } else { raw };
    let spec = format!("sup:z^-{n}");
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

/// Format a MATLAB coefficient row-vector (e.g. `"[1 2 1]"`, `"1,2,1"`) as a
/// polynomial string in `var`, highest power first (e.g. `"s^2+2s+1"`).
fn format_polynomial(raw: &str, var: char) -> String {
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
fn format_coeff(mag: f64) -> String {
    if (mag.fract()).abs() < 1e-9 {
        format!("{}", mag.round() as i64)
    } else {
        let s = format!("{mag:.3}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
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
        ctx.text_color,
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

#[cfg(test)]
mod tests {
    use super::{format_coeff, format_polynomial};

    #[test]
    fn polynomial_from_bracketed_vector() {
        assert_eq!(format_polynomial("[1 2 1]", 's'), "s^2+2s+1");
        assert_eq!(format_polynomial("[1 1]", 's'), "s+1");
        assert_eq!(format_polynomial("[1]", 's'), "1");
    }

    #[test]
    fn polynomial_handles_commas_zeros_and_signs() {
        assert_eq!(format_polynomial("1,0,-4", 's'), "s^2-4");
        assert_eq!(format_polynomial("[2 0 0]", 's'), "2s^2");
        assert_eq!(format_polynomial("[]", 's'), "1");
        assert_eq!(format_polynomial("[0 0]", 's'), "0");
    }

    #[test]
    fn coefficient_formatting_trims_trailing_zeros() {
        assert_eq!(format_coeff(3.0), "3");
        assert_eq!(format_coeff(2.5), "2.5");
    }
}
