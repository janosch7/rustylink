//! Signal names, aliases, test points and resolve hints carried on a target.

use super::graph::{line_destination_endpoints, normalized_path_segment};
use super::target::{ConnectionTarget, ConnectionTargetResolve, dedup_targets};
use crate::model::{Block, Line, System};
use crate::simulink_libraries::traits::{SignalRole, block_traits};
use std::collections::HashMap;

pub(crate) fn apply_local_line_metadata(line: &Line, targets: &mut [ConnectionTarget]) {
    let explicit_name = explicit_line_signal_name(line);
    let explicit_testpoint = line_testpoint(line);
    for target in targets {
        set_signal_name_only(target, explicit_name.clone().or(target.signal_name.clone()));
        if target.resolve.is_none() {
            set_signal_resolve(target, explicit_name.clone());
        }
        target.testpoint = target.testpoint || explicit_testpoint;
    }
}

pub(crate) fn apply_source_port_testpoint(
    block: &Block,
    line: &Line,
    targets: &mut [ConnectionTarget],
) {
    let Some(src) = &line.src else {
        return;
    };
    if !port_testpoint(block, src.port_type.as_str(), src.port_index) {
        return;
    }
    for target in targets {
        target.testpoint = true;
    }
}

pub(crate) fn merge_upstream_metadata(
    line: &Line,
    current_targets: &[ConnectionTarget],
    propagated_targets: &[ConnectionTarget],
    allow_cross_path: bool,
) -> Vec<ConnectionTarget> {
    let explicit_name = explicit_line_signal_name(line);
    let explicit_testpoint = line_testpoint(line);
    let path_counts = path_match_counts(current_targets);
    let mut merged_targets = current_targets.to_vec();

    for target in &mut merged_targets {
        // Split propagated targets into same-path and cross-path matches.
        // signal_names/signal_name should only flow along the same signal
        // path — a bus is just pack/unpack of signal lines, they should
        // NOT share their names with each other. testpoint, however, should
        // propagate across subsystem boundaries regardless of path.
        let same_path_count = path_counts.get(target.path.as_str()).copied().unwrap_or(0);
        let mut same_path_propagated: Vec<&ConnectionTarget> = Vec::new();
        let mut all_propagated: Vec<&ConnectionTarget> = Vec::new();

        for candidate in propagated_targets {
            if metadata_paths_match(target, candidate, same_path_count, allow_cross_path) {
                all_propagated.push(candidate);
                // Only treat as same-path for signal_names purposes when
                // the paths actually match (not a cross-path match).
                if target.path == candidate.path {
                    same_path_propagated.push(candidate);
                }
            }
        }

        if all_propagated.is_empty() {
            continue;
        }

        // signal_name and signal_names: only from same-path matches.
        if !same_path_propagated.is_empty() {
            let propagated_name = same_path_propagated
                .iter()
                .find_map(|candidate| candidate.signal_name.clone());
            set_signal_name_only(
                target,
                explicit_name
                    .clone()
                    .or(propagated_name)
                    .or(target.signal_name.clone()),
            );
            for candidate in &same_path_propagated {
                merge_signal_aliases(target, &candidate.signal_names);
            }
        } else if all_propagated.len() == 1 {
            // No same-path match, but exactly one cross-path match — this
            // is a single signal crossing a subsystem boundary, not a bus.
            // Safe to merge signal_names.
            let propagated_name = all_propagated
                .iter()
                .find_map(|candidate| candidate.signal_name.clone());
            set_signal_name_only(
                target,
                explicit_name
                    .clone()
                    .or(propagated_name)
                    .or(target.signal_name.clone()),
            );
            for candidate in &all_propagated {
                merge_signal_aliases(target, &candidate.signal_names);
            }
        } else if let Some(ref explicit) = explicit_name {
            // No same-path match, but the line itself has an explicit name.
            set_signal_name_only(target, Some(explicit.clone()));
        }

        // testpoint: from all matches (same-path + cross-path).
        target.testpoint = explicit_testpoint
            || target.testpoint
            || all_propagated.iter().any(|candidate| candidate.testpoint);
    }

    dedup_targets(merged_targets)
}

