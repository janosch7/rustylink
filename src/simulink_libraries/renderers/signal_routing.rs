//! Icon renderers and live behaviour for the Signal Routing blocks.
//!
//! Part of the renderer set: one module per Simulink library group.

#![cfg(feature = "egui")]

use crate::model::Block;
use crate::simulink_libraries::types::RenderContext;
use eframe::egui::{Painter, Rect};

/// Static renderer for the Switch block: the pass-through lever with the
/// control criterion (`Criteria` against `Threshold`, e.g. `> 0`) beside it.
pub fn static_switch(
    painter: &Painter,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let criteria = ctx.metadata.get("Criteria").unwrap_or("u2 >= Threshold");
    let threshold = ctx.metadata.get("Threshold").unwrap_or("0").trim();
    let threshold = if threshold.is_empty() { "0" } else { threshold };
    crate::egui_app::render::render_switch(
        painter,
        block,
        rect,
        ctx.font_scale,
        criteria,
        threshold,
        ctx.port_y,
        ctx.port_label_widths,
    );
    true
}

// ────────────────────────────────────────────────────────────────────────────
// Live renderers for Switch and MultiPortSwitch.
//
// These trace the incoming line to the control input port, find the source
// block, and use its live value to determine which data input the lever
// connects to.  They are display-only (not clickable).
// ────────────────────────────────────────────────────────────────────────────

/// Recursively check if a branch (or its sub-branches) terminates at the given
/// block SID and input port index.
fn branch_hits_port(branch: &crate::model::Branch, sid: &str, port_index: u32) -> bool {
    if branch
        .dst
        .as_ref()
        .is_some_and(|dst| dst.sid == sid && dst.port_index == port_index)
    {
        return true;
    }
    branch
        .branches
        .iter()
        .any(|child| branch_hits_port(child, sid, port_index))
}

/// Find the live value of the signal feeding a specific input port of `block`.
///
/// Traces the incoming line (or branch) to the given `control_port_index`,
/// finds the source block, and looks up its live value.
fn control_input_live_value(
    app: &crate::egui_app::state::SubsystemApp,
    block: &Block,
    control_port_index: u32,
) -> Option<f64> {
    let system = app.current_system()?;
    let block_sid = block.sid.as_deref()?;
    for line in &system.lines {
        let hits_control = line
            .dst
            .as_ref()
            .is_some_and(|dst| dst.sid == block_sid && dst.port_index == control_port_index)
            || line
                .branches
                .iter()
                .any(|b| branch_hits_port(b, block_sid, control_port_index));
        if hits_control
            && let Some(src) = &line.src
            && let Some(src_block) = system
                .blocks
                .iter()
                .find(|b| b.sid.as_deref() == Some(src.sid.as_str()))
        {
            return app
                .live_block_values
                .get(&app.live_value_key_for_block(src_block))
                .and_then(crate::live_values::LiveValueEntry::first_f64);
        }
    }
    None
}

/// Evaluate a Switch `Criteria` string against the control value and threshold.
///
/// Supported criteria forms:
/// - `u2 >= Threshold` / `u2 > Threshold`
/// - `u2 ~= 0` (or any literal threshold)
/// - `u2 <= Threshold` / `u2 < Threshold`
///
/// Returns `true` when the criteria is met (lever to top data input).
pub fn evaluate_switch_criteria(criteria: &str, control_value: f64, threshold: f64) -> bool {
    let trimmed = criteria.trim();
    // Find the comparison operator.
    for op in [">=", "<=", "~=", ">", "<", "=="] {
        if let Some(idx) = trimmed.find(op) {
            let lhs = trimmed[..idx].trim();
            let _ = lhs; // always "u2" or similar; we use control_value directly
            let rhs = trimmed[idx + op.len()..].trim();
            // rhs can be "Threshold" (use threshold param) or a literal number.
            let rhs_val: f64 = if rhs.eq_ignore_ascii_case("Threshold") {
                threshold
            } else {
                rhs.parse().unwrap_or(threshold)
            };
            return match op {
                ">=" => control_value >= rhs_val,
                "<=" => control_value <= rhs_val,
                "~=" => (control_value - rhs_val).abs() > f64::EPSILON,
                ">" => control_value > rhs_val,
                "<" => control_value < rhs_val,
                "==" => (control_value - rhs_val).abs() <= f64::EPSILON,
                _ => false,
            };
        }
    }
    // Default: treat as `>= threshold`.
    control_value >= threshold
}

