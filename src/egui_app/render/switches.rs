//! Switch blocks, whose icon is wired to the position of their ports.

#![cfg(feature = "egui")]

use super::labels::PortLabelMaxWidths;
use crate::model::Block;
use eframe::egui::{self, Align2, Color32, Pos2, Rect, Stroke};

/// Screen-space Y coordinates computed for a block's ports (as used by the UI when placing
/// port labels and clamped within the block rect). Keys are 1-based port indices.
#[derive(Clone, Debug, Default)]
pub struct ComputedPortYCoordinates {
    pub inputs: std::collections::HashMap<u32, f32>,
    pub outputs: std::collections::HashMap<u32, f32>,
}

/// Custom renderer for a ManualSwitch block.
///
/// Draws a simple switch symbol with two input poles (left) and one output pole (right).
/// The pole centers are aligned to the exact y-positions of the ports so that
/// connecting lines meet them cleanly. The lever connects from the selected input
/// (current_setting: "0" => bottom, "1" => top; default "0") to the output pole.
pub fn render_manual_switch(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    _font_scale: f32,
    coords: Option<&ComputedPortYCoordinates>,
) {
    // Determine how many ports to align (fall back to common defaults)
    let mut max_in: u32 = 0;
    let mut max_out: u32 = 0;
    for p in &block.ports {
        let idx = p.index.unwrap_or(0).max(1);
        if p.port_type == "in" {
            max_in = max_in.max(idx);
        }
        if p.port_type == "out" {
            max_out = max_out.max(idx);
        }
    }
    if max_in == 0 {
        max_in = 2;
    }
    if max_out == 0 {
        max_out = 1;
    }

    // Compute port anchors (in screen space)
    use crate::egui_app::geometry::PortSide;
    let mirrored = block.block_mirror.unwrap_or(false);
    let in_side = if mirrored {
        PortSide::Out
    } else {
        PortSide::In
    };
    let out_side = if mirrored {
        PortSide::In
    } else {
        PortSide::Out
    };
    let default_in1 = crate::egui_app::geometry::port_anchor_pos(*rect, in_side, 1, Some(max_in));
    let default_in2 = crate::egui_app::geometry::port_anchor_pos(*rect, in_side, 2, Some(max_in));
    let default_out = crate::egui_app::geometry::port_anchor_pos(*rect, out_side, 1, Some(max_out));

    // Place pole centers slightly inside the block border so the circles are fully visible
    let pad = 8.0_f32; // horizontal inset from the border for the circle centers
    let r_in = (rect.height() * 0.06).clamp(2.0, 6.0) * 0.8; // 20% smaller
    let r_out = r_in;
    let stroke_w = 1.5_f32; // thinner
    let col_active = Color32::from_rgb(32, 32, 32);
    let col_inactive = Color32::from_rgb(110, 110, 110); // dark gray for inactive

    let top_in_y = coords
        .and_then(|c| c.inputs.get(&1).copied())
        .unwrap_or(default_in1.y);
    let bot_in_y = coords
        .and_then(|c| c.inputs.get(&2).copied())
        .unwrap_or(default_in2.y);
    let out_y = coords
        .and_then(|c| c.outputs.get(&1).copied())
        .unwrap_or(default_out.y);

    let (top_in_center, bot_in_center, out_center) = if !mirrored {
        (
            Pos2::new(rect.left() + pad, top_in_y),
            Pos2::new(rect.left() + pad, bot_in_y),
            Pos2::new(rect.right() - pad, out_y),
        )
    } else {
        (
            Pos2::new(rect.right() - pad, top_in_y),
            Pos2::new(rect.right() - pad, bot_in_y),
            Pos2::new(rect.left() + pad, out_y),
        )
    };

    // Horizontal leads from border to the pole circles up to circle edge
    if !mirrored {
        let in1_anchor = Pos2::new(rect.left(), top_in_y);
        let in2_anchor = Pos2::new(rect.left(), bot_in_y);
        let out_anchor = Pos2::new(rect.right(), out_y);
        painter.line_segment(
            [
                in1_anchor,
                Pos2::new(top_in_center.x - r_in, top_in_center.y),
            ],
            Stroke::new(stroke_w, col_active),
        );
        painter.line_segment(
            [
                in2_anchor,
                Pos2::new(bot_in_center.x - r_in, bot_in_center.y),
            ],
            Stroke::new(stroke_w, col_active),
        );
        painter.line_segment(
            [Pos2::new(out_center.x + r_out, out_center.y), out_anchor],
            Stroke::new(stroke_w, col_active),
        );
    } else {
        let in1_anchor = Pos2::new(rect.right(), top_in_y);
        let in2_anchor = Pos2::new(rect.right(), bot_in_y);
        let out_anchor = Pos2::new(rect.left(), out_y);
        painter.line_segment(
            [
                in1_anchor,
                Pos2::new(top_in_center.x + r_in, top_in_center.y),
            ],
            Stroke::new(stroke_w, col_active),
        );
        painter.line_segment(
            [
                in2_anchor,
                Pos2::new(bot_in_center.x + r_in, bot_in_center.y),
            ],
            Stroke::new(stroke_w, col_active),
        );
        painter.line_segment(
            [Pos2::new(out_center.x - r_out, out_center.y), out_anchor],
            Stroke::new(stroke_w, col_active),
        );
    }

    // Draw open-circuit poles
    let set_top = matches!(block.current_setting.as_deref(), Some("1"));
    let top_col = if set_top { col_active } else { col_inactive };
    let bot_col = if set_top { col_inactive } else { col_active };
    painter.circle_stroke(top_in_center, r_in, Stroke::new(stroke_w, top_col));
    painter.circle_stroke(bot_in_center, r_in, Stroke::new(stroke_w, bot_col));
    painter.circle_stroke(out_center, r_out, Stroke::new(stroke_w, col_active));

    // Small stubs from the circle edge OUTSIDE the circle (1/3 of the circle diameter)
    let stub = (2.0 * r_in / 3.0).max(0.8); // 1/3 diameter
    // Input stubs extend inside the block: to the right for left-side inputs, to the left for right-side inputs.
    let in1_color = top_col;
    let in2_color = bot_col;
    if !mirrored {
        let in1_edge = top_in_center.x + r_in; // rightmost point of top input circle
        let in2_edge = bot_in_center.x + r_in; // rightmost point of bottom input circle
        painter.line_segment(
            [
                Pos2::new(in1_edge, top_in_center.y),
                Pos2::new(in1_edge + stub, top_in_center.y),
            ],
            Stroke::new(stroke_w, in1_color),
        );
        painter.line_segment(
            [
                Pos2::new(in2_edge, bot_in_center.y),
                Pos2::new(in2_edge + stub, bot_in_center.y),
            ],
            Stroke::new(stroke_w, in2_color),
        );
        // Output stub extends to the LEFT of the output circle: [edge - stub, edge]
        let out_edge_left = out_center.x - r_out; // leftmost point of output circle
        painter.line_segment(
            [
                Pos2::new(out_edge_left - stub, out_center.y),
                Pos2::new(out_edge_left, out_center.y),
            ],
            Stroke::new(stroke_w, col_active),
        );
        // Lever connects from active input stub end to output stub end
        let from_edge = in1_edge;
        let from_edge2 = in2_edge;
        let from_y_top = top_in_center.y;
        let from_y_bot = bot_in_center.y;
        let from_edge_sel = if set_top { from_edge } else { from_edge2 };
        let from_y_sel = if set_top { from_y_top } else { from_y_bot };
        let start = Pos2::new(from_edge_sel + stub, from_y_sel);
        let end = Pos2::new(out_edge_left - stub, out_center.y);
        painter.line_segment([start, end], Stroke::new(stroke_w, col_active));
    } else {
        let in1_edge = top_in_center.x - r_in; // leftmost point of top input circle (inputs on right)
        let in2_edge = bot_in_center.x - r_in; // leftmost point of bottom input circle
        painter.line_segment(
            [
                Pos2::new(in1_edge, top_in_center.y),
                Pos2::new(in1_edge - stub, top_in_center.y),
            ],
            Stroke::new(stroke_w, in1_color),
        );
        painter.line_segment(
            [
                Pos2::new(in2_edge, bot_in_center.y),
                Pos2::new(in2_edge - stub, bot_in_center.y),
            ],
            Stroke::new(stroke_w, in2_color),
        );
        // Output stub extends to the RIGHT of the output circle (output on left): [edge, edge + stub]
        let out_edge_right = out_center.x + r_out; // rightmost point of output circle
        painter.line_segment(
            [
                Pos2::new(out_edge_right, out_center.y),
                Pos2::new(out_edge_right + stub, out_center.y),
            ],
            Stroke::new(stroke_w, col_active),
        );
        // Lever
        let from_edge = in1_edge;
        let from_edge2 = in2_edge;
        let from_y_top = top_in_center.y;
        let from_y_bot = bot_in_center.y;
        let from_edge_sel = if set_top { from_edge } else { from_edge2 };
        let from_y_sel = if set_top { from_y_top } else { from_y_bot };
        let start = Pos2::new(from_edge_sel - stub, from_y_sel);
        let end = Pos2::new(out_edge_right + stub, out_center.y);
        painter.line_segment([start, end], Stroke::new(stroke_w, col_active));
    }
}

