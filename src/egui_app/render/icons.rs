//! Drawing the icon of a block: glyphs, math notation and icon specs.

#![cfg(feature = "egui")]

use super::block_type::get_block_type_cfg;
use super::body::concave_polygon_mesh;
use super::labels::PortLabelMaxWidths;
use crate::block_types::{self};
use crate::model::Block;
use eframe::egui::{self, Align2, Color32, Pos2, Rect, Stroke, Vec2};
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

pub fn compute_icon_available_rect(
    rect: &Rect,
    font_scale: f32,
    port_label_widths: Option<PortLabelMaxWidths>,
) -> Rect {
    let margin_x = rect.width() * 0.10;
    let margin_y = rect.height() * 0.10;

    let mut left_inset = margin_x;
    let mut right_inset = margin_x;

    if let Some(w) = port_label_widths {
        let label_pad = 4.0 * font_scale;
        let label_gap = 2.0 * font_scale;
        if w.left > 0.0 {
            left_inset = left_inset.max(label_pad + w.left + label_gap);
        }
        if w.right > 0.0 {
            right_inset = right_inset.max(label_pad + w.right + label_gap);
        }
    }

    let mut min = Pos2::new(rect.left() + left_inset, rect.top() + margin_y);
    let mut max = Pos2::new(rect.right() - right_inset, rect.bottom() - margin_y);
    if min.x >= max.x {
        let cx = rect.center().x;
        min.x = cx;
        max.x = cx;
    }
    if min.y >= max.y {
        let cy = rect.center().y;
        min.y = cy;
        max.y = cy;
    }
    Rect::from_min_max(min, max)
}

fn maximize_glyph_font_px(painter: &egui::Painter, glyph: &str, avail: Vec2) -> f32 {
    if avail.x <= 1.0 || avail.y <= 1.0 {
        return 1.0;
    }

    // Measure once at a reference size and scale. This avoids per-block binary searches.
    let ref_px = 100.0_f32;
    let ref_galley = painter.layout_no_wrap(
        glyph.to_string(),
        egui::FontId::proportional(ref_px),
        Color32::TRANSPARENT,
    );
    let ref_size = ref_galley.size();
    if ref_size.x <= 1e-3 || ref_size.y <= 1e-3 {
        return 1.0;
    }

    let mut font_px = (ref_px * (avail.x / ref_size.x).min(avail.y / ref_size.y)).max(1.0);

    // Nudge up a tiny bit while still fitting, then nudge down if needed.
    for _ in 0..6 {
        let try_px = font_px * 1.02;
        let g = painter.layout_no_wrap(
            glyph.to_string(),
            egui::FontId::proportional(try_px),
            Color32::TRANSPARENT,
        );
        let s = g.size();
        if s.x <= avail.x && s.y <= avail.y {
            font_px = try_px;
        } else {
            break;
        }
    }
    for _ in 0..8 {
        let g = painter.layout_no_wrap(
            glyph.to_string(),
            egui::FontId::proportional(font_px),
            Color32::TRANSPARENT,
        );
        let s = g.size();
        if s.x <= avail.x && s.y <= avail.y {
            break;
        }
        font_px *= 0.98;
        if font_px <= 1.0 {
            font_px = 1.0;
            break;
        }
    }
    font_px
}

pub fn render_center_glyph_maximized(
    painter: &egui::Painter,
    rect: &Rect,
    font_scale: f32,
    glyph: &str,
    color: Color32,
    port_label_widths: Option<PortLabelMaxWidths>,
) {
    let avail_rect = compute_icon_available_rect(rect, font_scale, port_label_widths);
    let avail = avail_rect.size();
    let font_px = maximize_glyph_font_px(painter, glyph, avail);
    let font_id = egui::FontId::proportional(font_px);
    painter.text(
        avail_rect.center(),
        Align2::CENTER_CENTER,
        glyph,
        font_id,
        color,
    );
}