/// Determine which data input (0-based index) a MultiPortSwitch should select
/// based on the control value and the block's numbering configuration.
pub fn compute_multiport_selection(
    block: &Block,
    meta: &crate::simulink_libraries::metadata::BlockMetadata,
    control_value: f64,
    data_inputs: u32,
) -> u32 {
    let numbered = multiport_switch_numbered_data_inputs(block, meta);
    let has_additional = multiport_switch_has_additional_default(meta);
    let order = meta.get("DataPortOrder").unwrap_or("One-based contiguous");

    let control_int = control_value as i64;

    // Build the list of index values for the numbered data ports.
    let indices: Vec<i64> = if order.trim().eq_ignore_ascii_case("Specify indices") {
        parse_data_port_indices(meta.get("DataPortIndices"))
            .iter()
            .map(|s| s.parse::<i64>().unwrap_or(0))
            .collect()
    } else if order.trim().eq_ignore_ascii_case("Zero-based contiguous") {
        (0..numbered as i64).collect()
    } else {
        (1..=numbered as i64).collect()
    };

    // Find which numbered port matches the control value.
    for (i, &idx) in indices.iter().enumerate() {
        if idx == control_int {
            return i as u32;
        }
    }

    // No match: select the default port.
    if has_additional {
        // Additional port is after the numbered ports.
        numbered
    } else {
        // Last numbered port is the default.
        numbered.saturating_sub(1)
    }
    .min(data_inputs.saturating_sub(1))
}

/// Live renderer for the Switch block: draws the lever to the top data input
/// when the control criteria is met, or the bottom data input otherwise.
pub fn live_switch(
    app: &mut crate::egui_app::state::SubsystemApp,
    ui: &mut eframe::egui::Ui,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let Some(control_value) = control_input_live_value(app, block, 2) else {
        return false;
    };
    let criteria = ctx.metadata.get("Criteria").unwrap_or("u2 >= Threshold");
    let threshold_str = ctx.metadata.get("Threshold").unwrap_or("0").trim();
    let threshold_val: f64 = threshold_str.parse().unwrap_or(0.0);
    let threshold = if threshold_str.is_empty() {
        "0"
    } else {
        threshold_str
    };
    let criteria_met = evaluate_switch_criteria(criteria, control_value, threshold_val);
    let painter = ui.painter().with_clip_rect(*rect);
    crate::egui_app::render::render_switch_with_selection(
        &painter,
        block,
        rect,
        ctx.font_scale,
        criteria,
        threshold,
        ctx.port_y,
        ctx.port_label_widths,
        criteria_met,
    );
    true
}

