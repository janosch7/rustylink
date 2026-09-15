//! The eight resize handles of a selected block.

#![cfg(feature = "egui")]

//! Editor UI — the interactive egui interface for model editing.
//!
//! This module provides the main rendering and interaction functions for
//! the Simulink model editor. It extends the viewer UI with:
//!
//! - Block dragging with arrow-key support
//! - Connection drawing with auto-snap to ports
//! - Rectangle selection of blocks and lines
//! - Block browser popup (hotkey "A")
//! - Context menus for blocks, lines, and canvas
//! - Code editor for MATLAB Function / CFunction blocks
//! - Keyboard shortcuts (Ctrl+Z/Y, Delete, Ctrl+C/V, R, M, etc.)
//! - Grid overlay

use crate::editor::state::{DragMode, EditorState};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
// ────────────────────────────────────────────────────────────────────────────
// Color utilities — re-exported from canonical `egui_app::ui::colors`
// ────────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────────
// Resize handles
// ────────────────────────────────────────────────────────────────────────────

/// Compute the 8 resize handle positions for a screen-space rectangle.
/// Returns [(center_pos, handle_index)] for TL, T, TR, R, BR, B, BL, L.
pub(super) fn resize_handle_positions(r: &Rect) -> [(Pos2, u8); 8] {
    let cx = r.center().x;
    let cy = r.center().y;
    [
        (r.left_top(), 0),              // TL
        (Pos2::new(cx, r.top()), 1),    // T
        (r.right_top(), 2),             // TR
        (Pos2::new(r.right(), cy), 3),  // R
        (r.right_bottom(), 4),          // BR
        (Pos2::new(cx, r.bottom()), 5), // B
        (r.left_bottom(), 6),           // BL
        (Pos2::new(r.left(), cy), 7),   // L
    ]
}

/// Draw resize handles on a selected block and handle interaction.
pub(super) fn draw_resize_handles(
    ui: &mut egui::Ui,
    r_screen: &Rect,
    block_idx: usize,
    state: &mut EditorState,
    model_rect: &Rect,
) {
    let handle_size = 5.0;
    let handle_color = Color32::from_rgb(0, 120, 255);
    let handle_hover_color = Color32::from_rgb(80, 180, 255);

    let handles = resize_handle_positions(r_screen);

    for (pos, handle_id) in &handles {
        let handle_rect = Rect::from_center_size(*pos, Vec2::splat(handle_size * 2.0));
        let resp = ui.allocate_rect(handle_rect, Sense::click_and_drag());

        let color = if resp.hovered() || resp.dragged() {
            handle_hover_color
        } else {
            handle_color
        };

        // Draw handle square
        ui.painter().rect_filled(
            Rect::from_center_size(*pos, Vec2::splat(handle_size)),
            0.0,
            color,
        );
        ui.painter().rect_stroke(
            Rect::from_center_size(*pos, Vec2::splat(handle_size)),
            0.0,
            Stroke::new(1.0_f32, Color32::WHITE),
            egui::StrokeKind::Outside,
        );

        // Start resize drag
        if resp.drag_started() {
            let (l, t, r, b) = (
                model_rect.left() as i32,
                model_rect.top() as i32,
                model_rect.right() as i32,
                model_rect.bottom() as i32,
            );
            state.drag_mode = DragMode::Resize {
                block_index: block_idx,
                handle: *handle_id,
                original_l: l,
                original_t: t,
                original_r: r,
                original_b: b,
                dx: 0.0,
                dy: 0.0,
            };
        }
    }
}

/// Compute the new rect after applying a resize delta from a specific handle.
/// Returns (new_l, new_t, new_r, new_b) with minimum size enforcement and grid snapping.
#[allow(clippy::too_many_arguments)]
pub(super) fn compute_resized_rect(
    l: f32,
    t: f32,
    r: f32,
    b: f32,
    handle: u8,
    dx: f32,
    dy: f32,
    grid_size: i32,
    snap_to_grid: bool,
) -> (f32, f32, f32, f32) {
    let min_size = 10.0;
    let snap = |v: f32| -> f32 {
        if snap_to_grid && grid_size > 0 {
            ((v / grid_size as f32).round()) * grid_size as f32
        } else {
            v
        }
    };

    let (mut nl, mut nt, mut nr, mut nb) = (l, t, r, b);

    match handle {
        0 => {
            // TL
            nl = snap(l + dx);
            nt = snap(t + dy);
        }
        1 => {
            // T
            nt = snap(t + dy);
        }
        2 => {
            // TR
            nr = snap(r + dx);
            nt = snap(t + dy);
        }
        3 => {
            // R
            nr = snap(r + dx);
        }
        4 => {
            // BR
            nr = snap(r + dx);
            nb = snap(b + dy);
        }
        5 => {
            // B
            nb = snap(b + dy);
        }
        6 => {
            // BL
            nl = snap(l + dx);
            nb = snap(b + dy);
        }
        7 => {
            // L
            nl = snap(l + dx);
        }
        _ => {}
    }

    // Enforce minimum size
    if nr - nl < min_size {
        if handle == 0 || handle == 6 || handle == 7 {
            nl = nr - min_size;
        } else {
            nr = nl + min_size;
        }
    }
    if nb - nt < min_size {
        if handle == 0 || handle == 1 || handle == 2 {
            nt = nb - min_size;
        } else {
            nb = nt + min_size;
        }
    }

    (nl, nt, nr, nb)
}
