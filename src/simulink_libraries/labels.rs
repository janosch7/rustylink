//! Shared metadata-dependent label helpers used by block definitions.
//!
//! These functions implement the `MetadataDependent` variants of
//! [`BlockLabelPolicy`](super::types::BlockLabelPolicy) and
//! [`PortLabelPolicy`](super::types::PortLabelPolicy): they derive a label from
//! the block's extracted [`BlockMetadata`].
//!
//! Property defaults (the value shown when the model omits the property) live in
//! each block's `metadata_keys` via
//! [`MetadataKey::with_default`](super::types::MetadataKey::with_default), so
//! these helpers are thin readers and never hard-code fallbacks themselves.

#![cfg(feature = "egui")]

use crate::model::Block;

use super::metadata::BlockMetadata;

/// Block label = the `Gain` property value.
pub fn gain_value(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    nonempty(meta.get("Gain"))
}

/// Block label = the `Value` property value (a `Constant`'s output value).
pub fn constant_value(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    nonempty(meta.get("Value"))
}

/// Block label = the `GotoTag` property value.
pub fn goto_tag(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    nonempty(meta.get("GotoTag"))
}

/// Logical Operator label: reads `Operator` property (AND/OR/NOT/NAND/NOR/XOR).
pub fn logic_operator(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    nonempty(meta.get("Operator")).map(|s| s.to_uppercase())
}

/// Relational Operator label: maps codes to math symbols.
pub fn relational_operator(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    nonempty(meta.get("Operator")).map(|s| {
        match s.as_str() {
            "<=" => "\u{2264}", // ≤
            ">=" => "\u{2265}", // ≥
            "==" => "=",
            "~=" => "\u{2260}", // ≠
            _ => s.as_str(),
        }
        .to_string()
    })
}

/// Math function label: reads `Operator` (exp, log, sqrt, conj, …).
pub fn math_function(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    nonempty(meta.get("Operator"))
}

/// Trigonometry function label: reads `Operator` (sin, cos, acos, atan2, …).
pub fn trig_function(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    nonempty(meta.get("Operator"))
}

/// MinMax label: reads `Function` property (min/max).
pub fn minmax_function(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    nonempty(meta.get("Function")).map(|s| s.to_lowercase())
}

/// Instance label for a `Compare To Constant` block, derived from its
/// `InstanceData` (`relop`/`const`).  Returns e.g. `"≤ 3.0"`, or `None` when the
/// parameters are absent.
pub fn compare_to_constant(block: &Block) -> Option<String> {
    let id = block.instance_data.as_ref()?;
    let relop = id.properties.get("relop")?;
    let const_val = id.properties.get("const")?;
    let sym = match relop.trim() {
        "<=" => "\u{2264}",
        ">=" => "\u{2265}",
        "~=" => "\u{2260}",
        "==" => "=",
        other => other,
    };
    Some(format!("{} {}", sym, const_val.trim()))
}

/// Trim a metadata value and discard it if empty.
fn nonempty(value: Option<&str>) -> Option<String> {
    value
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