/// Live renderer for the MultiPortSwitch block: draws the lever to the data
/// input selected by the control signal value.
pub fn live_multiport_switch(
    app: &mut crate::egui_app::state::SubsystemApp,
    ui: &mut eframe::egui::Ui,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let Some(control_value) = control_input_live_value(app, block, 1) else {
        return false;
    };
    let data_inputs = multiport_switch_data_inputs(block, ctx.metadata);
    let selected = compute_multiport_selection(block, ctx.metadata, control_value, data_inputs);
    let painter = ui.painter().with_clip_rect(*rect);
    crate::egui_app::render::render_multiport_switch_with_selection(
        &painter,
        block,
        rect,
        ctx.font_scale,
        data_inputs,
        ctx.port_y,
        ctx.port_label_widths,
        selected,
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
    crate::egui_app::render::render_multiport_switch(
        painter,
        block,
        rect,
        ctx.font_scale,
        data_inputs,
        ctx.port_y,
        ctx.port_label_widths,
    );
    true
}

/// Number of numbered data inputs a Multiport Switch exposes, excluding the
/// optional additional default (`*`) port.
///
/// The count comes from the `Inputs` property when set, otherwise from the
/// number of indices in `DataPortIndices` (when `DataPortOrder = Specify
/// indices`), or from the block's port count minus the control input.
fn multiport_switch_numbered_data_inputs(
    block: &Block,
    meta: &crate::simulink_libraries::metadata::BlockMetadata,
) -> u32 {
    if let Some(n) = meta
        .get("Inputs")
        .and_then(|s| s.trim().parse::<u32>().ok())
    {
        return n.max(1);
    }
    let order = meta.get("DataPortOrder").unwrap_or("One-based contiguous");
    if order.trim().eq_ignore_ascii_case("Specify indices") {
        let count = parse_data_port_indices(meta.get("DataPortIndices")).len() as u32;
        return count.max(1);
    }
    block
        .port_counts
        .as_ref()
        .and_then(|c| c.ins)
        .map(|n| n.saturating_sub(1))
        .unwrap_or(3)
        .max(1)
}

/// Whether the Multiport Switch has an additional default (`*`) data port
/// beyond the numbered ones.
fn multiport_switch_has_additional_default(
    meta: &crate::simulink_libraries::metadata::BlockMetadata,
) -> bool {
    meta.get("DataPortForDefault")
        .is_some_and(|v| v.trim().eq_ignore_ascii_case("Additional data port"))
}

/// Number of data inputs a Multiport Switch exposes (numbered inputs plus the
/// optional additional default `*` port).
fn multiport_switch_data_inputs(
    block: &Block,
    meta: &crate::simulink_libraries::metadata::BlockMetadata,
) -> u32 {
    let numbered = multiport_switch_numbered_data_inputs(block, meta);
    if multiport_switch_has_additional_default(meta) {
        numbered + 1
    } else {
        numbered
    }
}

/// Parse a `DataPortIndices` value like `"{6,8,15}"` into a list of index
/// strings.
fn parse_data_port_indices(raw: Option<&str>) -> Vec<String> {
    let Some(raw) = raw else {
        return Vec::new();
    };
    raw.trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .split([',', ' '])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Input port labels for the Multiport Switch: the unlabelled control input
/// followed by the data inputs.  Numbering depends on `DataPortOrder`
/// (one-based, zero-based, or individual indices from `DataPortIndices`).
/// The default (`*`) port is labeled just `*` when it is an additional port,
/// or `*, N` when the last numbered port doubles as the default.
pub fn multiport_switch_port_labels(
    block: &Block,
    meta: &crate::simulink_libraries::metadata::BlockMetadata,
    is_input: bool,
) -> Vec<String> {
    if !is_input {
        return Vec::new();
    }
    let numbered = multiport_switch_numbered_data_inputs(block, meta);
    let has_additional = multiport_switch_has_additional_default(meta);
    let total_data = if has_additional {
        numbered + 1
    } else {
        numbered
    };

    // Build the list of number labels for the numbered data ports.
    let order = meta.get("DataPortOrder").unwrap_or("One-based contiguous");
    let number_labels: Vec<String> = if order.trim().eq_ignore_ascii_case("Specify indices") {
        let indices = parse_data_port_indices(meta.get("DataPortIndices"));
        (0..numbered)
            .map(|i| {
                indices
                    .get(i as usize)
                    .cloned()
                    .unwrap_or_else(|| (i + 1).to_string())
            })
            .collect()
    } else if order.trim().eq_ignore_ascii_case("Zero-based contiguous") {
        (0..numbered).map(|i| i.to_string()).collect()
    } else {
        (1..=numbered).map(|i| i.to_string()).collect()
    };

    let mut labels = vec![String::new()]; // control input (port 1)
    for i in 0..total_data {
        if has_additional && i == numbered {
            // Additional default port: just `*`.
            labels.push("*".to_string());
        } else if !has_additional && i == numbered - 1 {
            // Last numbered port doubles as default: `*, N`.
            labels.push(format!("*, {}", number_labels[i as usize]));
        } else {
            labels.push(number_labels[i as usize].clone());
        }
    }
    labels
}

/// Port labels for the BusAssignment block: the bus arrives on the first input
/// and leaves on the only output, both labelled `Bus`; the remaining inputs
/// carry the element each of them assigns, written `:= bus_b.e` as in Simulink
/// and taken from the comma-separated `AssignedSignals` property.
pub fn bus_assignment_port_labels(
    _block: &Block,
    meta: &crate::simulink_libraries::metadata::BlockMetadata,
    is_input: bool,
) -> Vec<String> {
    if !is_input {
        return vec!["Bus".to_string()];
    }
    let assigned = meta
        .get("AssignedSignals")
        .map(|s| {
            s.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| format!(":= {s}"))
                .collect()
        })
        .unwrap_or_default();
    let mut labels = vec!["Bus".to_string()];
    labels.extend::<Vec<String>>(assigned);
    labels
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
