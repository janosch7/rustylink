//! Sum and Product, whose ports and signs come from the operator string.

#![cfg(feature = "egui")]

use super::body::BodyColors;
use super::icons::draw_plot_icon;
use eframe::egui::{self, Align2, Color32, Pos2, Rect, Stroke};

/// Draw the interior labels (+/-) for a Sum block.
///
/// The surrounding circle fill and stroke are drawn in the main ui loop's
/// background-fill and border-stroke passes.  This function only adds the
/// operator characters at their respective input-port positions inside the
/// circle – the left-edge operator for input 1 and the bottom-edge operator
/// for input 2.
///
/// The `Inputs` property format used by Simulink is e.g. `|++`: the first
/// character is an ignored spacer, the subsequent characters are the per-port
/// operators in order ('+' or '-').
pub fn render_sum_block(
    painter: &egui::Painter,
    rect: &Rect,
    font_scale: f32,
    operators: &[char],
    round: bool,
    colors: BodyColors,
    inputs_str: &str,
) {
    let stroke = Stroke::new((1.6 * font_scale).clamp(1.0, 3.0), colors.border);
    if round {
        let radius = rect.size().min_elem() / 2.0;
        painter.circle(rect.center(), radius, colors.fill, stroke);
    } else {
        painter.rect_filled(*rect, 4.0, colors.fill);
        painter.rect_stroke(*rect, 4.0, stroke, egui::StrokeKind::Inside);
    }

    let font_size = (rect.height() * 0.34).clamp(8.0, 22.0) * font_scale;
    let font_id = egui::FontId::proportional(font_size);
    let text = colors.text;

    if round {
        // Round Sum: place each operator label at its port's angular position
        // on the left semicircle (matching the port placement from
        // `sum_port_overrides`).
        let ops: &[char] = if operators.is_empty() {
            &['+', '+']
        } else {
            operators
        };
        let positions = round_sum_label_positions(rect, inputs_str);
        for (i, op) in ops.iter().enumerate() {
            if let Some(&pos) = positions.get(i) {
                painter.text(
                    pos,
                    Align2::CENTER_CENTER,
                    sign_str(*op),
                    font_id.clone(),
                    text,
                );
            }
        }
    } else {
        // Rectangular Add: stack the per-input signs down the left edge.
        let ops: &[char] = if operators.is_empty() {
            &['+', '+']
        } else {
            operators
        };
        let n = ops.len();
        for (i, op) in ops.iter().enumerate() {
            let f = (i as f32 + 1.0) / (n as f32 + 1.0);
            painter.text(
                Pos2::new(
                    rect.left() + rect.width() * 0.24,
                    rect.top() + f * rect.height(),
                ),
                Align2::CENTER_CENTER,
                sign_str(*op),
                font_id.clone(),
                text,
            );
        }
    }
}

/// Map a Sum/Product operator char to a display glyph (proper minus / division
/// signs instead of the ASCII forms).
fn sign_str(op: char) -> &'static str {
    match op {
        '-' => "\u{2212}", // −
        '/' => "\u{00F7}", // ÷
        '*' => "\u{00D7}", // ×
        _ => "+",
    }
}

/// Parse a Simulink `Inputs` string into per-port operator chars.
///
/// Accepts numeric forms (`"2"` → two `+`), sign strings (`"+-"`, `"|++"` –
/// the `|` spacer and other layout chars are ignored) for Sum, and `*`/`/`
/// forms for Product.
pub fn parse_input_operators(inputs: &str, default: char) -> Vec<char> {
    let s = inputs.trim();
    if s.is_empty() {
        return vec![default, default];
    }
    if let Ok(n) = s.parse::<usize>() {
        return vec![default; n.max(1)];
    }
    let ops: Vec<char> = s
        .chars()
        .filter(|c| matches!(c, '+' | '-' | '*' | '/'))
        .collect();
    if ops.is_empty() {
        vec![default, default]
    } else {
        ops
    }
}

/// Parse a Simulink Sum `Inputs` string into slots, preserving `|` spacers.
///
/// Each slot is `Some(char)` for an actual port (`+`, `-`, `*`, `/`) or
/// `None` for a `|` spacer.  A bare number means that many `+` ports with
/// no spacers.
pub fn parse_sum_slots(inputs: &str) -> Vec<Option<char>> {
    let s = inputs.trim();
    if s.is_empty() {
        return vec![Some('+'), Some('+')];
    }
    if let Ok(n) = s.parse::<usize>() {
        return vec![Some('+'); n.max(1)];
    }
    s.chars()
        .map(|c| if c == '|' { None } else { Some(c) })
        .collect()
}

/// Compute the angle (radians, math convention: 0 = right, pi/2 = up,
/// pi = left, 3*pi/2 = down) for each slot of a round Sum on the left
/// semicircle.
///
/// For N total slots, slot i is at:
/// - N == 1: 180° (9 o'clock)
/// - N > 1:  90° + i * (180° / (N-1))
///
/// Returns `None` for `|` spacer slots.
pub fn round_sum_slot_angles(slots: &[Option<char>]) -> Vec<Option<f32>> {
    let n = slots.len();
    if n == 0 {
        return Vec::new();
    }
    slots
        .iter()
        .enumerate()
        .map(|(i, slot)| {
            if slot.is_none() {
                None
            } else if n == 1 {
                Some(std::f32::consts::PI)
            } else {
                Some(std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / (n - 1) as f32)
            }
        })
        .collect()
}

