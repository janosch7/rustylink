//! Icon renderers for the Math Operations blocks.
//!
//! Part of the renderer set: one module per Simulink library group.

#![cfg(feature = "egui")]

use super::common::body_colors;
use crate::model::Block;
use crate::simulink_libraries::types::RenderContext;
use eframe::egui::{Painter, Rect};

/// Whether a Sum block's `IconShape` selects the round body.
fn sum_is_round(icon_shape: Option<&str>) -> bool {
    !icon_shape.is_some_and(|s| {
        let s = s.trim();
        s.eq_ignore_ascii_case("rectangular") || s.eq_ignore_ascii_case("rect")
    })
}

/// Port placement for the Sum block.
///
/// The round body distributes its input ports evenly on the **left
/// semicircle** (from 12 o'clock through 9 o'clock to 6 o'clock),
/// counterclockwise.  The `Inputs` property string may contain `|` spacer
/// characters that occupy an angular slot but have no port — e.g. `|++`
/// places a gap at 12 o'clock and ports at 9 and 6 o'clock.
///
/// The rectangular variant keeps every input on the left edge (no overrides).
pub fn sum_port_overrides(
    _block: &Block,
    meta: &crate::simulink_libraries::metadata::BlockMetadata,
) -> Vec<crate::simulink_libraries::types::PortPositionOverride> {
    if !sum_is_round(meta.get("IconShape")) {
        return Vec::new();
    }

    let inputs_str = meta.get("Inputs").unwrap_or("++");
    let slots = crate::egui_app::render::parse_sum_slots(inputs_str);
    if slots.is_empty() {
        return Vec::new();
    }

    let angles = crate::egui_app::render::round_sum_slot_angles(&slots);
    let mut overrides = Vec::new();

    for (slot, angle) in angles.iter().flatten().enumerate() {
        let (placement, fraction) = crate::egui_app::render::angle_to_placement(*angle);
        overrides.push(crate::simulink_libraries::types::PortPositionOverride {
            is_input: true,
            port_index: slot as u32 + 1,
            from_end: false,
            placement,
            fraction,
        });
    }

    overrides
}

