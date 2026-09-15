//! The resolved connection target and its de-duplication.

use super::metadata::merge_signal_aliases;
use crate::model::DashboardTargetPath;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::hash::Hash;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub enum ConnectionTargetOrigin {
    #[default]
    SourceBlock,
    SelfBlock,
    Internal,
    DashboardBinding,
    BusCreator,
    BusSelector,
    Mux,
    Demux,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ConnectionTargetResolve {
    Signal(String),
    Index(u32),
    TargetPath(DashboardTargetPath),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub struct ConnectionTarget {
    pub path: String,
    pub signal_name: Option<String>,
    /// Every signal name this target carries along the traced signal line,
    /// including names picked up crossing subsystem In/Outport boundaries and
    /// passing through Bus/Mux/Demux blocks. Order-preserving and deduplicated;
    /// `signal_name` (when set) is always included as the primary alias.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signal_names: Vec<String>,
    pub resolve: Option<ConnectionTargetResolve>,
    pub element_index: Option<u32>,
    pub origin: ConnectionTargetOrigin,
    pub signals_only: bool,
    pub testpoint: bool,
    /// The Simulink block type that produced this target (e.g. "Reference",
    /// "SubSystem", etc.). When the block type is "Reference", the matcher
    /// may use prefix matching instead of exact whole-path matching.
    pub block_type: Option<String>,
}

impl ConnectionTarget {
    pub fn new(path: String, origin: ConnectionTargetOrigin) -> Self {
        Self {
            path,
            origin,
            ..Self::default()
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn dedup_targets(targets: Vec<ConnectionTarget>) -> Vec<ConnectionTarget> {
    let mut seen: BTreeMap<
        (
            String,
            Option<String>,
            Option<ConnectionTargetResolve>,
            Option<u32>,
            ConnectionTargetOrigin,
            bool,
        ),
        usize,
    > = BTreeMap::new();
    let mut out: Vec<ConnectionTarget> = Vec::new();
    for target in targets {
        let key = (
            target.path.clone(),
            target.signal_name.clone(),
            target.resolve.clone(),
            target.element_index,
            target.origin,
            target.signals_only,
        );
        if let Some(index) = seen.get(&key).copied() {
            if let Some(existing) = out.get_mut(index) {
                existing.testpoint = existing.testpoint || target.testpoint;
                merge_signal_aliases(existing, &target.signal_names);
            }
        } else {
            seen.insert(key, out.len());
            out.push(target);
        }
    }
    out
}