/// Draw a small piece of typeset math (a horizontal fraction bar, a raised
/// superscript, or an overbar) centred in `rect`.  Simulink draws these block
/// icons as 2-D math that a single one-line glyph string can't reproduce, so we
/// paint them here.  `spec` is a compact notation understood by this painter:
///
/// * `frac:NUM/DEN` – numerator over denominator with a horizontal bar
///   (`frac:1/s`, `frac:(z-1)/z`, `frac:K(z-1)/Ts z`).
/// * `sup:BASE^SUP` – `BASE` with a raised, smaller superscript
///   (`sup:z^-2`, `sup:e^u`, `sup:u^2`).  Text after a space in `SUP` returns
///   to the baseline, so `sup:A^H A` typesets `AᴴA`.
/// * `over:BASE` – `BASE` with an overbar (conjugate, e.g. `over:u` → `ū`).
/// * `lines:A|B` – stacked, centred lines at a common font size (e.g. the
///   Descriptor State-Space icon `lines:Eẋ = Ax + Bu|y = Cx + Du`).
///
/// Anything else is drawn as a plain maximised glyph (same as a `Utf8` icon).
pub fn draw_math_icon(
    painter: &egui::Painter,
    rect: &Rect,
    font_scale: f32,
    spec: &str,
    color: Color32,
    port_label_widths: Option<PortLabelMaxWidths>,
) {
    let avail = compute_icon_available_rect(rect, font_scale, port_label_widths);
    if avail.width() <= 1.0 || avail.height() <= 1.0 {
        return;
    }
    if let Some(rest) = spec.strip_prefix("frac:") {
        let (num, den) = rest.split_once('/').unwrap_or((rest, ""));
        draw_fraction(painter, &avail, num.trim(), den.trim(), color);
    } else if let Some(rest) = spec.strip_prefix("sup:") {
        let (base, sup) = rest.split_once('^').unwrap_or((rest, ""));
        let (sup, tail) = sup.split_once(' ').unwrap_or((sup, ""));
        draw_superscript(painter, &avail, base, sup, tail, color);
    } else if let Some(base) = spec.strip_prefix("over:") {
        draw_overbar(painter, &avail, base, color);
    } else if let Some(rest) = spec.strip_prefix("lines:") {
        draw_stacked_lines(painter, &avail, rest, color);
    } else {
        render_center_glyph_maximized(painter, rect, font_scale, spec, color, port_label_widths);
    }
}

/// `|`-separated lines stacked vertically and centred, all at one font size.
fn draw_stacked_lines(painter: &egui::Painter, avail: &Rect, spec: &str, color: Color32) {
    let lines: Vec<&str> = spec.split('|').map(str::trim).collect();
    if lines.is_empty() {
        return;
    }
    let n = lines.len() as f32;
    let row = Vec2::new(avail.width() * 0.96, (avail.height() * 0.94) / n);
    let font_px = lines
        .iter()
        .map(|l| fit_font_px(painter, l, row))
        .fold(f32::INFINITY, f32::min)
        .clamp(5.0, 40.0);
    let font = egui::FontId::proportional(font_px);
    let step = font_px * 1.16;
    let top = avail.center().y - step * (n - 1.0) * 0.5;
    for (i, line) in lines.iter().enumerate() {
        painter.text(
            Pos2::new(avail.center().x, top + step * i as f32),
            Align2::CENTER_CENTER,
            *line,
            font.clone(),
            color,
        );
    }
}

