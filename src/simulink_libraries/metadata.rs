//! Per-instance block metadata.
//!
//! When a block is matched to its [`SimulinkBlockDefinition`], the renderer
//! extracts the model data it cares about into a [`BlockMetadata`] – a small
//! `HashMap` of resolved key/value pairs.  Label and icon policies, as well as
//! custom renderers, read from this map instead of digging through raw block
//! properties on every frame.  This keeps the renderer general (no
//! block-specific property access) and makes the matched metadata explicit.

#![cfg(feature = "egui")]

use std::borrow::Cow;

use crate::model::Block;

use super::types::SimulinkBlockDefinition;

/// Resolved model data for a single block instance, keyed by name.
///
/// Values are stored as strings (the native form in the SLX model) and parsed
/// on demand via the typed accessors.  A definition declares a handful of keys
/// at most, so the entries live in a `Vec` scanned linearly: building one map
/// per block per frame costs more than the scan saves.
#[derive(Clone, Debug, Default)]
pub struct BlockMetadata {
    values: Vec<(Cow<'static, str>, String)>,
}

impl BlockMetadata {
    /// Look up a raw string value previously extracted for this block.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Insert a value into the metadata map.
    pub fn insert(&mut self, key: impl Into<Cow<'static, str>>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        match self.values.iter_mut().find(|(k, _)| *k == key) {
            Some(entry) => entry.1 = value,
            None => self.values.push((key, value)),
        }
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