/// Map an angle (math convention) to a `(PortPlacement, fraction)` on a
/// block's bounding rectangle.
///
/// For the left semicircle (90°..270°), the ray from the center hits the
/// top, left, or bottom edge.  The fraction is the position along that edge.
pub fn angle_to_placement(angle: f32) -> (crate::simulink_libraries::types::PortPlacement, f32) {
    use crate::simulink_libraries::types::PortPlacement;

    let cos_a = angle.cos();
    let sin_a = angle.sin();

    if sin_a.abs() > 1e-6 && cos_a.abs() > 1e-6 {
        if sin_a.abs() > cos_a.abs() {
            // Hits top or bottom
            if sin_a > 0.0 {
                // Top edge: fraction from 0.5 (at 90°) to 0.0 (at 135°)
                let frac = 0.5 - 0.5 * (cos_a.abs() / sin_a.abs());
                (PortPlacement::Top, frac)
            } else {
                // Bottom edge: fraction from 0.0 (at 225°) to 0.5 (at 270°)
                let frac = 0.5 - 0.5 * (cos_a.abs() / sin_a.abs());
                (PortPlacement::Bottom, frac)
            }
        } else {
            // Hits left edge: fraction from top
            let frac = 0.5 - 0.5 * (sin_a / cos_a.abs());
            (PortPlacement::Left, frac.clamp(0.0, 1.0))
        }
    } else if sin_a.abs() > 1e-6 {
        if sin_a > 0.0 {
            (PortPlacement::Top, 0.5)
        } else {
            (PortPlacement::Bottom, 0.5)
        }
    } else {
        (PortPlacement::Left, 0.5)
    }
}

/// Compute the screen positions for the `+`/`-` operator labels inside a
/// round Sum block, one per actual port (in order).
///
/// Labels are placed on the circle at the port's angle, slightly inward
/// from the circle edge.
pub fn round_sum_label_positions(rect: &Rect, inputs_str: &str) -> Vec<Pos2> {
    let slots = parse_sum_slots(inputs_str);
    let angles = round_sum_slot_angles(&slots);
    let center = rect.center();
    let radius = rect.size().min_elem() / 2.0;
    // Place labels at 65% of the radius from center
    let label_radius = radius * 0.65;

    let mut positions = Vec::new();
    for angle in angles.iter().flatten() {
        // Convert math angle to screen direction:
        // math: x = cos, y = sin (up)
        // screen: x = cos, y = -sin (y is flipped)
        let dx = angle.cos() * label_radius;
        let dy = -angle.sin() * label_radius;
        positions.push(Pos2::new(center.x + dx, center.y + dy));
    }
    positions
}

/// Draw a Product/Divide block interior (the shared passes draw the body).
///
/// When every input multiplies, a single centred `×` is shown (Simulink's
/// element-wise product icon).  When any input divides, the per-port `×`/`÷`
/// signs are stacked down the left edge.  Matrix multiplication adds brackets.
pub fn render_product_block(
    painter: &egui::Painter,
    rect: &Rect,
    font_scale: f32,
    operators: &[char],
    matrix: bool,
    text: Color32,
) {
    let has_div = operators.contains(&'/');
    if !has_div {
        // A single multiply port collapses the input vector: Simulink shows the
        // product-of-elements symbol `∏`, drawn as large as the block allows,
        // rather than the element-wise `×`.  It is drawn rather than typeset so
        // the bar overhangs both legs the way Simulink's icon does.
        if matches!(operators, [c] if *c == '*') && !matrix {
            draw_plot_icon(
                painter,
                rect,
                font_scale,
                "p 0.08,0.16 0.92,0.16; p 0.26,0.16 0.26,0.90; p 0.74,0.16 0.74,0.90",
                text,
                None,
            );
            return;
        }
        let font_size = (rect.height() * 0.5).clamp(9.0, 30.0) * font_scale;
        let font_id = egui::FontId::proportional(font_size);
        let glyph = if matrix {
            "[\u{00D7}]"
        } else {
            "\u{00D7}" // ×
        };
        painter.text(rect.center(), Align2::CENTER_CENTER, glyph, font_id, text);
        return;
    }
    let font_size = (rect.height() * 0.34).clamp(8.0, 22.0) * font_scale;
    let font_id = egui::FontId::proportional(font_size);
    let n = operators.len().max(1);
    for (i, op) in operators.iter().enumerate() {
        let f = (i as f32 + 1.0) / (n as f32 + 1.0);
        painter.text(
            Pos2::new(
                rect.left() + rect.width() * 0.30,
                rect.top() + f * rect.height(),
            ),
            Align2::CENTER_CENTER,
            sign_str(*op),
            font_id.clone(),
            text,
        );
    }
}