/// Custom renderer for a MultiPortSwitch block.
///
/// Draws a selector lever routing the numbered data inputs to the output.
/// Port 1 is the control input (topmost on the left edge), ports 2..=N+1 are
/// the data inputs, and output port 1 is on the right edge.  The lever connects
/// from the first data contact to the output, matching Simulink's default
/// selection.  All line endpoints align to the exact Y-positions of the ports
/// so connecting lines meet them cleanly.
pub fn render_multiport_switch(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    data_inputs: u32,
    coords: Option<&ComputedPortYCoordinates>,
    port_label_widths: Option<PortLabelMaxWidths>,
) {
    render_multiport_switch_with_selection(
        painter,
        block,
        rect,
        font_scale,
        data_inputs,
        coords,
        port_label_widths,
        0,
    )
}

/// Like [`render_multiport_switch`] but draws the lever to the `selected`-th
/// data contact (0-based) instead of always the first.  Used by the live
/// renderer to reflect the control signal value.
#[allow(clippy::too_many_arguments)]
pub fn render_multiport_switch_with_selection(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    data_inputs: u32,
    coords: Option<&ComputedPortYCoordinates>,
    port_label_widths: Option<PortLabelMaxWidths>,
    selected: u32,
) {
    let mut max_in: u32 = 0;
    let mut max_out: u32 = 0;
    for p in &block.ports {
        let idx = p.index.unwrap_or(0).max(1);
        if p.port_type == "in" {
            max_in = max_in.max(idx);
        }
        if p.port_type == "out" {
            max_out = max_out.max(idx);
        }
    }
    // total inputs = control(1) + data_inputs
    let total_in = data_inputs + 1;
    if max_in == 0 {
        max_in = total_in;
    }
    if max_out == 0 {
        max_out = 1;
    }

    use crate::egui_app::geometry::PortSide;
    let mirrored = block.block_mirror.unwrap_or(false);
    let in_side = if mirrored {
        PortSide::Out
    } else {
        PortSide::In
    };
    let out_side = if mirrored {
        PortSide::In
    } else {
        PortSide::Out
    };

    let stroke_w = 1.5_f32;
    let col_active = Color32::from_rgb(32, 32, 32);
    let col_inactive = Color32::from_rgb(110, 110, 110);
    let r_contact = (rect.height() * 0.04).clamp(2.0, 5.0);

    // Compute horizontal insets that clear the port labels.  The insets locate
    // the *centre* of a contact, so the contact's own radius has to be kept
    // clear of the label as well, otherwise the numbering runs into the
    // circles.
    let label_pad = 4.0 * font_scale;
    let label_gap = 4.0 * font_scale + r_contact;
    let margin_x = rect.width() * 0.10;
    let left_inset = if let Some(w) = port_label_widths {
        margin_x.max(label_pad + w.left + label_gap)
    } else {
        margin_x
    };
    let right_inset = if let Some(w) = port_label_widths {
        margin_x.max(label_pad + w.right + label_gap)
    } else {
        margin_x
    };

    // Port Y positions: control = port 1, data = ports 2..=total_in
    let port_y = |index: u32| {
        coords
            .and_then(|c| c.inputs.get(&index).copied())
            .unwrap_or_else(|| {
                crate::egui_app::geometry::port_anchor_pos(*rect, in_side, index, Some(max_in)).y
            })
    };
    let out_y = coords
        .and_then(|c| c.outputs.get(&1).copied())
        .unwrap_or_else(|| {
            crate::egui_app::geometry::port_anchor_pos(*rect, out_side, 1, Some(max_out)).y
        });

    let in_x = if mirrored { rect.right() } else { rect.left() };
    let out_x = if mirrored { rect.left() } else { rect.right() };
    let contact_x = if mirrored {
        rect.right() - left_inset
    } else {
        rect.left() + left_inset
    };
    let out_contact_x = if mirrored {
        rect.left() + right_inset
    } else {
        rect.right() - right_inset
    };
    // The port numbering is drawn between the border and the contacts, so the
    // leads start behind it instead of striking the text through.
    let lead_in_x = match port_label_widths {
        Some(w) if w.left > 0.0 => {
            let x = label_pad + w.left;
            if mirrored {
                rect.right() - x
            } else {
                rect.left() + x
            }
        }
        _ => in_x,
    };

    let stroke = Stroke::new(stroke_w, col_active);

    // Control input lead (port 1): short horizontal line from border to a
    // vertical bar representing the control contact.
    let control_y = port_y(1);
    painter.line_segment(
        [
            Pos2::new(lead_in_x, control_y),
            Pos2::new(contact_x, control_y),
        ],
        stroke,
    );
    // Vertical bar for the control contact.
    let bar_half = r_contact * 1.5;
    painter.line_segment(
        [
            Pos2::new(contact_x, control_y - bar_half),
            Pos2::new(contact_x, control_y + bar_half),
        ],
        stroke,
    );

    // Data input leads (ports 2..=total_in): horizontal line from border to
    // contact circle.
    let selected = selected.min(data_inputs.saturating_sub(1));
    for i in 0..data_inputs {
        let port_idx = i + 2;
        let y = port_y(port_idx);
        painter.line_segment([Pos2::new(lead_in_x, y), Pos2::new(contact_x, y)], stroke);
        // Contact circle: selected one is active, rest inactive.
        let col = if i == selected {
            col_active
        } else {
            col_inactive
        };
        painter.circle_stroke(
            Pos2::new(contact_x, y),
            r_contact,
            Stroke::new(stroke_w, col),
        );
    }

    // Output lead: from output contact circle to output border.
    let out_center = Pos2::new(out_contact_x, out_y);
    painter.line_segment(
        [Pos2::new(out_contact_x, out_y), Pos2::new(out_x, out_y)],
        stroke,
    );
    painter.circle_stroke(out_center, r_contact, Stroke::new(stroke_w, col_active));

    // Lever: from selected data contact to output contact.
    let selected_data_y = port_y(selected + 2);
    let lever_start = Pos2::new(contact_x, selected_data_y);
    let lever_end = out_center;
    painter.line_segment([lever_start, lever_end], stroke);
}

