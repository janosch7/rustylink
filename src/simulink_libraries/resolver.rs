//! Resolution of a parsed [`Block`] to its [`SimulinkBlockDefinition`].
//!
//! The registry is built once (lazily) from every catalog library plus the
//! bridged built-in virtual libraries.  All resolution keys are pre-computed
//! and inserted into a `HashMap`, so matching a block to its definition is an
//! O(1) lookup with a few cheap normalisations as fallbacks.

#![cfg(feature = "egui")]

use std::collections::HashMap;
use std::sync::RwLock;

use once_cell::sync::OnceCell;

use crate::model::Block;

use super::types::{DefinitionRegistry, SimulinkBlockDefinition, unknown_block_definition};

/// Normalise a block/library name: collapse whitespace, lowercase.
pub fn normalize_name(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Insert all useful key variants for a definition into the map.
fn register_keys(
    map: &mut HashMap<String, &'static SimulinkBlockDefinition>,
    def: &'static SimulinkBlockDefinition,
) {
    let mut names: Vec<&str> = Vec::with_capacity(1 + def.aliases.len());
    names.push(def.block_type);
    names.extend_from_slice(def.aliases);

    for n in names {
        use crate::builtin_libraries::virtual_library::humanize_camel_case;
        let human = humanize_camel_case(n);
        for key in [n.to_string(), normalize_name(n), normalize_name(&human)] {
            map.entry(key).or_insert(def);
        }
    }
}

impl DefinitionRegistry {
    fn build() -> Self {
        let mut by_key: HashMap<String, &'static SimulinkBlockDefinition> = HashMap::new();
        let mut all: Vec<&'static SimulinkBlockDefinition> = Vec::new();

        for lib in super::libraries::ALL_LIBRARIES {
            for def in lib.blocks {
                register_keys(&mut by_key, def);
                all.push(def);
            }
        }

        // Bridge the built-in virtual libraries (matrix, discrete, …) for their
        // rendering metadata.  Registered after the hand-written libraries so
        // core definitions win on key collisions.
        for def in super::bridge::bridged_definitions() {
            register_keys(&mut by_key, def);
            all.push(def);
        }

        // Bridge user-registered libraries.
        if let Some(extra) = USER_DEFINITIONS.get()
            && let Ok(guard) = extra.read()
        {
            for def in guard.iter() {
                register_keys(&mut by_key, def);
                all.push(def);
            }
        }

        Self { by_key, all }
    }

    /// Look up a definition by an exact or normalised key.
    pub fn lookup(&self, key: &str) -> Option<&'static SimulinkBlockDefinition> {
        self.by_key
            .get(key)
            .copied()
            .or_else(|| self.by_key.get(&normalize_name(key)).copied())
    }

    /// All known definitions (deduplicated by insertion).
    pub fn all(&self) -> &[&'static SimulinkBlockDefinition] {
        &self.all
    }
}

static REGISTRY: OnceCell<DefinitionRegistry> = OnceCell::new();
static USER_DEFINITIONS: OnceCell<RwLock<Vec<&'static SimulinkBlockDefinition>>> = OnceCell::new();

/// Register an additional, user-supplied block definition.
///
/// Must be called before the registry is first built (i.e. before any
/// rendering).  Definitions have `'static` lifetime so they can be embedded in
/// the global registry without allocation churn.
pub fn register_user_definition(def: &'static SimulinkBlockDefinition) {
    let store = USER_DEFINITIONS.get_or_init(|| RwLock::new(Vec::new()));
    if let Ok(mut guard) = store.write() {
        guard.push(def);
    }
}

/// Get the global definition registry.
pub fn registry() -> &'static DefinitionRegistry {
    REGISTRY.get_or_init(DefinitionRegistry::build)
}

/// Strip an `.slx`/`.SLX` suffix from the first path segment and convert
/// backslashes to slashes; collapse embedded newlines (SLX word-wrap) to spaces.
fn normalize_library_path(path: &str) -> String {
    let path = path.replace(['\n', '\r'], " ");
    let path = path.replace('\\', "/");
    if let Some((lib, rest)) = path.split_once('/') {
        let lib = lib
            .strip_suffix(".slx")
            .or_else(|| lib.strip_suffix(".SLX"))
            .unwrap_or(lib);
        format!("{lib}/{rest}")
    } else {
        path
    }
}

/// Resolve a parsed block to its definition.
///
/// Resolution order (highest priority first):
/// 1. MATLAB Function blocks.
/// 2. Library path / `SourceBlock` full path and its last segment.
/// 3. Property-driven semantic overrides (e.g. matrix-multiply Product).
/// 4. Generic `block_type`.
/// 5. The fallback [`unknown_block_definition`].
pub fn resolve_definition(block: &Block) -> &'static SimulinkBlockDefinition {
    let reg = registry();

    if block.is_matlab_function
        && let Some(def) = reg.lookup("MATLAB Function")
    {
        return def;
    }

    // Library-specific candidates.
    let mut candidates: Vec<String> = Vec::new();
    let push = |s: String, v: &mut Vec<String>| {
        if !s.is_empty() && !v.contains(&s) {
            v.push(s);
        }
    };
    if let Some(lib_path) = &block.library_block_path {
        push(lib_path.clone(), &mut candidates);
        push(normalize_library_path(lib_path), &mut candidates);
    }
    if let Some(source) = block.properties.get("SourceBlock") {
        push(source.clone(), &mut candidates);
        push(normalize_library_path(source), &mut candidates);
    }
    // Last path segments.
    let mut segments: Vec<String> = Vec::new();
    for c in &candidates {
        if let Some((_, last)) = c.rsplit_once('/') {
            push(last.to_string(), &mut segments);
        }
    }
    for key in candidates.iter().chain(segments.iter()) {
        if let Some(def) = reg.lookup(key) {
            return def;
        }
    }

    // Property-driven semantic override: a Product configured for matrix
    // multiplication should resolve to the matrix-multiply definition.
    if block.block_type == "Product"
        && block.properties.get("Multiplication").map(|v| v.trim()) == Some("Matrix(*)")
        && let Some(def) = reg.lookup("matrix multiply")
    {
        return def;
    }

    reg.lookup(&block.block_type)
        .unwrap_or_else(|| unknown_block_definition())
}
