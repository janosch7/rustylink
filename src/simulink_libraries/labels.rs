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