/// Custom renderer for a Switch block.
///
/// Draws a pass-through lever with the control criterion beside it.  Port 1
/// (top) and port 3 (bottom) are data inputs, port 2 (middle) is the control
/// input, and output port 1 is on the right edge.  The lever connects from
/// port 1 (top data, default selected) to the output.  All line endpoints
/// align to the exact Y-positions of the ports.
#[allow(clippy::too_many_arguments)]
pub fn render_switch(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    criteria: &str,
    threshold: &str,
    coords: Option<&ComputedPortYCoordinates>,
    port_label_widths: Option<PortLabelMaxWidths>,
) {
    render_switch_with_selection(
        painter,
        block,
        rect,
        font_scale,
        criteria,
        threshold,
        coords,
        port_label_widths,
        true,
    )
}

/// Like [`render_switch`] but draws the lever to the top data input (port 1)
/// when `selected_top` is true, or the bottom data input (port 3) when false.
/// Used by the live renderer to reflect the control signal value.
#[allow(clippy::too_many_arguments)]
pub fn render_switch_with_selection(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    criteria: &str,
    threshold: &str,
    coords: Option<&ComputedPortYCoordinates>,
    port_label_widths: Option<PortLabelMaxWidths>,
    selected_top: bool,
) {
    let mut max_in: u32 = 0;
    let mut max_out: u32 = 0;
    for p in &block.ports {
        let idx = p.index.unwrap_or(0).max(1);
        if p.port_type == "in" {
            max_in = max_in.max(idx);
        }
        if p.port_type == "out" {
            max_out = max_out.max(idx);
        }
    }
    if max_in == 0 {
        max_in = 3;
    }
    if max_out == 0 {
        max_out = 1;
    }

    use crate::egui_app::geometry::PortSide;
    let mirrored = block.block_mirror.unwrap_or(false);
    let in_side = if mirrored {
        PortSide::Out
    } else {
        PortSide::In
    };
    let out_side = if mirrored {
        PortSide::In
    } else {
        PortSide::Out
    };

    let stroke_w = 1.5_f32;
    let col_active = Color32::from_rgb(32, 32, 32);
    let col_inactive = Color32::from_rgb(110, 110, 110);
    let r_contact = (rect.height() * 0.04).clamp(2.0, 5.0);

    // Compute horizontal insets that clear the port labels; as in the
    // MultiPortSwitch, the contact radius is part of the gap because the inset
    // positions the contact's centre.
    let label_pad = 4.0 * font_scale;
    let label_gap = 4.0 * font_scale + r_contact;
    let margin_x = rect.width() * 0.10;
    let left_inset = if let Some(w) = port_label_widths {
        margin_x.max(label_pad + w.left + label_gap)
    } else {
        margin_x
    };
    let right_inset = if let Some(w) = port_label_widths {
        margin_x.max(label_pad + w.right + label_gap)
    } else {
        margin_x
    };

    let port_y = |index: u32| {
        coords
            .and_then(|c| c.inputs.get(&index).copied())
            .unwrap_or_else(|| {
                crate::egui_app::geometry::port_anchor_pos(*rect, in_side, index, Some(max_in)).y
            })
    };
    let out_y = coords
        .and_then(|c| c.outputs.get(&1).copied())
        .unwrap_or_else(|| {
            crate::egui_app::geometry::port_anchor_pos(*rect, out_side, 1, Some(max_out)).y
        });

    let in_x = if mirrored { rect.right() } else { rect.left() };
    let out_x = if mirrored { rect.left() } else { rect.right() };
    let contact_x = if mirrored {
        rect.right() - left_inset
    } else {
        rect.left() + left_inset
    };
    let out_contact_x = if mirrored {
        rect.left() + right_inset
    } else {
        rect.right() - right_inset
    };

    let stroke = Stroke::new(stroke_w, col_active);

    // Port 1 = top data input (u1), port 2 = control (u2), port 3 = bottom data (u3)
    let u1_y = port_y(1);
    let u2_y = port_y(2);
    let u3_y = port_y(3);

    // Data input leads with contact circles.
    // Top data (port 1): active when selected_top.
    painter.line_segment([Pos2::new(in_x, u1_y), Pos2::new(contact_x, u1_y)], stroke);
    painter.circle_stroke(
        Pos2::new(contact_x, u1_y),
        r_contact,
        Stroke::new(
            stroke_w,
            if selected_top {
                col_active
            } else {
                col_inactive
            },
        ),
    );
    // Bottom data (port 3): active when !selected_top.
    painter.line_segment([Pos2::new(in_x, u3_y), Pos2::new(contact_x, u3_y)], stroke);
    painter.circle_stroke(
        Pos2::new(contact_x, u3_y),
        r_contact,
        Stroke::new(
            stroke_w,
            if selected_top {
                col_inactive
            } else {
                col_active
            },
        ),
    );

    // Control input lead (port 2): short horizontal line with a vertical bar.
    painter.line_segment([Pos2::new(in_x, u2_y), Pos2::new(contact_x, u2_y)], stroke);
    let bar_half = r_contact * 1.5;
    painter.line_segment(
        [
            Pos2::new(contact_x, u2_y - bar_half),
            Pos2::new(contact_x, u2_y + bar_half),
        ],
        stroke,
    );

    // Output lead and contact.
    let out_center = Pos2::new(out_contact_x, out_y);
    painter.line_segment(
        [Pos2::new(out_contact_x, out_y), Pos2::new(out_x, out_y)],
        stroke,
    );
    painter.circle_stroke(out_center, r_contact, Stroke::new(stroke_w, col_active));

    // Lever: from selected data contact to output.
    let lever_y = if selected_top { u1_y } else { u3_y };
    let lever_start = Pos2::new(contact_x, lever_y);
    painter.line_segment([lever_start, out_center], stroke);

    // Criteria text (e.g. ">= 0") near the bottom-right of the block.
    let op = criteria
        .split_whitespace()
        .find(|t| t.starts_with('>') || t.starts_with('~') || t.starts_with('='))
        .unwrap_or(">=");
    let threshold = if threshold.is_empty() { "0" } else { threshold };
    let text = format!("{op} {threshold}");
    let font_px = (rect.height() * 0.22 * font_scale).clamp(6.0, 16.0);
    let text_pos = if mirrored {
        Pos2::new(
            rect.left() + rect.width() * 0.35,
            rect.bottom() - rect.height() * 0.15,
        )
    } else {
        Pos2::new(
            rect.right() - rect.width() * 0.35,
            rect.bottom() - rect.height() * 0.15,
        )
    };
    painter.text(
        text_pos,
        Align2::CENTER_CENTER,
        &text,
        egui::FontId::proportional(font_px),
        col_active,
    );
}
