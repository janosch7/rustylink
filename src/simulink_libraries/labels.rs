//! Shared metadata-dependent label helpers used by block definitions.
//!
//! These functions implement the `MetadataDependent` variants of
//! [`BlockLabelPolicy`](super::types::BlockLabelPolicy) and
//! [`PortLabelPolicy`](super::types::PortLabelPolicy): they derive a label from
//! the block's extracted [`BlockMetadata`].

#![cfg(feature = "egui")]

use crate::model::Block;

use super::metadata::BlockMetadata;

/// Block label = the `Gain` property value.
pub fn gain_value(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    meta.get("Gain")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Block label = the `GotoTag` property value (defaults to `"A"`).
pub fn goto_tag(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    Some(
        meta.get("GotoTag")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or("A")
            .to_string(),
    )
}

/// Logical Operator label: reads `Operator` property (AND/OR/NOT/NAND/NOR/XOR).
pub fn logic_operator(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    Some(
        meta.get("Operator")
            .map(|s| s.trim().to_uppercase())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "AND".to_string()),
    )
}

/// Relational Operator label: maps codes to math symbols.
pub fn relational_operator(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    Some(
        meta.get("Operator")
            .map(|s| {
                match s.trim() {
                    "<=" => "\u{2264}", // ≤
                    ">=" => "\u{2265}", // ≥
                    "==" => "=",
                    "~=" => "\u{2260}", // ≠
                    other => other,
                }
                .to_string()
            })
            .unwrap_or_else(|| "==".to_string()),
    )
}

/// Math function label: reads `Operator` (exp, log, sqrt, conj, …).
pub fn math_function(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    meta.get("Operator")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| Some("exp".to_string()))
}

/// Trigonometry function label: reads `Operator` (sin, cos, acos, atan2, …).
pub fn trig_function(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    meta.get("Operator")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| Some("sin".to_string()))
}

/// MinMax label: reads `Function` property (min/max).
pub fn minmax_function(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    Some(
        meta.get("Function")
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "min".to_string()),
    )
}
