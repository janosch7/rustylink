//! Variant subsystem selection and the sim/codegen switching mode.

use crate::model::{Block, System};
use once_cell::sync::OnceCell;

// ────────────────────────────────────────────────────────────────────────────
// Variant configuration: sim vs codegen switching mode.
// ────────────────────────────────────────────────────────────────────────────

/// Which variant is active in "sim codegen switching" mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SimCodegenMode {
    /// Use the `(codegen)` variant (default).
    #[default]
    Codegen,
    /// Use the `(sim)` variant.
    Sim,
}

static VARIANT_CONFIG: OnceCell<std::sync::RwLock<SimCodegenMode>> = OnceCell::new();

/// Set the active variant for "sim codegen switching" mode.
///
/// Must be called before the first `ConnectionTargetResolver::new` call;
/// changing it after construction has no effect until the resolver is rebuilt.
pub fn set_sim_codegen_mode(mode: SimCodegenMode) {
    let config = VARIANT_CONFIG.get_or_init(|| std::sync::RwLock::new(SimCodegenMode::Codegen));
    if let Ok(mut guard) = config.write() {
        *guard = mode;
    }
}

/// Get the current sim/codegen switching mode (default: `Codegen`).
fn current_sim_codegen_mode() -> SimCodegenMode {
    VARIANT_CONFIG
        .get()
        .and_then(|c| c.read().ok())
        .map(|g| *g)
        .unwrap_or(SimCodegenMode::Codegen)
}

// ────────────────────────────────────────────────────────────────────────────
// Variant helpers
// ────────────────────────────────────────────────────────────────────────────

/// Whether a system is a variant subsystem (contains children with
/// `VariantControl` property).
pub(crate) fn is_variant_system(system: &System) -> bool {
    system
        .blocks
        .iter()
        .any(|b| b.properties.contains_key("VariantControl"))
}

/// Determine the SID of the active variant child subsystem, if it can be
/// determined statically from the XML.
///
/// Returns `None` when the active variant cannot be determined (expression
/// mode with non-literal controls, or ambiguous `true`/`false`).
pub(crate) fn active_variant_child_sid(
    system: &System,
    parent_block: Option<&Block>,
) -> Option<String> {
    let mode = parent_block
        .and_then(|b| b.properties.get("VariantControlMode"))
        .map(|s| s.trim());

    match mode {
        Some("label") => {
            let active_choice = parent_block
                .and_then(|b| b.properties.get("LabelModeActiveChoice"))
                .map(|s| s.trim())?;
            if active_choice.is_empty() {
                return None;
            }
            system
                .blocks
                .iter()
                .find(|b| {
                    b.properties.get("VariantControl").map(|s| s.trim()) == Some(active_choice)
                })
                .and_then(|b| b.sid.clone())
        }
        Some("sim codegen switching") => {
            let target = match current_sim_codegen_mode() {
                SimCodegenMode::Codegen => "(codegen)",
                SimCodegenMode::Sim => "(sim)",
            };
            system
                .blocks
                .iter()
                .find(|b| b.properties.get("VariantControl").map(|s| s.trim()) == Some(target))
                .and_then(|b| b.sid.clone())
        }
        _ => {
            // Expression mode (no VariantControlMode or unknown):
            // check for literal true/false.
            let true_children: Vec<&Block> = system
                .blocks
                .iter()
                .filter(|b| {
                    b.properties.contains_key("VariantControl")
                        && b.properties.get("VariantControl").map(|s| s.trim()) == Some("true")
                })
                .collect();
            if true_children.len() == 1 {
                true_children[0].sid.clone()
            } else {
                None
            }
        }
    }
}

/// Determine the 1-based port index of the active variant for
/// VariantStart/End/Sink/Source blocks, based on the `VariantControls`
/// array property.
///
/// Returns `None` when the active variant cannot be determined.
pub fn active_variant_port_index(block: &Block) -> Option<u32> {
    let controls = block.properties.get("VariantControls")?;
    let parts: Vec<&str> = controls.split(';').map(|s| s.trim()).collect();
    let mut active_indices: Vec<u32> = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        if *part == "true" {
            active_indices.push((i + 1) as u32);
        }
    }
    if active_indices.len() == 1 {
        Some(active_indices[0])
    } else {
        None
    }
}
