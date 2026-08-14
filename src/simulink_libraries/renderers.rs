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

/// A saturation curve (flat, ramp, flat) normalised to the icon area – the
/// marker Simulink adds to an integrator whose output is limited.
const SATURATION_CURVE: &str = "p 0.04,0.84 0.30,0.84 0.72,0.16 0.96,0.16";

/// Whether a Simulink on/off property is enabled.
fn is_on(ctx: &RenderContext<'_>, key: &str) -> bool {
    ctx.metadata
        .get(key)
        .is_some_and(|v| v.trim().eq_ignore_ascii_case("on"))
}

/// Whether a property selects an external source (`external` / `Input port`).
fn is_external(ctx: &RenderContext<'_>, key: &str) -> bool {
    ctx.metadata.get(key).is_some_and(|v| {
        let v = v.trim();
        v.eq_ignore_ascii_case("external") || v.eq_ignore_ascii_case("input port")
    })
}

/// Static renderer for the continuous Integrator block.
///
/// The `1/s` core is constant; the configuration decorates it: `LimitOutput`
/// adds the saturation curve, `WrapState` encircles the fraction.
pub fn static_integrator(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let limited = is_on(ctx, "LimitOutput");
    let wrapped = is_on(ctx, "WrapState");
    if !limited && !wrapped {
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
            "c 0.5,0.5 0.46",
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
        spec.push_str("c 0.50,0.26 0.26; p 0.62,0.06 0.76,0.14 0.60,0.22;");
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
    meta: &super::metadata::BlockMetadata,
    is_input: bool,
) -> Vec<String> {
    if !is_input {
        return Vec::new();
    }
    let mut labels = vec![String::new()];
    if let Some(glyph) = reset_glyph(meta.get("ExternalReset")) {
        labels.push(glyph.to_string());
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
    meta: &super::metadata::BlockMetadata,
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

/// The pictogram Simulink prints beside a reset port for each trigger edge.
fn reset_glyph(external_reset: Option<&str>) -> Option<&'static str> {
    match external_reset?.trim().to_ascii_lowercase().as_str() {
        "" | "none" => None,
        "rising" => Some("\u{2197}"),     // ↗
        "falling" => Some("\u{2198}"),    // ↘
        "either" => Some("\u{2195}"),     // ↕
        "level hold" => Some("\u{2294}"), // ⊔
        _ => Some("\u{2293}"),            // ⊓  (level)
    }
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
    let content = SubsystemContent::of(block);
    let rows = content.in_names.len().max(content.out_names.len()).max(1);
    let mut spec = String::new();
    for row in 0..rows {
        let y = 0.20 + (row as f32 + 0.5) / rows as f32 * 0.60;
        let left = content.in_names.get(row);
        let right = content.out_names.get(row);
        if let Some(name) = left {
            spec.push_str(&format!("t 0.08,{y:.3},0.20 {name};"));
            spec.push_str(&format!("o 0.22,{y:.3},0.10,0.10;"));
        }
        if let Some(name) = right {
            spec.push_str(&format!("o 0.78,{y:.3},0.10,0.10;"));
            spec.push_str(&format!("t 0.92,{y:.3},0.20 {name};"));
        }
        if left.is_some() && right.is_some() {
            spec.push_str(&format!("p 0.27,{y:.3} 0.73,{y:.3};"));
        }
    }
    // Simulink prints the conditional-execution pictogram of the port block the
    // subsystem contains above the wiring preview.
    if content.enabled {
        spec.push_str("p 0.40,0.13 0.44,0.13 0.44,0.05 0.56,0.05 0.56,0.13 0.60,0.13;");
    }
    if content.triggered {
        spec.push_str("p 0.42,0.13 0.42,0.05 0.50,0.05 0.50,0.13 0.58,0.13 0.58,0.05;");
    }
    if content.for_each {
        spec.push_str(concat!(
            "t 0.52,0.09,0.18 N;",
            "b 0.54,0.12 0.72,0.44; r 0.54,0.12 0.72,0.44;",
            "r 0.48,0.18 0.66,0.50; r 0.42,0.24 0.60,0.56"
        ));
    }
    if let Some(event) = content.event.as_ref() {
        // Simulink heads the function-call subsystem with a lifecycle pictogram
        // and the name of the event it listens for.
        spec.push_str("c 0.24,0.26,0.10;");
        spec.push_str(match event.kind {
            EventKind::Reset => "p 0.24,0.16 0.31,0.19 0.30,0.12;",
            _ => "p 0.24,0.14 0.24,0.26;",
        });
        spec.push_str(&format!("t 0.58,0.26,0.26 {};", event.caption));
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

/// The parts of a subsystem's contents that shape how Simulink draws it.
struct SubsystemContent {
    in_names: Vec<String>,
    out_names: Vec<String>,
    enabled: bool,
    triggered: bool,
    for_each: bool,
    /// The lifecycle event a contained `EventListener` responds to – what
    /// distinguishes initialize/reset/reinitialize/terminate function
    /// subsystems from one another.
    event: Option<SubsystemEvent>,
}

struct SubsystemEvent {
    kind: EventKind,
    caption: String,
}

#[derive(PartialEq, Eq)]
enum EventKind {
    Lifecycle,
    Reset,
}

impl SubsystemEvent {
    /// Simulink captions Initialize and Terminate functions with the event
    /// itself; Reset and Reinitialize functions carry a user-chosen event name.
    fn of(listener: &Block) -> SubsystemEvent {
        let event_type = listener
            .properties
            .get("EventType")
            .map(|s| s.trim())
            .unwrap_or("Initialize");
        let name = listener
            .properties
            .get("EventName")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        match event_type.to_ascii_lowercase().as_str() {
            "reset" => SubsystemEvent {
                kind: EventKind::Reset,
                caption: name.unwrap_or("reset").to_string(),
            },
            "reinitialize" => SubsystemEvent {
                kind: EventKind::Lifecycle,
                caption: name.unwrap_or("reinit").to_string(),
            },
            other => SubsystemEvent {
                kind: EventKind::Lifecycle,
                caption: other.to_string(),
            },
        }
    }
}

impl SubsystemContent {
    fn of(block: &Block) -> Self {
        let mut content = SubsystemContent {
            in_names: Vec::new(),
            out_names: Vec::new(),
            enabled: false,
            triggered: false,
            for_each: false,
            event: None,
        };
        if let Some(system) = block.subsystem.as_deref() {
            for child in &system.blocks {
                match child.block_type.as_str() {
                    "Inport" => content.in_names.push(port_caption(child)),
                    "Outport" => content.out_names.push(port_caption(child)),
                    "EnablePort" => content.enabled = true,
                    "TriggerPort" => content.triggered = true,
                    "ForEach" => content.for_each = true,
                    "EventListener" => content.event = Some(SubsystemEvent::of(child)),
                    _ => {}
                }
            }
        }
        content
    }
}

/// Caption Simulink prints for a boundary port inside a subsystem preview: the
/// port number for the default `In1`/`Out1` names, the block name otherwise.
fn port_caption(child: &Block) -> String {
    let name = child.name.trim();
    let digits: String = name
        .trim_start_matches(|c: char| c.is_ascii_alphabetic())
        .to_string();
    let stem = &name[..name.len() - digits.len()];
    if (stem.eq_ignore_ascii_case("in") || stem.eq_ignore_ascii_case("out")) && !digits.is_empty() {
        digits
    } else {
        name.to_string()
    }
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
        .get("Mode")
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

/// Static renderer for the Delay block: `z` raised to a negative superscript.
///
/// The exponent is the configured `DelayLength` when the length is a dialog
/// parameter, and the symbolic `d` once it comes from the delay-length input
/// port (`DelayLengthSource = Input port`).
pub fn static_delay(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
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

/// Input port labels for the Delay block, derived from `InputPortMap`.
///
/// Simulink encodes the enabled optional inputs as a comma-separated token
/// list (`u0,p1,e6,r5,p4`): the signal, the delay length, the enable, the
/// reset and the external initial condition, in the order they appear on the
/// block.  A Delay with only the signal input is left unlabelled.
pub fn delay_port_labels(
    _block: &Block,
    meta: &super::metadata::BlockMetadata,
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
            Some('e') => "\u{2293}".to_string(),
            Some('r') => reset_glyph(meta.get("ExternalReset"))
                .unwrap_or("\u{2293}")
                .to_string(),
            // `p1` is the delay length, `p4` the initial condition; Simulink
            // lists the length first.
            Some('p') if *token == "p1" => "d".to_string(),
            Some('p') => "x0".to_string(),
            _ => String::new(),
        })
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

/// Static renderer for the Concatenate block.
///
/// `Mode` picks the pictogram: multidimensional-array concatenation is drawn
/// as two joined cuboids labelled with `ConcatenateDimension`, while vector
/// and matrix concatenation are the stacked slabs of the incoming signals.
pub fn static_concatenate(
    painter: &Painter,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let multidimensional = ctx
        .metadata
        .get("Mode")
        .is_some_and(|m| m.to_lowercase().contains("multidimensional"));
    if multidimensional {
        return static_matrix_concatenate(painter, block, rect, ctx);
    }
    let inputs = block
        .port_counts
        .as_ref()
        .and_then(|c| c.ins)
        .unwrap_or(2)
        .clamp(2, 6);
    let mut spec = String::new();
    for i in 1..inputs {
        let y = i as f32 / inputs as f32;
        spec.push_str(&format!("p 0.0,{y:.3} 1.0,{y:.3};"));
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

/// Static renderer for the Selector block.
///
/// A one-dimensional selection is drawn literally: one marker per input
/// element, filled when the index vector picks it, wired across to the output
/// elements.  Multi-dimensional selections are labelled `U`/`Y` instead.
pub fn static_selector(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let dims: u32 = ctx
        .metadata
        .get("NumberOfDimensions")
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(1);
    if dims > 1 {
        crate::egui_app::render::draw_plot_icon(
            painter,
            rect,
            ctx.font_scale,
            "t 0.20,0.50,0.34 U; t 0.80,0.50,0.34 Y",
            ctx.text_color,
            None,
        );
        return true;
    }

    let width: usize = ctx
        .metadata
        .get("InputPortWidth")
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(3usize)
        .clamp(1, 6);
    let selected: Vec<usize> = ctx
        .metadata
        .get("Indices")
        .unwrap_or("[1]")
        .split([',', ' ', '[', ']'])
        .filter_map(|t| t.trim().parse::<usize>().ok())
        .collect();

    let mut spec = String::new();
    let mut out_row = 0usize;
    let out_count = selected.len().max(1);
    for i in 0..width {
        let y = (i as f32 + 0.5) / width as f32;
        let y = 0.14 + y * 0.72;
        let picked = selected.contains(&(i + 1));
        if picked {
            spec.push_str(&format!("f 0.14,{:.3} 0.30,{:.3};", y - 0.09, y + 0.09));
            let oy = (out_row as f32 + 0.5) / out_count as f32;
            let oy = 0.14 + oy * 0.72;
            spec.push_str(&format!("f 0.70,{:.3} 0.86,{:.3};", oy - 0.09, oy + 0.09));
            spec.push_str(&format!("p 0.30,{y:.3} 0.70,{oy:.3};"));
            out_row += 1;
        } else {
            spec.push_str(&format!("r 0.16,{:.3} 0.28,{:.3};", y - 0.07, y + 0.07));
        }
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

/// Static renderer for the Multiport Switch: the selector lever routing the
/// numbered data inputs to the output.
pub fn static_multiport_switch(
    painter: &Painter,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let data_inputs = multiport_switch_data_inputs(block, ctx.metadata);
    // Ports are spaced evenly down the left edge: the control input on top,
    // then one contact per data input, the first of which the lever selects.
    let total = data_inputs + 1;
    let port_y = |index: u32| (index as f32 + 1.0) / (total as f32 + 1.0);
    let control_y = port_y(0);
    let mut spec = format!(
        "p 0.0,{control_y:.3} 0.30,{control_y:.3}; p 0.30,{:.3} 0.30,{:.3};",
        control_y - 0.06,
        control_y + 0.06
    );
    let first_contact_y = port_y(1);
    spec.push_str(&format!(
        "p 0.72,{first_contact_y:.3} 0.90,0.50; p 0.90,0.50 1.0,0.50;"
    ));
    for i in 0..data_inputs {
        let y = port_y(i + 1);
        spec.push_str(&format!(
            "p 0.0,{y:.3} 0.64,{y:.3}; r 0.64,{:.3} 0.72,{:.3};",
            y - 0.035,
            y + 0.035
        ));
    }
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

/// Number of data inputs a Multiport Switch exposes (the port count minus the
/// control input, or the configured `Inputs` count).
fn multiport_switch_data_inputs(block: &Block, meta: &super::metadata::BlockMetadata) -> u32 {
    meta.get("Inputs")
        .and_then(|s| s.trim().parse::<u32>().ok())
        .or_else(|| {
            block
                .port_counts
                .as_ref()
                .and_then(|c| c.ins)
                .map(|n| n.saturating_sub(1))
        })
        .unwrap_or(3)
        .max(1)
}

/// Input port labels for the Multiport Switch: the unlabelled control input
/// followed by the data inputs, the last of which is also the default (`*`).
pub fn multiport_switch_port_labels(
    block: &Block,
    meta: &super::metadata::BlockMetadata,
    is_input: bool,
) -> Vec<String> {
    if !is_input {
        return Vec::new();
    }
    let data = multiport_switch_data_inputs(block, meta);
    let mut labels = vec![String::new()];
    for i in 1..=data {
        labels.push(if i == data {
            format!("*, {i}")
        } else {
            i.to_string()
        });
    }
    labels
}

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