/// Draw a line-art block icon from a compact notation.
///
/// Simulink draws many icons (source waveforms, saturation/backlash curves,
/// scope screens, verification plots) as vector line art rather than as a
/// glyph.  `spec` is a `;`-separated list of drawing commands whose coordinates
/// are normalised to `0.0..=1.0` inside the icon area, with `y` pointing
/// **down** so the notation reads like screen space:
///
/// * `p X,Y X,Y …` – polyline through the listed points.
/// * `a X,Y X,Y …` – same, but faint: Simulink's thin grey axis cross.
/// * `b X0,Y0,X1,Y1` – translucent filled band (Simulink's grey limit bands).
/// * `r X0,Y0,X1,Y1` – stroked rectangle.
/// * `f X0,Y0,X1,Y1` – solid rectangle (Simulink's black bus bars).
/// * `c CX,CY,R` – stroked circle (`R` is a fraction of the icon width).
/// * `d CX,CY,R` – filled dot.
/// * `o CX,CY,W,H` – stroked obround (the In/Out ports of a subsystem preview).
/// * `pg R,G,B,A X1,Y1 X2,Y2 X3,Y3 …` – filled polygon with an explicit RGBA
///   fill (each channel 0..=255, alpha included) and the standard outline
///   stroke.  Used for the shaded 3-D faces of the Matrix Concatenate icon.
/// * `pf R,G,B,A X1,Y1 X2,Y2 X3,Y3 …` – filled polygon with an explicit RGBA
///   fill but NO outline stroke.  Used for the concave L-shape pieces of the
///   Matrix Concatenate icon so the internal seam is not stroked; the external
///   boundary is drawn separately by a `p` command.
/// * `t X,Y,H TEXT` – `TEXT` centred at `X,Y` with cap height `H` (a fraction
///   of the icon height), for the letters Simulink sets inside its pictograms
///   (`A ⇒ D`, the `U` of Is Triangular).
///
/// Unknown commands are skipped, so a malformed spec degrades to blank rather
/// than panicking.
pub fn draw_plot_icon(
    painter: &egui::Painter,
    rect: &Rect,
    font_scale: f32,
    spec: &str,
    color: Color32,
    port_label_widths: Option<PortLabelMaxWidths>,
) {
    let avail = compute_icon_available_rect(rect, font_scale, port_label_widths);
    if avail.width() <= 1.0 || avail.height() <= 1.0 {
        return;
    }
    let at = |x: f32, y: f32| {
        Pos2::new(
            avail.left() + x * avail.width(),
            avail.top() + y * avail.height(),
        )
    };
    let width = (avail.width().min(avail.height()) * 0.055).clamp(0.8, 2.2);
    let stroke = Stroke::new(width, color);
    let faint = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 90);
    let axis_stroke = Stroke::new((width * 0.7).max(0.7), faint);
    let band = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 46);

    for cmd in spec.split(';') {
        let cmd = cmd.trim();
        let Some((kind, args)) = cmd.split_once(char::is_whitespace) else {
            continue;
        };
        let nums: Vec<f32> = args
            .split([' ', ','])
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse::<f32>().ok())
            .collect();
        match kind {
            "p" | "a" => {
                let pts: Vec<Pos2> = nums.chunks_exact(2).map(|c| at(c[0], c[1])).collect();
                if pts.len() >= 2 {
                    let s = if kind == "a" { axis_stroke } else { stroke };
                    painter.add(egui::Shape::line(pts, s));
                }
            }
            // Cubic Bezier curve: first pair is the start point (SVG M), each
            // subsequent group of 3 pairs is a cubic segment (SVG C: two
            // control points + endpoint).  Each segment is sampled with 16
            // steps for a smooth polyline.
            "bc" if nums.len() >= 8 && (nums.len() - 2).is_multiple_of(6) => {
                let mut pts: Vec<Pos2> = Vec::new();
                pts.push(at(nums[0], nums[1]));
                let segs = (nums.len() - 2) / 6;
                let steps = 16;
                for s in 0..segs {
                    let base = 2 + s * 6;
                    let p0 = *pts.last().unwrap();
                    let p1 = at(nums[base], nums[base + 1]);
                    let p2 = at(nums[base + 2], nums[base + 3]);
                    let p3 = at(nums[base + 4], nums[base + 5]);
                    for i in 1..=steps {
                        let t = i as f32 / steps as f32;
                        let u = 1.0 - t;
                        let x = p0.x * (u * u * u)
                            + p1.x * (3.0 * u * u * t)
                            + p2.x * (3.0 * u * t * t)
                            + p3.x * (t * t * t);
                        let y = p0.y * (u * u * u)
                            + p1.y * (3.0 * u * u * t)
                            + p2.y * (3.0 * u * t * t)
                            + p3.y * (t * t * t);
                        pts.push(Pos2::new(x, y));
                    }
                }
                painter.add(egui::Shape::line(pts, stroke));
            }
            "b" | "r" | "f" if nums.len() >= 4 => {
                let r = Rect::from_two_pos(at(nums[0], nums[1]), at(nums[2], nums[3]));
                match kind {
                    "b" => {
                        painter.rect_filled(r, 0.0, band);
                    }
                    "f" => {
                        painter.rect_filled(r, 0.0, color);
                    }
                    _ => {
                        painter.rect_stroke(r, 0.0, stroke, egui::StrokeKind::Inside);
                    }
                }
            }
            "o" if nums.len() >= 4 => {
                let c = at(nums[0], nums[1]);
                let half = Vec2::new(nums[2] * avail.width(), nums[3] * avail.height()) * 0.5;
                let r = Rect::from_center_size(c, half * 2.0);
                painter.rect_stroke(r, r.height() * 0.5, stroke, egui::StrokeKind::Inside);
            }
            // Filled polygon with an explicit RGBA fill: the first four numbers
            // are R,G,B,A (0..=255), the rest are vertex pairs.  Used for the
            // shaded 3-D faces of the Matrix Concatenate icon.
            "pg" if nums.len() >= 10 => {
                let fill = Color32::from_rgba_unmultiplied(
                    nums[0].round().clamp(0.0, 255.0) as u8,
                    nums[1].round().clamp(0.0, 255.0) as u8,
                    nums[2].round().clamp(0.0, 255.0) as u8,
                    nums[3].round().clamp(0.0, 255.0) as u8,
                );
                let pts: Vec<Pos2> = nums[4..].chunks_exact(2).map(|c| at(c[0], c[1])).collect();
                if pts.len() >= 3 {
                    painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
                }
            }
            // Filled polygon with an explicit RGBA fill but NO outline stroke:
            // same format as `pg`, used for the concave L-shape of the Matrix
            // Concatenate icon so the internal seam between the two convex
            // pieces is not stroked (the outline is drawn separately by a `p`
            // command tracing only the external boundary).
            "pf" if nums.len() >= 10 => {
                let fill = Color32::from_rgba_unmultiplied(
                    nums[0].round().clamp(0.0, 255.0) as u8,
                    nums[1].round().clamp(0.0, 255.0) as u8,
                    nums[2].round().clamp(0.0, 255.0) as u8,
                    nums[3].round().clamp(0.0, 255.0) as u8,
                );
                let pts: Vec<Pos2> = nums[4..].chunks_exact(2).map(|c| at(c[0], c[1])).collect();
                if pts.len() >= 3 {
                    painter.add(egui::Shape::mesh(concave_polygon_mesh(&pts, fill)));
                }
            }
            "t" if nums.len() >= 3 => {
                let Some((_, text)) = args.split_once(char::is_whitespace) else {
                    continue;
                };
                painter.text(
                    at(nums[0], nums[1]),
                    Align2::CENTER_CENTER,
                    text.trim(),
                    egui::FontId::proportional((nums[2] * avail.height()).clamp(4.0, 40.0)),
                    color,
                );
            }
            "c" | "d" if nums.len() >= 3 => {
                let c = at(nums[0], nums[1]);
                let r = nums[2] * avail.width();
                if kind == "c" {
                    painter.circle_stroke(c, r, stroke);
                } else {
                    painter.circle_filled(c, r, color);
                }
            }
            // Circular arc `cx,cy,r,from,to` (angles in turns, clockwise on
            // screen); `sa` puts an arrow head on the end, `sb` on both ends.
            // The radius is a fraction of the smaller side so the arc stays a
            // circle in a non-square icon area.
            "s" | "sa" | "sb" if nums.len() >= 5 => {
                let c = at(nums[0], nums[1]);
                let r = nums[2] * avail.width().min(avail.height());
                let (a0, a1) = (
                    nums[3] * std::f32::consts::TAU,
                    nums[4] * std::f32::consts::TAU,
                );
                let steps = 48;
                let point = |a: f32| Pos2::new(c.x + r * a.cos(), c.y + r * a.sin());
                let pts: Vec<Pos2> = (0..=steps)
                    .map(|i| point(a0 + (a1 - a0) * i as f32 / steps as f32))
                    .collect();
                painter.add(egui::Shape::line(pts, stroke));
                let head = |a: f32, backwards: bool| {
                    let tip = point(a);
                    // Tangent of the arc at `a`, pointing along the travel
                    // direction, and the inward normal.
                    let dir = if (a1 > a0) != backwards { 1.0 } else { -1.0 };
                    let t = Vec2::new(-a.sin(), a.cos()) * dir;
                    let n = Vec2::new(a.cos(), a.sin());
                    let len = (r * 0.28).clamp(2.0, 10.0);
                    painter.add(egui::Shape::line(
                        vec![
                            tip - t * len + n * len * 0.5,
                            tip,
                            tip - t * len - n * len * 0.5,
                        ],
                        stroke,
                    ));
                };
                if kind == "sa" || kind == "sb" {
                    head(a1, false);
                }
                if kind == "sb" {
                    head(a0, true);
                }
            }
            _ => {}
        }
    }
}