fn path_match_counts(targets: &[ConnectionTarget]) -> HashMap<&str, usize> {
    let mut counts = HashMap::new();
    for target in targets {
        *counts.entry(target.path.as_str()).or_insert(0) += 1;
    }
    counts
}

fn metadata_paths_match(
    current: &ConnectionTarget,
    propagated: &ConnectionTarget,
    same_path_count: usize,
    allow_cross_path: bool,
) -> bool {
    match (
        resolve_index_value(&current.resolve),
        resolve_index_value(&propagated.resolve),
    ) {
        (Some(current_index), Some(propagated_index)) => {
            return current_index == propagated_index;
        }
        (Some(_), None) | (None, Some(_)) if !allow_cross_path => {}
        _ => {}
    }

    if current.path != propagated.path {
        if allow_cross_path {
            return true;
        }
        return false;
    }

    match (current.element_index, propagated.element_index) {
        (_, None) => true,
        (Some(current_index), Some(propagated_index)) => current_index == propagated_index,
        (None, Some(_)) => same_path_count <= 1,
    }
}

pub(crate) fn set_signal_name_only(target: &mut ConnectionTarget, signal_name: Option<String>) {
    let normalized = signal_name.and_then(|signal_name| normalized_path_segment(&signal_name));
    if let Some(name) = &normalized {
        push_signal_alias(target, name);
    }
    target.signal_name = normalized;
}

/// Record `name` as one of the signal names this target carries, keeping
/// insertion order and skipping duplicates. `name` is expected to already be a
/// normalized path segment.
fn push_signal_alias(target: &mut ConnectionTarget, name: &str) {
    if !target.signal_names.iter().any(|existing| existing == name) {
        target.signal_names.push(name.to_string());
    }
}

/// Union `aliases` into `target.signal_names`, preserving order and dropping
/// duplicates.
pub(crate) fn merge_signal_aliases(target: &mut ConnectionTarget, aliases: &[String]) {
    for alias in aliases {
        push_signal_alias(target, alias);
    }
}

pub(crate) fn set_signal_resolve(target: &mut ConnectionTarget, signal_name: Option<String>) {
    target.resolve = signal_name
        .and_then(|signal_name| normalize_resolve_signal(&signal_name))
        .map(ConnectionTargetResolve::Signal);
}

pub(crate) fn normalize_resolve_signal(signal_name: &str) -> Option<String> {
    let trimmed = signal_name
        .trim()
        .trim_start_matches('<')
        .trim_end_matches('>');
    normalized_path_segment(trimmed)
}

pub(crate) fn resolve_signal_value(resolve: &Option<ConnectionTargetResolve>) -> Option<&str> {
    match resolve {
        Some(ConnectionTargetResolve::Signal(signal_name)) => Some(signal_name.as_str()),
        _ => None,
    }
}

fn resolve_index_value(resolve: &Option<ConnectionTargetResolve>) -> Option<u32> {
    match resolve {
        Some(ConnectionTargetResolve::Index(index)) => Some(*index),
        Some(ConnectionTargetResolve::TargetPath(target_path)) => target_path.port_index,
        _ => None,
    }
}

pub(crate) fn matches_resolve_signal(target: &ConnectionTarget, selected_name: &str) -> bool {
    resolve_signal_value(&target.resolve)
        .is_some_and(|signal_name| signal_keys_match(signal_name, selected_name))
}

pub(crate) fn signal_keys_match(left: &str, right: &str) -> bool {
    let Some(left) = normalize_resolve_signal(left) else {
        return false;
    };
    let Some(right) = normalize_resolve_signal(right) else {
        return false;
    };
    left.eq_ignore_ascii_case(&right)
}

