//! The block browser and the code editor windows.

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
use eframe::egui::{self, Color32, RichText};
use std::collections::HashMap;
// ────────────────────────────────────────────────────────────────────────────
// Color utilities — re-exported from canonical `egui_app::ui::colors`
// ────────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────────
// Block browser window
// ────────────────────────────────────────────────────────────────────────────

pub(super) fn show_block_browser(state: &mut EditorState, ui: &mut egui::Ui) {
    if !state.block_browser.open {
        return;
    }

    let mut open = state.block_browser.open;
    let insert_x = state.block_browser.insert_x;
    let insert_y = state.block_browser.insert_y;

    egui::Window::new("Add Block")
        .open(&mut open)
        .default_size([350.0, 500.0])
        .resizable(true)
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut state.block_browser.query);
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                let query = state.block_browser.query.clone();
                let categories = state.block_browser.categories.clone();
                let expanded = state.block_browser.expanded_category;
                for (cat_idx, cat) in categories.iter().enumerate() {
                    let matching: Vec<_> = cat
                        .entries
                        .iter()
                        .filter(|e| query.is_empty() || e.matches_query(&query))
                        .collect();
                    if matching.is_empty() {
                        continue;
                    }

                    let is_expanded = expanded == Some(cat_idx) || !query.is_empty();

                    let header = egui::CollapsingHeader::new(
                        RichText::new(format!("{} ({})", cat.name, matching.len())).strong(),
                    )
                    .default_open(is_expanded);
                    header.show(ui, |ui| {
                        for entry in matching {
                            let label = format!("{} — {}", entry.display_name, entry.description);
                            if ui
                                .button(&entry.display_name)
                                .on_hover_text(&label)
                                .clicked()
                            {
                                // Add block to current system
                                if let Some(system) =
                                    crate::editor::state::resolve_subsystem_by_vec_mut(
                                        &mut state.app.root,
                                        &state.app.path,
                                    )
                                {
                                    let block = operations::create_default_block(
                                        &entry.block_type,
                                        &entry.display_name,
                                        insert_x,
                                        insert_y,
                                        entry.default_inputs,
                                        entry.default_outputs,
                                    );
                                    let cmd = operations::add_block(system, block);
                                    state.history.push(cmd);
                                    state.dirty = true;
                                    state.app.show_notification(
                                        format!("Added {}", entry.display_name),
                                        2000,
                                    );
                                }
                                state.block_browser.close();
                            }
                        }
                    });
                }
            });
        });

    state.block_browser.open = open;
}

// ────────────────────────────────────────────────────────────────────────────
// Code editor window
// ────────────────────────────────────────────────────────────────────────────

pub(super) fn show_code_editor(state: &mut EditorState, ui: &mut egui::Ui) {
    if !state.code_editor.open {
        return;
    }

    let mut open = state.code_editor.open;

    let title = format!(
        "Code: {}{}",
        state.code_editor.block_name,
        if state.code_editor.is_modified() {
            " *"
        } else {
            ""
        },
    );

    egui::Window::new(title)
        .open(&mut open)
        .default_size([600.0, 400.0])
        .resizable(true)
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if ui.button("Apply").clicked() {
                    // Save code back to block
                    let block_index = state.code_editor.block_index;
                    let code = state.code_editor.code.clone();
                    if let Some(system) = crate::editor::state::resolve_subsystem_by_vec_mut(
                        &mut state.app.root,
                        &state.app.path,
                    ) && let Some(block) = system.blocks.get_mut(block_index)
                    {
                        set_block_code(block, &code);
                        state.mark_dirty();
                        state.app.show_notification("Code applied", 1500);
                    }
                    state.code_editor.original_code = code;
                }
                if ui.button("Revert").clicked() {
                    state.code_editor.code = state.code_editor.original_code.clone();
                }
                if state.code_editor.is_modified() {
                    ui.colored_label(Color32::from_rgb(255, 200, 80), "Modified");
                }
            });
            ui.separator();

            // Code text area with syntax highlighting
            let theme = egui::TextEdit::multiline(&mut state.code_editor.code)
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
                .desired_rows(20);
            ui.add(theme);
        });

    state.code_editor.open = open;
}

// ────────────────────────────────────────────────────────────────────────────
// Helper functions
// ────────────────────────────────────────────────────────────────────────────

pub fn is_code_block(block: &crate::model::Block) -> bool {
    crate::simulink_libraries::traits::carries_code(block)
}

pub fn is_subsystem_block(block: &crate::model::Block) -> bool {
    crate::simulink_libraries::traits::is_navigable_subsystem(block)
}

pub(super) fn open_code_editor(
    state: &mut EditorState,
    block_idx: usize,
    block: &crate::model::Block,
) {
    let code = get_block_code(block);
    state
        .code_editor
        .open_for_block(block_idx, &block.name, &code);
}

pub fn get_block_code(block: &crate::model::Block) -> String {
    // Try Script property (MATLAB Function), then Code (CFunction)
    if let Some(script) = block.properties.get("Script") {
        return script.clone();
    }
    if let Some(code) = block.properties.get("Code") {
        return code.clone();
    }
    if let Some(expr) = block.properties.get("Expr") {
        return expr.clone();
    }
    String::new()
}

pub fn set_block_code(block: &mut crate::model::Block, code: &str) {
    if block.properties.contains_key("Script") {
        block
            .properties
            .insert("Script".to_string(), code.to_string());
    } else if block.properties.contains_key("Code") {
        block
            .properties
            .insert("Code".to_string(), code.to_string());
    } else if block.properties.contains_key("Expr") {
        block
            .properties
            .insert("Expr".to_string(), code.to_string());
    } else {
        // Default to Script
        block
            .properties
            .insert("Script".to_string(), code.to_string());
    }
}

pub(super) fn handle_block_double_click(
    state: &mut EditorState,
    block_idx: usize,
    block: &crate::model::Block,
    subsystem_block_lookup: &HashMap<String, crate::model::Block>,
) {
    if is_code_block(block) {
        open_code_editor(state, block_idx, block);
    } else if block
        .sid
        .as_ref()
        .is_some_and(|sid| subsystem_block_lookup.contains_key(sid))
    {
        let full_block: crate::model::Block = block
            .sid
            .as_ref()
            .and_then(|sid| subsystem_block_lookup.get(sid))
            .cloned()
            .unwrap_or_else(|| block.clone());
        state.app.open_block_if_subsystem(&full_block);
        state.selection.clear();
    }
}