/// Font size (points) at which `text` fits within `max` bounds.
pub(super) fn fit_font_px(painter: &egui::Painter, text: &str, max: Vec2) -> f32 {
    if text.is_empty() {
        return 100.0;
    }
    let ref_px = 100.0_f32;
    let g = painter.layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(ref_px),
        Color32::TRANSPARENT,
    );
    let s = g.size();
    if s.x <= 1e-3 || s.y <= 1e-3 {
        return ref_px;
    }
    ref_px * (max.x / s.x).min(max.y / s.y)
}

fn text_width(painter: &egui::Painter, text: &str, font_px: f32) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    painter
        .layout_no_wrap(
            text.to_string(),
            egui::FontId::proportional(font_px),
            Color32::TRANSPARENT,
        )
        .size()
        .x
}

/// Numerator over a horizontal bar over denominator.
fn draw_fraction(painter: &egui::Painter, avail: &Rect, num: &str, den: &str, color: Color32) {
    let row_max = Vec2::new(avail.width() * 0.94, avail.height() * 0.44);
    let font_px = fit_font_px(painter, num, row_max)
        .min(fit_font_px(painter, den, row_max))
        .clamp(6.0, 40.0);
    let font = egui::FontId::proportional(font_px);
    let cx = avail.center().x;
    let cy = avail.center().y;
    let gap = font_px * 0.14;
    painter.text(
        Pos2::new(cx, cy - gap),
        Align2::CENTER_BOTTOM,
        num,
        font.clone(),
        color,
    );
    painter.text(
        Pos2::new(cx, cy + gap),
        Align2::CENTER_TOP,
        den,
        font,
        color,
    );
    let bar_w = text_width(painter, num, font_px).max(text_width(painter, den, font_px)) * 1.08;
    let stroke = Stroke::new((font_px * 0.07).clamp(1.0, 3.0), color);
    painter.line_segment(
        [
            Pos2::new(cx - bar_w * 0.5, cy),
            Pos2::new(cx + bar_w * 0.5, cy),
        ],
        stroke,
    );
}

