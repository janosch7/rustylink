//! Cache keys and the geometry-independent topology signature.

use crate::model::{Block, Branch, EndpointRef, Line, System};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub(crate) fn block_cache_key(system_path: &[String], block: &Block) -> String {
    if let Some(sid) = &block.sid {
        return format!("sid:{sid}");
    }
    let mut key = system_path.join("/");
    if !key.is_empty() {
        key.push('/');
    }
    key.push_str(&block.name);
    key.push('#');
    key.push_str(&block.block_type);
    key
}

pub(crate) fn line_cache_key(system_path: &[String], line: &Line) -> String {
    let mut key = system_path.join("/");
    key.push('|');
    key.push_str(&line_identity(line));
    key
}

pub(crate) fn line_identity(line: &Line) -> String {
    let src = line
        .src
        .as_ref()
        .map(|src| format!("{}:{}:{}", src.sid, src.port_type, src.port_index))
        .unwrap_or_else(|| "none".to_string());
    let dst = line
        .dst
        .as_ref()
        .map(|dst| format!("{}:{}:{}", dst.sid, dst.port_type, dst.port_index))
        .unwrap_or_else(|| branch_identity(&line.branches));
    format!(
        "{src}->{dst}:{}:{}",
        line.name.as_deref().unwrap_or(""),
        line.points.len()
    )
}

fn branch_identity(branches: &[Branch]) -> String {
    let mut parts = Vec::new();
    collect_branch_identity(branches, &mut parts);
    parts.join(",")
}

fn collect_branch_identity(branches: &[Branch], parts: &mut Vec<String>) {
    for branch in branches {
        if let Some(dst) = &branch.dst {
            parts.push(format!("{}:{}:{}", dst.sid, dst.port_type, dst.port_index));
        }
        collect_branch_identity(&branch.branches, parts);
    }
}

pub(crate) fn same_line(left: &Line, right: &Line) -> bool {
    line_identity(left) == line_identity(right)
}

/// Property keys that only affect a block's or line's *geometry* (position,
/// stacking, routing waypoints) and therefore never change the resolved signal
/// / target-path graph.  They are skipped by [`model_topology_signature`].
const GEOMETRY_PROPERTY_KEYS: &[&str] = &["Position", "ZOrder", "Points", "SortIndex"];

/// A cheap 64-bit signature of everything in the model that the connection
/// target resolver depends on, deliberately *excluding* geometry (block
/// positions, z-order, line waypoints).
///
/// Building the resolver walks the whole subsystem tree and re-propagates every
/// signal, which is far more expensive than hashing.  Layout-only edits (moving
/// a block, dragging a line waypoint) leave this signature unchanged, so the
/// cached resolver can be reused instead of rebuilt every frame while dragging.
pub fn model_topology_signature(root: &System) -> u64 {
    let mut h = DefaultHasher::new();
    hash_system(root, &mut h);
    h.finish()
}

fn hash_system(sys: &System, h: &mut DefaultHasher) {
    if let Some(name) = sys.properties.get("Name") {
        name.hash(h);
    }
    sys.blocks.len().hash(h);
    for block in &sys.blocks {
        hash_block(block, h);
    }
    sys.lines.len().hash(h);
    for line in &sys.lines {
        hash_line(line, h);
    }
}

fn hash_non_geometry_properties(
    properties: &indexmap::IndexMap<String, String>,
    h: &mut DefaultHasher,
) {
    for (key, value) in properties {
        if GEOMETRY_PROPERTY_KEYS.contains(&key.as_str()) {
            continue;
        }
        key.hash(h);
        value.hash(h);
    }
}

fn hash_block(block: &Block, h: &mut DefaultHasher) {
    block.block_type.hash(h);
    block.name.hash(h);
    block.sid.hash(h);
    block.commented.hash(h);
    block.value.hash(h);
    if let Some(pc) = &block.port_counts {
        pc.ins.hash(h);
        pc.outs.hash(h);
    }
    block.ports.len().hash(h);
    for port in &block.ports {
        port.port_type.hash(h);
        port.index.hash(h);
    }
    hash_non_geometry_properties(&block.properties, h);
    block.dashboard_binding.is_some().hash(h);
    if let Some(subsystem) = &block.subsystem {
        hash_system(subsystem, h);
    }
}

fn hash_endpoint(endpoint: &Option<EndpointRef>, h: &mut DefaultHasher) {
    match endpoint {
        Some(e) => {
            1u8.hash(h);
            e.sid.hash(h);
            e.port_type.hash(h);
            e.port_index.hash(h);
        }
        None => 0u8.hash(h),
    }
}

fn hash_branch(branch: &Branch, h: &mut DefaultHasher) {
    branch.name.hash(h);
    hash_endpoint(&branch.dst, h);
    branch.branches.len().hash(h);
    for child in &branch.branches {
        hash_branch(child, h);
    }
}

fn hash_line(line: &Line, h: &mut DefaultHasher) {
    line.name.hash(h);
    hash_endpoint(&line.src, h);
    hash_endpoint(&line.dst, h);
    line.branches.len().hash(h);
    for branch in &line.branches {
        hash_branch(branch, h);
    }
    hash_non_geometry_properties(&line.properties, h);
}