/// Static renderer for the Sum block. Reads `IconShape` (round vs rectangular)
/// and `Inputs` (per-port +/- signs) from metadata and paints its own body.
pub fn static_sum(painter: &Painter, _block: &Block, rect: &Rect, ctx: &RenderContext<'_>) -> bool {
    let round = sum_is_round(ctx.metadata.get("IconShape"));
    let inputs_str = ctx.metadata.get("Inputs").unwrap_or_default();
    let ops = crate::egui_app::render::parse_input_operators(inputs_str, '+');
    crate::egui_app::render::render_sum_block(
        painter,
        rect,
        ctx.font_scale,
        &ops,
        round,
        body_colors(ctx),
        inputs_str,
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
        // Every remaining power operator (`2^u`, `u^3`, …) is typeset as the
        // base with a raised exponent rather than printed with a literal caret.
        other => match other.split_once('^') {
            Some((base, exp)) if !base.is_empty() && !exp.is_empty() => {
                format!("sup:{base}^{exp}").into()
            }
            _ => other.into(),
        },
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

/// Whether a Trigonometry block's `Operator` produces the sine and the cosine
/// of its input on two separate outputs.
fn trig_is_sincos(operator: Option<&str>) -> bool {
    operator.is_some_and(|o| o.trim().eq_ignore_ascii_case("sincos"))
}

/// Static renderer for the Trigonometry block.
///
/// The `sincos` variant has no caption at all: Simulink identifies it by the
/// names of its two outputs, drawn by the port-label pass.  Every other
/// operator falls through to the definition's textual label.
pub fn static_trigonometry(
    _painter: &Painter,
    _block: &Block,
    _rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    trig_is_sincos(ctx.metadata.get("Operator"))
}

/// Output port labels for the Trigonometry block: only `sincos` names them.
pub fn trigonometry_port_labels(
    _block: &Block,
    meta: &crate::simulink_libraries::metadata::BlockMetadata,
    is_input: bool,
) -> Vec<String> {
    if is_input || !trig_is_sincos(meta.get("Operator")) {
        return Vec::new();
    }
    vec!["sin".to_string(), "cos".to_string()]
}

/// Output port labels for the Sine/Cosine lookup-table Reference blocks
/// (`simulink/Lookup Tables/Cosine`): the block's `Formula` from
/// `<InstanceData>`, split on ` and ` so a SineCosine block labels its two
/// outputs `sin(2*pi*u)` / `cos(2*pi*u)`.  Returns one label per output port;
/// an empty/short vector falls back to the default for the missing ports.
pub fn sine_cosine_output_labels(
    _block: &Block,
    meta: &crate::simulink_libraries::metadata::BlockMetadata,
    is_input: bool,
) -> Vec<String> {
    if is_input {
        return Vec::new();
    }
    let Some(formula) = meta.get("Formula") else {
        return Vec::new();
    };
    formula
        .split(" and ")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Static renderer for the Algebraic Constraint block: `Solve` above the
/// constraint the block enforces (`f(z) = 0` unless the model overrides it).
pub fn static_algebraic_constraint(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let constraint = ctx
        .metadata
        .get("Constraint")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("f(z) = 0");
    crate::egui_app::render::draw_math_icon(
        painter,
        rect,
        ctx.font_scale,
        &format!("lines:Solve|{constraint}"),
        ctx.text_color,
        ctx.port_label_widths,
    );
    true
}

/// Fork line-art splitting one input into the two named parts.
fn split_fork(top: &str, bottom: &str) -> String {
    format!(
        concat!(
            "p 0.04,0.50 0.24,0.50; p 0.24,0.50 0.40,0.24 0.56,0.24;",
            "p 0.24,0.50 0.40,0.76 0.56,0.76;",
            "t 0.78,0.24,0.30 {top}; t 0.78,0.76,0.30 {bottom}"
        ),
        top = top,
        bottom = bottom
    )
}

/// Fork line-art merging the two named parts into one output.
fn merge_fork(top: &str, bottom: &str) -> String {
    format!(
        concat!(
            "t 0.22,0.24,0.30 {top}; t 0.22,0.76,0.30 {bottom};",
            "p 0.44,0.24 0.60,0.24 0.76,0.50; p 0.44,0.76 0.60,0.76 0.76,0.50;",
            "p 0.76,0.50 0.96,0.50"
        ),
        top = top,
        bottom = bottom
    )
}

/// Draw either the two-part fork icon or the single-part formula, depending on
/// which parts the block is configured to expose.
fn draw_complex_icon(
    painter: &Painter,
    rect: &Rect,
    ctx: &RenderContext<'_>,
    fork: Option<String>,
    formula: &str,
) -> bool {
    match fork {
        Some(spec) => crate::egui_app::render::draw_plot_icon(
            painter,
            rect,
            ctx.font_scale,
            &spec,
            ctx.text_color,
            ctx.port_label_widths,
        ),
        None => crate::egui_app::render::draw_math_icon(
            painter,
            rect,
            ctx.font_scale,
            formula,
            ctx.text_color,
            ctx.port_label_widths,
        ),
    }
    true
}

/// Static renderer for Complex to Magnitude-Angle: `Output` selects whether
/// both parts fork out or a single `|u|` / `∠u` is produced.
pub fn static_complex_to_magnitude_angle(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let (fork, formula) = match ctx.metadata.get("Output").unwrap_or("").trim() {
        "Magnitude" => (None, "|u|"),
        "Angle" => (None, "\u{2220}u"),
        _ => (Some(split_fork("|u|", "\u{2220}u")), ""),
    };
    draw_complex_icon(painter, rect, ctx, fork, formula)
}

/// Static renderer for Complex to Real-Imag (`Output`: both, `Re(u)`, `Im(u)`).
pub fn static_complex_to_real_imag(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let (fork, formula) = match ctx.metadata.get("Output").unwrap_or("").trim() {
        "Real" => (None, "Re(u)"),
        "Imag" => (None, "Im(u)"),
        _ => (Some(split_fork("Re", "Im")), ""),
    };
    draw_complex_icon(painter, rect, ctx, fork, formula)
}

/// Static renderer for Magnitude-Angle to Complex.  When only one part comes
/// from the input port, Simulink names the dialog-supplied one `K`.
pub fn static_magnitude_angle_to_complex(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let (fork, formula) = match ctx.metadata.get("Input").unwrap_or("").trim() {
        "Magnitude" => (None, "u \u{2220}K"),
        "Angle" => (None, "K \u{2220}u"),
        _ => (Some(merge_fork("|u|", "\u{2220}u")), ""),
    };
    draw_complex_icon(painter, rect, ctx, fork, formula)
}

/// Static renderer for Real-Imag to Complex (`Input`: both, `u + jK`, `K + ju`).
pub fn static_real_imag_to_complex(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let (fork, formula) = match ctx.metadata.get("Input").unwrap_or("").trim() {
        "Real" => (None, "u + jK"),
        "Imag" => (None, "K + ju"),
        _ => (Some(merge_fork("Re", "Im")), ""),
    };
    draw_complex_icon(painter, rect, ctx, fork, formula)
}
