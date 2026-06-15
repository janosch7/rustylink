//! Block-browser catalog: a searchable, categorized view over the catalog.
//!
//! The browser is a *view* over the single unified Simulink block-definition
//! catalog (`crate::simulink_libraries`).  Every searchable, categorized entry
//! shown in the palette is derived from a [`SimulinkBlockDefinition`], so the
//! editor and the renderer share one source of truth.  To add your own block to
//! the browser, add a definition to the catalog (see
//! `simulink_libraries::libraries`); it shows up here automatically.
//!
//! # Usage
//!
//! ```rust,ignore
//! use rustylink::simulink_libraries::browser::{get_block_catalog, BlockCatalogEntry};
//!
//! let catalog = get_block_catalog();
//! // Search for blocks matching "gain"
//! let matches: Vec<&BlockCatalogEntry> = catalog
//!     .iter()
//!     .filter(|e| e.matches_query("gain"))
//!     .collect();
//! ```

#![cfg(feature = "egui")]

use once_cell::sync::Lazy;

use crate::simulink_libraries::resolver::registry;

/// A single entry in the block catalog.
#[derive(Debug, Clone)]
pub struct BlockCatalogEntry {
    /// Internal block type name (e.g., `"Gain"`, `"SubSystem"`).
    pub block_type: String,
    /// Human-readable display name shown in the browser.
    pub display_name: String,
    /// Category path (e.g., `"Math Operations"`, `"Signal Routing"`).
    pub category: String,
    /// Default number of input ports.
    pub default_inputs: u32,
    /// Default number of output ports.
    pub default_outputs: u32,
    /// Brief description of the block's function.
    pub description: String,
}

impl BlockCatalogEntry {
    /// Check if this entry matches a search query (case-insensitive substring match
    /// on block type, display name, category, or description).
    pub fn matches_query(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        let q = query.to_lowercase();
        self.block_type.to_lowercase().contains(&q)
            || self.display_name.to_lowercase().contains(&q)
            || self.category.to_lowercase().contains(&q)
            || self.description.to_lowercase().contains(&q)
    }
}

/// A category of blocks in the catalog, with a name and list of entries.
#[derive(Debug, Clone)]
pub struct BlockCatalogCategory {
    /// Category display name.
    pub name: String,
    /// Entries belonging to this category.
    pub entries: Vec<BlockCatalogEntry>,
}

/// Returns the complete block catalog, derived from the unified definition
/// catalog.
///
/// The catalog is lazily initialized on first access and cached for the
/// lifetime of the process.
pub fn get_block_catalog() -> &'static [BlockCatalogEntry] {
    static CATALOG: Lazy<Vec<BlockCatalogEntry>> = Lazy::new(build_catalog);
    &CATALOG
}

/// Returns the catalog organized by category.
pub fn get_block_catalog_by_category() -> &'static [BlockCatalogCategory] {
    static CATEGORIES: Lazy<Vec<BlockCatalogCategory>> = Lazy::new(|| {
        let catalog = get_block_catalog();
        let mut cat_map: indexmap::IndexMap<String, Vec<BlockCatalogEntry>> =
            indexmap::IndexMap::new();
        for e in catalog {
            cat_map
                .entry(e.category.clone())
                .or_default()
                .push(e.clone());
        }
        cat_map
            .into_iter()
            .map(|(name, entries)| BlockCatalogCategory { name, entries })
            .collect()
    });
    &CATEGORIES
}

/// Build the browser catalog from the unified definition registry.
///
/// One entry per unique block type.  When several definitions share a block
/// type (a rich/bridged definition plus a palette entry), the one carrying a
/// non-empty description wins so the browser shows the most informative
/// metadata; ports and category come from that same definition.
fn build_catalog() -> Vec<BlockCatalogEntry> {
    let mut chosen: indexmap::IndexMap<
        &'static str,
        &'static crate::simulink_libraries::types::SimulinkBlockDefinition,
    > = indexmap::IndexMap::new();

    for def in registry().all() {
        match chosen.get(def.block_type) {
            None => {
                chosen.insert(def.block_type, def);
            }
            Some(existing) => {
                if existing.description.is_empty() && !def.description.is_empty() {
                    chosen.insert(def.block_type, def);
                }
            }
        }
    }

    chosen
        .values()
        .map(|def| {
            let display_name = def.display_name().to_string();
            let description = if def.description.is_empty() {
                display_name.clone()
            } else {
                def.description.to_string()
            };
            BlockCatalogEntry {
                block_type: def.block_type.to_string(),
                display_name,
                category: def.category.to_string(),
                default_inputs: def.inputs.default_count(),
                default_outputs: def.outputs.default_count(),
                description,
            }
        })
        .collect()
}