/// `base` with a smaller superscript raised above the baseline.
fn draw_superscript(
    painter: &egui::Painter,
    avail: &Rect,
    base: &str,
    sup: &str,
    tail: &str,
    color: Color32,
) {
    if sup.is_empty() {
        let px = fit_font_px(painter, base, avail.size() * 0.9).clamp(6.0, 40.0);
        painter.text(
            avail.center(),
            Align2::CENTER_CENTER,
            base,
            egui::FontId::proportional(px),
            color,
        );
        return;
    }
    // Size so base (full) + superscript (0.62×, raised) fit the available box.
    let sup_ratio = 0.62_f32;
    let rise = 0.42_f32; // fraction of base height the superscript rises
    let ref_px = 100.0_f32;
    let bw = text_width(painter, base, ref_px);
    let sw = text_width(painter, sup, ref_px * sup_ratio);
    let tw = text_width(painter, tail, ref_px);
    let total_w = bw + sw + tw;
    let total_h = ref_px * (1.0 + rise);
    let font_px = if total_w <= 1e-3 {
        ref_px
    } else {
        (ref_px * ((avail.width() * 0.94) / total_w).min((avail.height() * 0.94) / total_h))
            .clamp(6.0, 40.0)
    };
    let base_font = egui::FontId::proportional(font_px);
    let sup_font = egui::FontId::proportional(font_px * sup_ratio);
    let base_w = text_width(painter, base, font_px);
    let sup_w = text_width(painter, sup, font_px * sup_ratio);
    let run_w = base_w + sup_w + text_width(painter, tail, font_px);
    let left = avail.center().x - run_w * 0.5;
    let cy = avail.center().y;
    painter.text(
        Pos2::new(left, cy),
        Align2::LEFT_CENTER,
        base,
        base_font.clone(),
        color,
    );
    painter.text(
        Pos2::new(left + base_w, cy - font_px * rise * 0.5),
        Align2::LEFT_CENTER,
        sup,
        sup_font,
        color,
    );
    if !tail.is_empty() {
        painter.text(
            Pos2::new(left + base_w + sup_w, cy),
            Align2::LEFT_CENTER,
            tail,
            base_font,
            color,
        );
    }
}