/// Returns the hierarchical signal path for a BusSelector output port,
/// parsed from the block's `OutputSignals` property.  Returns `None` when
/// the property is absent (caller falls back to flat matching).
pub(crate) fn bus_selector_output_path(block: &Block, port_index: u32) -> Option<String> {
    let output_signals = block.properties.get("OutputSignals")?;
    let paths: Vec<&str> = output_signals
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    let idx = (port_index as usize).checked_sub(1)?;
    let path = paths.get(idx)?;
    normalize_resolve_signal(path)
}

/// Checks whether a target's resolve path matches the given hierarchical
/// path.  Matches exactly for leaf signals, or as a prefix for sub-bus
/// selection (e.g. path `bus_c.bus_a` matches resolve `bus_c.bus_a.a`).
pub(crate) fn bus_signal_path_matches(target: &ConnectionTarget, path: &str) -> bool {
    let Some(target_path) = resolve_signal_value(&target.resolve) else {
        return false;
    };
    let Some(t) = normalize_resolve_signal(target_path) else {
        return false;
    };
    let Some(p) = normalize_resolve_signal(path) else {
        return false;
    };
    // Exact match (leaf signal)
    if t.eq_ignore_ascii_case(&p) {
        return true;
    }
    // Prefix match (sub-bus selection: path is a parent of target)
    let t_lower = t.to_ascii_lowercase();
    let p_lower = p.to_ascii_lowercase();
    t_lower.starts_with(&format!("{p_lower}."))
}

pub(crate) fn apply_line_resolve_hint(
    line: &Line,
    block_lookup: &HashMap<&str, &Block>,
    target: &mut ConnectionTarget,
) {
    if let Some(signal_name) = explicit_line_signal_name(line) {
        set_signal_resolve(target, Some(signal_name));
        return;
    }

    // The mux input this line ends at – for a branched line that is one of the
    // branch endpoints, not `line.dst`.
    if let Some(dst) = line_destination_endpoints(line).into_iter().find(|dst| {
        block_lookup
            .get(dst.sid.as_str())
            .is_some_and(|block| block_traits(&block.block_type).signal_role == SignalRole::Mux)
    }) {
        target.resolve = Some(ConnectionTargetResolve::Index(dst.port_index));
        return;
    }

    if target.resolve.is_none() && target.element_index.is_some() {
        target.resolve = target.element_index.map(ConnectionTargetResolve::Index);
    }
}

pub(crate) fn port_signal_name(block: &Block, port_type: &str, port_index: u32) -> Option<String> {
    block
        .ports
        .iter()
        .find(|port| port.port_type == port_type && port.index.unwrap_or(0) == port_index)
        .and_then(|port| {
            port.properties
                .get("Name")
                .or_else(|| port.properties.get("name"))
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .and_then(normalized_path_segment)
        })
}

pub(crate) fn port_testpoint(block: &Block, port_type: &str, port_index: u32) -> bool {
    block
        .ports
        .iter()
        .find(|port| port.port_type == port_type && port.index.unwrap_or(0) == port_index)
        .and_then(|port| port.properties.get("TestPoint"))
        .is_some_and(|value| matches!(value.trim(), "on" | "true" | "1" | "On" | "True"))
}

pub(crate) fn line_testpoint(line: &Line) -> bool {
    line.properties
        .get("TestPoint")
        .is_some_and(|value| matches!(value.trim(), "on" | "true" | "1" | "On" | "True"))
}

pub(crate) fn explicit_line_signal_name(line: &Line) -> Option<String> {
    line.name
        .as_ref()
        .map(|name| name.trim())
        .filter(|name| !name.is_empty())
        .and_then(normalized_path_segment)
}

pub(crate) fn routing_line_signal_name(_system: &System, line: &Line) -> Option<String> {
    explicit_line_signal_name(line)
}
