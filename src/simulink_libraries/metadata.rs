//! Per-instance block metadata.
//!
//! When a block is matched to its [`SimulinkBlockDefinition`], the renderer
//! extracts the model data it cares about into a [`BlockMetadata`] – a small
//! `HashMap` of resolved key/value pairs.  Label and icon policies, as well as
//! custom renderers, read from this map instead of digging through raw block
//! properties on every frame.  This keeps the renderer general (no
//! block-specific property access) and makes the matched metadata explicit.

#![cfg(feature = "egui")]

use std::collections::HashMap;

use crate::model::Block;

use super::types::SimulinkBlockDefinition;

/// Resolved model data for a single block instance, keyed by name.
///
/// Values are stored as strings (the native form in the SLX model) and parsed
/// on demand via the typed accessors.
#[derive(Clone, Debug, Default)]
pub struct BlockMetadata {
    values: HashMap<String, String>,
}

impl BlockMetadata {
    /// Look up a raw string value previously extracted for this block.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    /// Insert a value into the metadata map.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.values.insert(key.into(), value.into());
    }

    /// Parse a value as `f64`, if present and numeric.
    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| v.trim().parse::<f64>().ok())
    }

    /// Number of stored entries.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether no metadata was extracted.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// Extract the metadata declared by a definition from the parsed block.
///
/// This is the single, general property-parsing entry point: for each
/// `metadata_keys` entry it copies the value from `block.properties`, then from
/// the block's `InstanceData` (where library-reference blocks such as `Bit Set`
/// store their mask parameters), falling back to the entry's declared default
/// when the model omits the property (e.g. a `Constant` with no `Value`
/// property resolves to `"1"`).  The optional `metadata_fn` hook then runs for
/// any computed values.
pub fn extract_metadata(block: &Block, def: &SimulinkBlockDefinition) -> BlockMetadata {
    let mut meta = BlockMetadata::default();
    for mk in def.metadata_keys {
        let instance_value = block
            .instance_data
            .as_ref()
            .and_then(|id| id.properties.get(mk.key));
        if let Some(value) = block.properties.get(mk.key).or(instance_value) {
            meta.insert(mk.key, value.clone());
        } else if let Some(default) = mk.default {
            meta.insert(mk.key, default);
        }
    }
    if let Some(f) = def.metadata_fn {
        f(block, &mut meta);
    }
    meta
}