/// `base` with a horizontal overbar (Simulink's conjugate icon `ū`).
fn draw_overbar(painter: &egui::Painter, avail: &Rect, base: &str, color: Color32) {
    let font_px = fit_font_px(painter, base, avail.size() * Vec2::new(0.8, 0.78)).clamp(6.0, 40.0);
    let font = egui::FontId::proportional(font_px);
    let cx = avail.center().x;
    let cy = avail.center().y + font_px * 0.08;
    painter.text(Pos2::new(cx, cy), Align2::CENTER_CENTER, base, font, color);
    let w = text_width(painter, base, font_px) * 1.05;
    let bar_y = cy - font_px * 0.52;
    let stroke = Stroke::new((font_px * 0.07).clamp(1.0, 3.0), color);
    painter.line_segment(
        [
            Pos2::new(cx - w * 0.5, bar_y),
            Pos2::new(cx + w * 0.5, bar_y),
        ],
        stroke,
    );
}

/// Emit a one-time-per-block-type warning when no icon can be resolved.
static ICON_WARNED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn warn_missing_icon(block_type: &str, block_path: &str) {
    let warned = ICON_WARNED.get_or_init(|| Mutex::new(HashSet::new()));
    if let Ok(mut set) = warned.lock()
        && set.insert(block_type.to_string())
    {
        eprintln!(
            "\x1b[33m[rustylink] WARNING: {} at {} does not have a corresponding virtual library block\x1b[0m",
            block_type, block_path
        );
    }
}

/// Draw a single [`IconSpec`] centered in `rect`.
///
/// Shared by the config-map icon path (`render_block_icon`) and the
/// definition-driven icon path so the catalog definition's `icon` and the
/// legacy registry render identically.  The rendered glyph is maximized to fill
/// the available center area while leaving a margin to the block border and
/// avoiding overlap with optional inside-block port labels.
pub fn draw_icon_spec(
    painter: &egui::Painter,
    rect: &Rect,
    font_scale: f32,
    icon: &block_types::IconSpec,
    color: Color32,
    port_label_widths: Option<PortLabelMaxWidths>,
) {
    match icon {
        block_types::IconSpec::Utf8(glyph) => {
            render_center_glyph_maximized(
                painter,
                rect,
                font_scale,
                glyph,
                color,
                port_label_widths,
            );
        }
        block_types::IconSpec::Math(spec) => {
            draw_math_icon(painter, rect, font_scale, spec, color, port_label_widths);
        }
        block_types::IconSpec::Plot(spec) => {
            draw_plot_icon(painter, rect, font_scale, spec, color, port_label_widths);
        }
        block_types::IconSpec::Phosphor(name) => {
            let avail_rect = compute_icon_available_rect(rect, font_scale, port_label_widths);
            let avail_points = avail_rect.size();
            if avail_points.x <= 1.0 || avail_points.y <= 1.0 {
                return;
            }
            let font_id = egui::FontId::proportional(avail_points.y * 0.7);
            painter.text(
                avail_rect.center(),
                egui::Align2::CENTER_CENTER,
                name,
                font_id,
                color,
            );
        }
    }
}

