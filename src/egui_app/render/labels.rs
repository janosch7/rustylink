//! Port label names and the text wrapping used for block captions.

#![cfg(feature = "egui")]

use crate::block_types::BlockTypeConfig;
use crate::model::Block;
use eframe::egui::{self, Color32};

/// Max measured width of port labels drawn *inside* the block on the left/right side.
///
/// This is used to keep the center icon from overlapping those labels.
#[derive(Clone, Copy, Debug, Default)]
pub struct PortLabelMaxWidths {
    pub left: f32,
    pub right: f32,
}

#[allow(dead_code)]
pub(crate) fn port_label_display_name(
    block: &Block,
    index: u32,
    is_input: bool,
    cfg: &BlockTypeConfig,
) -> String {
    // Note: The port-label drawing code treats mirroring as swapping the logical direction
    // when looking up Port properties. Keep this logic in one place so icon sizing and
    // label rendering stay consistent.
    let mirrored = block.block_mirror.unwrap_or(false);
    let logical_is_input = if mirrored { !is_input } else { is_input };

    let fallback_name = || {
        let names = if logical_is_input {
            &cfg.input_port_names
        } else {
            &cfg.output_port_names
        };
        if index > 0 && (index as usize) <= names.len() {
            names[(index - 1) as usize].clone()
        } else {
            format!("{}{}", if is_input { "In" } else { "Out" }, index)
        }
    };

    let explicit_port_name = || {
        block
            .ports
            .iter()
            .filter(|p| {
                p.port_type == if logical_is_input { "in" } else { "out" }
                    && p.index.unwrap_or(0) == index
            })
            .find_map(|p| {
                p.properties
                    .get("Name")
                    .cloned()
                    .or_else(|| p.properties.get("name").cloned())
                    .map(|name| name.trim().to_string())
                    .filter(|name| !name.is_empty())
            })
    };

    subsystem_boundary_port_name(block, index, logical_is_input)
        .or_else(|| crate::simulink_libraries::render::port_label(block, index, logical_is_input))
        .or_else(explicit_port_name)
        .unwrap_or_else(fallback_name)
}

pub(crate) fn subsystem_boundary_port_name(
    block: &Block,
    index: u32,
    logical_is_input: bool,
) -> Option<String> {
    if !crate::simulink_libraries::traits::is_container(block) {
        return None;
    }
    let boundary_role = if logical_is_input {
        crate::simulink_libraries::traits::SignalRole::BoundaryInput
    } else {
        crate::simulink_libraries::traits::SignalRole::BoundaryOutput
    };

    block
        .subsystem
        .as_ref()?
        .blocks
        .iter()
        .filter(|child| {
            crate::simulink_libraries::traits::block_traits(&child.block_type).signal_role
                == boundary_role
        })
        .find(|child| subsystem_boundary_port_index(child) == index)
        .and_then(|child| boundary_block_display_name(child, index))
}

fn subsystem_boundary_port_index(block: &Block) -> u32 {
    block
        .properties
        .get("Port")
        .or_else(|| block.properties.get("PortNumber"))
        .and_then(|value| value.trim().parse::<u32>().ok())
        .unwrap_or(1)
}

/// Replace Simulink's default `In<N>` / `Out<N>` boundary-block naming with the
/// port *number* (what the Inport block's own icon draws), while user-chosen
/// names such as `u` or `theta` are kept verbatim.  The number is the port's,
/// not the one in the block name: reordering the ports of a subsystem renumbers
/// them while the boundary blocks keep the names they were created with.
fn simplify_boundary_name(name: &str, port_index: u32) -> String {
    for prefix in ["In", "Out"] {
        if let Some(rest) = name.strip_prefix(prefix)
            && !rest.is_empty()
            && rest.chars().all(|c| c.is_ascii_digit())
        {
            return port_index.to_string();
        }
    }
    name.to_string()
}

fn boundary_block_display_name(block: &Block, port_index: u32) -> Option<String> {
    let name = block.name.trim();
    if !name.is_empty() {
        return Some(simplify_boundary_name(name, port_index));
    }

    block
        .properties
        .get("Name")
        .or_else(|| block.properties.get("name"))
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
}

pub fn wrap_text_to_max_width(
    painter: &egui::Painter,
    text: &str,
    font_id: egui::FontId,
    max_width: f32,
) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    if !max_width.is_finite() || max_width <= 1.0 {
        return text.split('\n').map(|s| s.to_string()).collect();
    }

    fn measure_width(painter: &egui::Painter, s: &str, font_id: &egui::FontId) -> f32 {
        painter
            .layout_no_wrap(s.to_string(), font_id.clone(), Color32::TRANSPARENT)
            .size()
            .x
    }

    fn split_prefix_that_fits<'a>(
        painter: &egui::Painter,
        word: &'a str,
        font_id: &egui::FontId,
        max_width: f32,
    ) -> (&'a str, &'a str) {
        if word.is_empty() {
            return ("", "");
        }

        let mut boundaries: Vec<usize> = word.char_indices().map(|(i, _)| i).collect();
        boundaries.push(word.len());
        if boundaries.len() <= 2 {
            // One character (or empty) — must make progress.
            return (word, "");
        }

        let mut best = 1usize; // at least one char
        let mut lo = 1usize;
        let mut hi = boundaries.len();
        while lo < hi {
            let mid = (lo + hi) / 2;
            let idx = boundaries[mid];
            let prefix = &word[..idx];
            if measure_width(painter, prefix, font_id) <= max_width {
                best = mid;
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }

        let split_idx = boundaries[best];
        (&word[..split_idx], &word[split_idx..])
    }

    let mut out: Vec<String> = Vec::new();
    for para in text.split('\n') {
        // Preserve explicit newlines.
        if para.trim().is_empty() {
            out.push(String::new());
            continue;
        }

        let mut current = String::new();
        for word in para.split_whitespace() {
            if current.is_empty() {
                if measure_width(painter, word, &font_id) <= max_width {
                    current.push_str(word);
                } else {
                    // Extremely long word: split by character to guarantee progress.
                    let mut rest = word;
                    while !rest.is_empty() {
                        let (prefix, new_rest) =
                            split_prefix_that_fits(painter, rest, &font_id, max_width);
                        out.push(prefix.to_string());
                        rest = new_rest;
                    }
                }
                continue;
            }

            let candidate = format!("{} {}", current, word);
            if measure_width(painter, &candidate, &font_id) <= max_width {
                current = candidate;
            } else {
                out.push(current);
                current = String::new();

                if measure_width(painter, word, &font_id) <= max_width {
                    current.push_str(word);
                } else {
                    let mut rest = word;
                    while !rest.is_empty() {
                        let (prefix, new_rest) =
                            split_prefix_that_fits(painter, rest, &font_id, max_width);
                        out.push(prefix.to_string());
                        rest = new_rest;
                    }
                }
            }
        }

        if !current.is_empty() {
            out.push(current);
        }
    }

    out
}
