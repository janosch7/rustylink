//! Right-click context menus of blocks, lines and the canvas.

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
use crate::egui_app::{BlockDialog, SignalDialog};
use eframe::egui::{self, Pos2};
use std::collections::HashMap;
use std::sync::Arc;
// ────────────────────────────────────────────────────────────────────────────
// Color utilities — re-exported from canonical `egui_app::ui::colors`
// ────────────────────────────────────────────────────────────────────────────

use super::panels::{is_code_block, open_code_editor};

// ────────────────────────────────────────────────────────────────────────────
// Context menus
// ────────────────────────────────────────────────────────────────────────────

pub(super) fn block_context_menu(
    state: &mut EditorState,
    ui: &mut egui::Ui,
    block_idx: usize,
    block: &crate::model::Block,
    subsystem_block_lookup: &HashMap<String, crate::model::Block>,
) {
    if ui.button("Delete").clicked() {
        state.selection.select_block(block_idx);
        state.delete_selection();
        ui.close();
    }
    if ui.button("Comment / Uncomment").clicked() {
        state.selection.select_block(block_idx);
        state.comment_selection();
        ui.close();
    }
    if ui.button("Rotate").clicked() {
        state.selection.select_block(block_idx);
        state.rotate_selection();
        ui.close();
    }
    if ui.button("Mirror").clicked() {
        state.selection.select_block(block_idx);
        state.mirror_selection();
        ui.close();
    }
    ui.separator();
    if ui.button("Copy").clicked() {
        state.selection.select_block(block_idx);
        state.copy_selection();
        ui.close();
    }
    ui.separator();
    if is_code_block(block) {
        if ui.button("Edit Code…").clicked() {
            open_code_editor(state, block_idx, block);
            ui.close();
        }
        ui.separator();
    }
    let is_subsystem = block
        .sid
        .as_ref()
        .is_some_and(|sid| subsystem_block_lookup.contains_key(sid));
    if is_subsystem && ui.button("Open Subsystem").clicked() {
        let full_block: crate::model::Block = block
            .sid
            .as_ref()
            .and_then(|sid| subsystem_block_lookup.get(sid))
            .cloned()
            .unwrap_or_else(|| block.clone());
        state.app.open_block_if_subsystem(&full_block);
        state.selection.clear();
        ui.close();
    }
    if !state.selection.selected_blocks.is_empty()
        && state.selection.selected_blocks.len() > 1
        && ui.button("Create Subsystem from Selection…").clicked()
    {
        let name = format!(
            "Subsystem{}",
            state.current_system().map_or(0, |s| s.blocks.len())
        );
        state.create_subsystem_from_selection(&name);
        ui.close();
    }
    ui.separator();
    if ui.button("Properties…").clicked() {
        // Show block info
        state.app.block_view = Some(BlockDialog {
            title: format!("Block: {}", block.name),
            block: Arc::new(block.clone()),
            open: true,
        });
        ui.close();
    }
}

pub(super) fn line_context_menu(
    state: &mut EditorState,
    ui: &mut egui::Ui,
    line_idx: usize,
    line: &crate::model::Line,
) {
    if ui.button("Delete").clicked() {
        state.selection.select_line(line_idx);
        state.delete_selection();
        ui.close();
    }
    ui.separator();
    // Rename label
    if ui.button("Rename Label…").clicked() {
        // For now, just set a default label (a dialog would be better in a real app)
        if let Some(system) = state.current_system_mut() {
            let new_name = if line.name.is_some() {
                None // Toggle off
            } else {
                Some(format!("signal_{}", line_idx))
            };
            let cmd = operations::rename_line(system, line_idx, new_name);
            state.history.push(cmd);
            state.mark_dirty();
        }
        ui.close();
    }
    ui.separator();
    if ui.button("Properties…").clicked() {
        state.app.signal_view = Some(SignalDialog {
            title: format!("Signal: {}", line.name.as_deref().unwrap_or("<unnamed>")),
            line_idx,
            open: true,
        });
        ui.close();
    }
}

pub(super) fn canvas_context_menu(
    state: &mut EditorState,
    ui: &mut egui::Ui,
    from_screen: &dyn Fn(Pos2) -> Pos2,
    canvas_resp: &egui::Response,
) {
    if ui.button("Add Block… (A)").clicked() {
        let pos = canvas_resp
            .hover_pos()
            .map(from_screen)
            .unwrap_or(Pos2::new(200.0, 200.0));
        state.block_browser.open_at(pos.x as i32, pos.y as i32);
        ui.close();
    }
    if ui.button("Paste").clicked() {
        state.paste();
        ui.close();
    }
    ui.separator();
    if ui.button("Select All").clicked() {
        let counts = crate::egui_app::resolve_subsystem_by_vec(&state.app.root, &state.app.path)
            .map(|s| (s.blocks.len(), s.lines.len()));
        if let Some((nb, nl)) = counts {
            state.selection.selected_blocks = (0..nb).collect();
            state.selection.selected_lines = (0..nl).collect();
        }
        ui.close();
    }
    ui.separator();
    if ui.button("Reassign SIDs").clicked() {
        if let Some(system) =
            crate::editor::state::resolve_subsystem_by_vec_mut(&mut state.app.root, &state.app.path)
        {
            let cmd = operations::assign_sids(system);
            state.history.push(cmd);
            state.dirty = true;
            state.app.show_notification("SIDs reassigned", 2000);
        }
        ui.close();
    }
}
