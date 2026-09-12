//! Keyboard shortcuts of the editor canvas.

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

use super::operations;
use crate::editor::state::EditorState;
use eframe::egui::{self, Rect};
// ────────────────────────────────────────────────────────────────────────────
// Color utilities — re-exported from canonical `egui_app::ui::colors`
// ────────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────────
// Keyboard shortcuts
// ────────────────────────────────────────────────────────────────────────────

pub(super) fn handle_keyboard_shortcuts(
    state: &mut EditorState,
    ui: &mut egui::Ui,
    _avail: &Rect,
    _base_scale: f32,
    _bb: &Rect,
) {
    let input = ui.input(|i| {
        (
            i.modifiers.ctrl,
            i.modifiers.shift,
            i.key_pressed(egui::Key::Z),
            i.key_pressed(egui::Key::Y),
            i.key_pressed(egui::Key::Delete),
            i.key_pressed(egui::Key::A),
            i.key_pressed(egui::Key::C),
            i.key_pressed(egui::Key::V),
            i.key_pressed(egui::Key::R),
            i.key_pressed(egui::Key::M),
            i.key_pressed(egui::Key::ArrowUp),
            i.key_pressed(egui::Key::ArrowDown),
            i.key_pressed(egui::Key::ArrowLeft),
            i.key_pressed(egui::Key::ArrowRight),
            i.key_pressed(egui::Key::Escape),
        )
    });
    let (ctrl, _shift, z, y, delete, a, c, v, r, m, up, down, left, right, escape) = input;

    // Ctrl+Z: Undo
    if ctrl && z {
        state.undo();
    }
    // Ctrl+Y: Redo
    if ctrl && y {
        state.redo();
    }
    // Delete: Delete selection
    if delete {
        state.delete_selection();
    }
    // A: Open block browser
    if a && !ctrl {
        state.block_browser.open_at(200, 200);
    }
    // Ctrl+C: Copy
    if ctrl && c {
        state.copy_selection();
    }
    // Ctrl+V: Paste
    if ctrl && v {
        state.paste();
    }
    // R: Rotate selection
    if r && !ctrl {
        state.rotate_selection();
    }
    // M: Mirror selection
    if m && !ctrl {
        state.mirror_selection();
    }
    // Arrow keys: Move selected blocks
    let arrow_step = if ctrl { 1 } else { 5 };
    if !state.selection.selected_blocks.is_empty() {
        let (adx, ady) = match (up, down, left, right) {
            (true, _, _, _) => (0, -arrow_step),
            (_, true, _, _) => (0, arrow_step),
            (_, _, true, _) => (-arrow_step, 0),
            (_, _, _, true) => (arrow_step, 0),
            _ => (0, 0),
        };
        if adx != 0 || ady != 0 {
            let indices = state.selection.selected_blocks.clone();
            if let Some(system) = crate::editor::state::resolve_subsystem_by_vec_mut(
                &mut state.app.root,
                &state.app.path,
            ) {
                let cmd = operations::move_blocks(system, &indices, adx, ady);
                state.history.push(cmd);
                state.dirty = true;
            }
        }
    }
    // Escape: Clear selection / close browser
    if escape {
        state.selection.clear();
        state.block_browser.close();
        state.code_editor.close();
    }
}
