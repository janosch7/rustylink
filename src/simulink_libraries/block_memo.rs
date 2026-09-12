//! Memoisation of the per-block catalog lookups.
//!
//! Resolving a block to its definition or to its rendering configuration walks
//! a handful of string candidates and hashes each of them.  The result only
//! depends on a few fields of the block, all of which are stable while the
//! model is not edited, so the canvas would otherwise redo the same work for
//! every block on every frame.  [`BlockMemo`] caches the result under those
//! fields and re-verifies them on each hit, so a hash collision recomputes
//! instead of returning a foreign entry.

#![cfg(feature = "egui")]

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

use crate::model::Block;

use super::traits::property_variant_key;

/// The block fields every catalog lookup is a function of.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BlockIdentity<'a> {
    matlab_function: bool,
    library_path: Option<&'a str>,
    source_block: Option<&'a str>,
    block_type: &'a str,
    variant: Option<&'static str>,
}

impl<'a> BlockIdentity<'a> {
    pub fn of(block: &'a Block) -> Self {
        Self {
            matlab_function: block.is_matlab_function,
            library_path: block.library_block_path.as_deref(),
            source_block: block.properties.get("SourceBlock").map(String::as_str),
            block_type: block.block_type.as_str(),
            variant: property_variant_key(block),
        }
    }

    fn key(&self) -> u64 {
        let mut hasher = Fnv1a::default();
        hasher.write_u8(self.matlab_function as u8);
        for part in [self.library_path, self.source_block, self.variant] {
            hasher.write(part.unwrap_or_default().as_bytes());
            hasher.write_u8(0xff);
        }
        hasher.write(self.block_type.as_bytes());
        hasher.finish()
    }

    fn to_owned_identity(self) -> OwnedIdentity {
        OwnedIdentity {
            matlab_function: self.matlab_function,
            library_path: self.library_path.map(str::to_string),
            source_block: self.source_block.map(str::to_string),
            block_type: self.block_type.to_string(),
            variant: self.variant,
        }
    }
}

struct OwnedIdentity {
    matlab_function: bool,
    library_path: Option<String>,
    source_block: Option<String>,
    block_type: String,
    variant: Option<&'static str>,
}

impl OwnedIdentity {
    fn matches(&self, other: &BlockIdentity<'_>) -> bool {
        self.matlab_function == other.matlab_function
            && self.library_path.as_deref() == other.library_path
            && self.source_block.as_deref() == other.source_block
            && self.block_type == other.block_type
            && self.variant == other.variant
    }
}

/// A cache of values derived from a [`BlockIdentity`].
///
/// `generation` lets a caller whose backing registry can change at runtime
/// drop the whole cache when that registry is written to.
pub struct BlockMemo<V> {
    entries: HashMap<u64, (OwnedIdentity, V), BuildHasherDefault<PassThrough>>,
    generation: u64,
}

/// FNV-1a over the identity's bytes: the keys are a handful of short strings,
/// for which SipHash's setup dominates.
struct Fnv1a(u64);

impl Default for Fnv1a {
    fn default() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
}

impl Hasher for Fnv1a {
    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 ^= *b as u64;
            self.0 = self.0.wrapping_mul(0x1000_0000_01b3);
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

/// Hasher for a map whose keys are already well-distributed hashes.
#[derive(Default)]
pub struct PassThrough(u64);

impl Hasher for PassThrough {
    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 = (self.0 << 8) | *b as u64;
        }
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = value;
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

impl<V> Default for BlockMemo<V> {
    fn default() -> Self {
        Self {
            entries: HashMap::default(),
            generation: 0,
        }
    }
}

impl<V: Clone> BlockMemo<V> {
    /// The cached value for `identity`, computing and storing it on a miss or
    /// when `generation` differs from the one the cache was filled under.
    pub fn get_or_insert_with(
        &mut self,
        identity: &BlockIdentity<'_>,
        generation: u64,
        compute: impl FnOnce() -> V,
    ) -> V {
        if self.generation != generation {
            self.entries.clear();
            self.generation = generation;
        }
        let key = identity.key();
        if let Some((owned, value)) = self.entries.get(&key)
            && owned.matches(identity)
        {
            return value.clone();
        }
        let value = compute();
        self.entries
            .insert(key, (identity.to_owned_identity(), value.clone()));
        value
    }
}
