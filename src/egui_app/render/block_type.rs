//! Resolving a block to its catalog configuration.

#![cfg(feature = "egui")]

use std::cell::RefCell;
use std::sync::Arc;

use crate::block_types::{self, BlockTypeConfig};
use crate::model::Block;
use crate::simulink_libraries::block_memo::{BlockIdentity, BlockMemo};

fn normalize_library_block_path(path: &str) -> Option<String> {
    let path = path.trim();
    if path.is_empty() {
        return None;
    }

    // SLX XML stores long block/path names split across multiple lines.  Replace
    // the embedded newlines (and carriage-returns) with a space before any further
    // processing so that e.g. "matrix_library/Compare\nTo Constant" becomes
    // "matrix_library/Compare To Constant", which then matches the registry key.
    // Newlines act as word-wrap separators in the XML, not as characters to delete.
    let no_newlines;
    let path = if path.contains(['\n', '\r']) {
        no_newlines = path.replace(['\n', '\r'], " ");
        no_newlines.as_str()
    } else {
        path
    };

    let path = path.replace('\\', "/");
    let Some((lib, rest)) = path.split_once('/') else {
        return Some(path);
    };
    // Some models use `Something.slx/BlockName` while our registry keys are
    // typically stored without the `.slx` suffix.
    let lib_norm = lib
        .strip_suffix(".slx")
        .or_else(|| lib.strip_suffix(".SLX"))
        .unwrap_or(lib);
    Some(format!("{lib_norm}/{rest}"))
}

/// The rendering configuration of `block`.
///
/// Shared so that the several call sites per block and frame do not each clone
/// the configuration; the copy-on-write only happens for the few definitions
/// whose port placement depends on the block's own properties.
pub fn get_block_type_cfg(block: &Block) -> Arc<BlockTypeConfig> {
    let mut cfg = cached_block_type_cfg(block);
    // Port placement can depend on the block's own properties (a round Sum
    // wraps its last input onto the bottom edge, the rectangular one does not).
    let def = crate::simulink_libraries::resolve_definition(block);
    if let Some(f) = def.port_overrides_fn {
        let metadata = crate::simulink_libraries::metadata::extract_metadata(block, def);
        Arc::make_mut(&mut cfg).port_position_overrides = f(block, &metadata);
    }
    cfg
}

/// Cached [`lookup_block_type_cfg`]: the configuration depends only on the
/// block's identity fields and on the configuration map itself.
fn cached_block_type_cfg(block: &Block) -> Arc<BlockTypeConfig> {
    thread_local! {
        static MEMO: RefCell<BlockMemo<Arc<BlockTypeConfig>>> = RefCell::new(BlockMemo::default());
    }
    let identity = BlockIdentity::of(block);
    let generation = block_types::block_type_config_generation();
    MEMO.with(|memo| {
        memo.borrow_mut()
            .get_or_insert_with(&identity, generation, || {
                Arc::new(lookup_block_type_cfg(block))
            })
    })
}

fn lookup_block_type_cfg(block: &Block) -> BlockTypeConfig {
    let map = block_types::get_block_type_config_map();
    let Ok(g) = map.read() else {
        return BlockTypeConfig::default();
    };

    if block.is_matlab_function {
        return g.get("MATLAB Function").cloned().unwrap_or_default();
    }

    // Build library-specific candidates (library path / SourceBlock).  These are
    // kept separate from `block_type` so that virtual-library icons always take
    // priority over the generic block-kind icon (e.g. a "Product"-typed cross-
    // product block should show the cross-product SVG, not the generic "×").
    let mut lib_candidates: Vec<String> = Vec::new();

    if let Some(ref lib_path) = block.library_block_path {
        lib_candidates.push(lib_path.clone());
        if let Some(n) = normalize_library_block_path(lib_path)
            && n != *lib_path
        {
            lib_candidates.push(n);
        }
    }
    // Always check SourceBlock as well (not only when library_block_path is absent),
    // since library_block_path is derived from it and may carry the same casing issues.
    if let Some(source_block) = block.properties.get("SourceBlock") {
        if block.library_block_path.as_deref() != Some(source_block.as_str()) {
            lib_candidates.push(source_block.clone());
        }
        if let Some(n) = normalize_library_block_path(source_block)
            && !lib_candidates.contains(&n)
        {
            lib_candidates.push(n);
        }
    }

    // Collect all unique last-path-segments from the library candidates.
    let mut last_segments: Vec<String> = Vec::new();
    for c in &lib_candidates {
        if let Some((_, name)) = c.rsplit_once('/') {
            let s = name.to_string();
            if !last_segments.contains(&s) {
                last_segments.push(s);
            }
        }
    }

    // Phase 1 – exact match against full library paths and their last segments.
    // This intentionally runs BEFORE the block_type fallback so that virtual-
    // library icons win over the generic kind icon.
    for key in lib_candidates.iter().chain(last_segments.iter()) {
        if let Some(cfg) = g.get(key.as_str()) {
            return cfg.clone();
        }
    }

    // Phase 2 – whitespace-normalized and CamelCase-humanized fallback on last
    // segments.  Handles blocks where the SLX name differs from our registry
    // key by whitespace collapsing or CamelCase spacing (e.g.
    // "Create Diagonal\nMatrix" after newline removal → "CreateDiagonalMatrix"
    // → normalize(humanize(…)) → "create diagonal matrix" → match).
    //
    // Because `register_virtual_keys` pre-registers all normalized forms, these
    // are plain O(1) hash lookups – no linear scan needed.
    for seg in &last_segments {
        use crate::simulink_libraries::stubs::{humanize_camel_case, normalize_block_name};
        let seg_norm = normalize_block_name(seg);
        if let Some(cfg) = g.get(seg_norm.as_str()) {
            return cfg.clone();
        }
        let seg_human_norm = normalize_block_name(&humanize_camel_case(seg));
        if seg_human_norm != seg_norm
            && let Some(cfg) = g.get(seg_human_norm.as_str())
        {
            return cfg.clone();
        }
    }

    // Phase 3 – Simulink-semantic overrides that are expressed through block
    // properties rather than via a SourceBlock/library path.
    if let Some(key) = crate::simulink_libraries::traits::property_variant_key(block)
        && let Some(cfg) = g.get(key)
    {
        return cfg.clone();
    }

    // Phase 4 – generic block-type fallback (lowest priority).
    if let Some(cfg) = g.get(block.block_type.as_str()) {
        return cfg.clone();
    }

    BlockTypeConfig::default()
}