/// Draw a single-glyph [`IconSpec`] (Utf8 / Phosphor) rotated 90° clockwise,
/// centered in `rect`.  Used only by the far-zoom dashboard fallback for the
/// Toggle/Rocker switches, which Simulink draws vertically.  Non-glyph specs
/// (Math / Plot) fall back to the unrotated [`draw_icon_spec`].
pub fn draw_icon_spec_rotated_quarter(
    painter: &egui::Painter,
    rect: &Rect,
    icon: &block_types::IconSpec,
    color: Color32,
) {
    let glyph: &str = match icon {
        block_types::IconSpec::Utf8(g) => g,
        block_types::IconSpec::Phosphor(n) => n,
        _ => {
            draw_icon_spec(painter, rect, 1.0, icon, color, None);
            return;
        }
    };
    let avail = compute_icon_available_rect(rect, 1.0, None);
    // The glyph is rotated, so its unrotated height must fit the available
    // width (and vice versa) — size against the smaller dimension.
    let target = avail.size().min_elem();
    if target <= 1.0 {
        return;
    }
    let font_id = egui::FontId::proportional(target * 0.7);
    let galley = painter.layout_no_wrap(glyph.to_owned(), font_id, color);
    let angle = std::f32::consts::FRAC_PI_2;
    let rot = egui::emath::Rot2::from_angle(angle);
    // TextShape rotates the galley around its `pos` (top-left); offset so the
    // galley's visual center lands on the available rect's center.
    let pos = avail.center() - rot * (galley.size() * 0.5);
    let mut shape = egui::epaint::TextShape::new(pos, galley, color);
    shape.angle = angle;
    painter.add(shape);
}

/// Contrast color for a glyph icon drawn on this block's background.
pub fn block_icon_color(block: &Block) -> Color32 {
    let cfg = get_block_type_cfg(block);
    crate::egui_app::ui::colors::contrast_color(crate::egui_app::ui::colors::block_base_color(
        block, &cfg,
    ))
}

pub fn render_block_icon(
    painter: &egui::Painter,
    block: &Block,
    rect: &Rect,
    font_scale: f32,
    icon_color: Color32,
    port_label_widths: Option<PortLabelMaxWidths>,
) {
    // Always prefer library-specific identifiers (library path / SourceBlock)
    // over generic `block_type` mappings.
    let cfg = get_block_type_cfg(block);
    // Glyph icons use the caller-provided contrast color (matching the actual
    // block fill); SVGs keep their own colors.
    let dark_icon = icon_color;
    if let Some(icon) = cfg.icon {
        draw_icon_spec(
            painter,
            rect,
            font_scale,
            &icon,
            dark_icon,
            port_label_widths,
        );
    } else {
        // No icon for this block.  Only warn for truly unknown blocks.
        // Known virtual-library blocks that simply lack a dedicated SVG
        // (e.g. "Is Hermitian", "Permute Matrix") are silently rendered
        // as "?" without a terminal warning.
        if !cfg.known {
            let raw_path = block
                .library_block_path
                .as_deref()
                .or_else(|| block.properties.get("SourceBlock").map(|s| s.as_str()))
                .unwrap_or("<unknown>");
            // Normalize the path for display: replace newlines with spaces so the
            // warning message is readable (SLX paths are word-wrapped with newlines).
            let block_path_display = raw_path.replace(['\n', '\r'], " ").replace('\\', "/");
            warn_missing_icon(&block.block_type, &block_path_display);
        }
        render_center_glyph_maximized(painter, rect, font_scale, "?", dark_icon, port_label_widths);
    }
}
