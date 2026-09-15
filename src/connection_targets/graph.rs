//! Pure lookups over the model graph: lines, branches, ports and paths.

use super::resolver::ConnectionTargetResolver;
use super::target::{ConnectionTarget, dedup_targets};
use super::topology::same_line;
use crate::model::{
    Block, Branch, DashboardBinding, DashboardTargetPath, EndpointRef, Line, System,
};
use crate::simulink_libraries::traits::{SignalRole, block_traits};
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub(crate) fn build_block_lookup(system: &System) -> HashMap<&str, &Block> {
    system
        .blocks
        .iter()
        .filter_map(|block| block.sid.as_deref().map(|sid| (sid, block)))
        .collect()
}

pub(crate) fn child_system_path(system_path: &[String], block_name: &str) -> Vec<String> {
    let mut path = system_path.to_vec();
    path.push(block_name.to_string());
    path
}

pub(crate) fn boundary_port_index(block: &Block) -> u32 {
    block
        .properties
        .get("Port")
        .or_else(|| block.properties.get("PortNumber"))
        .and_then(|value| value.trim().parse::<u32>().ok())
        .unwrap_or(1)
}

/// Collect targets from all incoming lines of a block (any input port).
pub(crate) fn incoming_line_targets_for_block(
    system: &System,
    block: &Block,
    line_targets: &[Vec<ConnectionTarget>],
) -> Vec<ConnectionTarget> {
    let mut targets = Vec::new();
    for incoming in incoming_lines_for_block(system, block) {
        if let Some(line_index) = system
            .lines
            .iter()
            .position(|candidate| same_line(candidate, incoming))
        {
            targets.extend(line_targets[line_index].clone());
        }
    }
    targets
}

/// Collect targets from incoming lines ending at a specific input port of a
/// block.
pub(crate) fn incoming_line_targets_for_block_on_port(
    system: &System,
    block: &Block,
    port_index: u32,
    line_targets: &[Vec<ConnectionTarget>],
) -> Vec<ConnectionTarget> {
    let Some(block_sid) = block.sid.as_deref() else {
        return Vec::new();
    };
    let mut targets = Vec::new();
    for (line_index, line) in system.lines.iter().enumerate() {
        if line_data_input_ports(line, block_sid).contains(&port_index) {
            targets.extend(line_targets[line_index].clone());
        }
    }
    targets
}

/// Collect targets from outgoing lines originating from a specific output
/// port of a block.
pub(crate) fn outgoing_line_targets_for_block_on_port(
    system: &System,
    block: &Block,
    port_index: u32,
    line_targets: &[Vec<ConnectionTarget>],
) -> Vec<ConnectionTarget> {
    let mut targets = Vec::new();
    for (line_index, line) in outgoing_line_indices_for_block(system, block) {
        if line.src.as_ref().map(|src| src.port_index) == Some(port_index) {
            targets.extend(line_targets[line_index].clone());
        }
    }
    targets
}

/// Collect targets from lines connecting to a subsystem block's control
/// ports (enable/trigger/reset).  These are the targets that an
/// EnablePort/TriggerPort/ResetPort with `ShowOutputPort=on` should
/// propagate.
pub(crate) fn control_incoming_targets(
    system: &System,
    block: &Block,
    line_targets: &[Vec<ConnectionTarget>],
) -> Vec<ConnectionTarget> {
    let Some(block_sid) = block.sid.as_deref() else {
        return Vec::new();
    };
    let mut targets = Vec::new();
    for (line_index, line) in system.lines.iter().enumerate() {
        // Check if this line ends at the block with a control port type.
        let mut hits_control = false;
        for dst in line_destination_endpoints(line) {
            if dst.sid == block_sid && is_control_port_type(&dst.port_type) {
                hits_control = true;
                break;
            }
        }
        if hits_control {
            targets.extend(line_targets[line_index].clone());
        }
    }
    dedup_targets(targets)
}

/// The data input ports of `block_sid` that `line` ends at, counting every
/// branch: a branched signal reaches a port through `line.branches`, where the
/// line's own `dst` says nothing about which port that is.  Control endpoints
/// (`enable`, `trigger`, …) are skipped — they belong to the matching control
/// port block, not to the numbered `Inport`s.
pub(crate) fn line_data_input_ports(line: &Line, block_sid: &str) -> BTreeSet<u32> {
    fn collect(dst: Option<&EndpointRef>, block_sid: &str, ports: &mut BTreeSet<u32>) {
        if let Some(dst) = dst
            && dst.sid == block_sid
            && !is_control_port_type(&dst.port_type)
        {
            ports.insert(dst.port_index);
        }
    }

    fn collect_branches(branches: &[Branch], block_sid: &str, ports: &mut BTreeSet<u32>) {
        for branch in branches {
            collect(branch.dst.as_ref(), block_sid, ports);
            collect_branches(&branch.branches, block_sid, ports);
        }
    }

    let mut ports = BTreeSet::new();
    collect(line.dst.as_ref(), block_sid, &mut ports);
    collect_branches(&line.branches, block_sid, &mut ports);
    ports
}

/// The input ports of `block` that `line` ends at, falling back to port 1 when
/// the wiring does not say (a block without a SID).
pub(crate) fn input_port_indices(block: &Block, line: &Line) -> Vec<u32> {
    let ports = block
        .sid
        .as_deref()
        .map(|sid| line_data_input_ports(line, sid))
        .unwrap_or_default();
    if ports.is_empty() {
        vec![1]
    } else {
        ports.into_iter().collect()
    }
}

/// Every endpoint a line ends at: its own `dst` plus the destination of every
/// branch, because a branched line has no `dst` of its own.
pub(crate) fn line_destination_endpoints(line: &Line) -> Vec<&EndpointRef> {
    fn collect<'a>(branches: &'a [Branch], out: &mut Vec<&'a EndpointRef>) {
        for branch in branches {
            out.extend(branch.dst.as_ref());
            collect(&branch.branches, out);
        }
    }

    let mut endpoints: Vec<&EndpointRef> = line.dst.as_ref().into_iter().collect();
    collect(&line.branches, &mut endpoints);
    endpoints
}

pub(crate) fn is_control_port_type(port_type: &str) -> bool {
    matches!(
        port_type.to_ascii_lowercase().as_str(),
        "enable" | "trigger" | "ifaction" | "action" | "reset" | "state" | "event"
    )
}

pub(crate) fn incoming_targets_by_port(
    system: &System,
    block: &Block,
    line_targets: &[Vec<ConnectionTarget>],
) -> BTreeMap<u32, Vec<ConnectionTarget>> {
    let mut by_port = BTreeMap::new();
    let Some(block_sid) = block.sid.as_deref() else {
        return by_port;
    };

    for (line, targets) in system.lines.iter().zip(line_targets.iter()) {
        for port_index in line_data_input_ports(line, block_sid) {
            by_port
                .entry(port_index)
                .or_insert_with(Vec::new)
                .extend(targets.clone());
        }
    }

    for targets in by_port.values_mut() {
        *targets = dedup_targets(std::mem::take(targets));
    }

    by_port
}

pub(crate) fn child_outgoing_targets_by_port(
    resolver: &ConnectionTargetResolver,
    system: &System,
    system_path: &[String],
    line_targets: &[Vec<ConnectionTarget>],
) -> BTreeMap<u32, Vec<ConnectionTarget>> {
    let mut by_port = BTreeMap::new();
    let inport_boundary_paths =
        subsystem_boundary_paths(resolver, system, system_path, SignalRole::BoundaryInput);
    for block in &system.blocks {
        if block_traits(&block.block_type).signal_role != SignalRole::BoundaryOutput {
            continue;
        }

        let port_index = boundary_port_index(block);
        let mut targets = Vec::new();
        for incoming in incoming_lines_for_block(system, block) {
            if let Some(line_index) = system
                .lines
                .iter()
                .position(|candidate| same_line(candidate, incoming))
            {
                targets.extend(line_targets[line_index].clone());
            }
        }
        targets.retain(|target| !inport_boundary_paths.contains(&target.path));
        if !targets.is_empty() {
            by_port.insert(port_index, dedup_targets(targets));
        }
    }
    by_port
}

pub(crate) fn child_incoming_targets_by_port(
    system: &System,
    line_targets: &[Vec<ConnectionTarget>],
) -> BTreeMap<u32, Vec<ConnectionTarget>> {
    let mut by_port = BTreeMap::new();
    for block in &system.blocks {
        if block_traits(&block.block_type).signal_role != SignalRole::BoundaryInput {
            continue;
        }

        let port_index = boundary_port_index(block);
        let mut targets = Vec::new();
        for (line_index, _) in outgoing_line_indices_for_block(system, block) {
            targets.extend(line_targets[line_index].clone());
        }
        if !targets.is_empty() {
            by_port.insert(port_index, dedup_targets(targets));
        }
    }
    by_port
}

pub(crate) fn normalized_path_segment(segment: &str) -> Option<String> {
    let normalized = segment.split_whitespace().collect::<Vec<_>>().join(" ");
    (!normalized.is_empty()).then_some(normalized)
}

fn normalize_path(path: &str) -> String {
    path.trim_matches('/')
        .split('/')
        .filter_map(normalized_path_segment)
        .collect::<Vec<_>>()
        .join("/")
}

pub(crate) fn incoming_lines_for_block<'a>(system: &'a System, block: &Block) -> Vec<&'a Line> {
    let Some(block_sid) = block.sid.as_deref() else {
        return Vec::new();
    };
    system
        .lines
        .iter()
        .filter(|line| line_targets_block_sid(line, block_sid))
        .collect()
}

pub(crate) fn outgoing_line_indices_for_block<'a>(
    system: &'a System,
    block: &Block,
) -> Vec<(usize, &'a Line)> {
    let Some(block_sid) = block.sid.as_deref() else {
        return Vec::new();
    };

    system
        .lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.src.as_ref().is_some_and(|src| src.sid == block_sid))
        .collect()
}

pub(crate) fn outgoing_targets_by_port(
    system: &System,
    block: &Block,
    line_targets: &[Vec<ConnectionTarget>],
) -> BTreeMap<u32, Vec<ConnectionTarget>> {
    let mut by_port = BTreeMap::new();
    for (line_index, line) in outgoing_line_indices_for_block(system, block) {
        let port_index = line.src.as_ref().map(|src| src.port_index).unwrap_or(1);
        by_port
            .entry(port_index)
            .or_insert_with(Vec::new)
            .extend(line_targets[line_index].clone());
    }

    for targets in by_port.values_mut() {
        *targets = dedup_targets(std::mem::take(targets));
    }

    by_port
}

fn line_targets_block_sid(line: &Line, block_sid: &str) -> bool {
    line.dst.as_ref().is_some_and(|dst| dst.sid == block_sid)
        || branch_targets_block_sid(&line.branches, block_sid)
}

fn branch_targets_block_sid(branches: &[Branch], block_sid: &str) -> bool {
    branches.iter().any(|branch| {
        branch.dst.as_ref().is_some_and(|dst| dst.sid == block_sid)
            || branch_targets_block_sid(&branch.branches, block_sid)
    })
}

pub(crate) fn output_port_count(block: &Block) -> u32 {
    block
        .port_counts
        .as_ref()
        .and_then(|counts| counts.outs)
        .unwrap_or_else(|| {
            block
                .ports
                .iter()
                .filter(|port| port.port_type == "out")
                .count() as u32
        })
}

fn subsystem_boundary_paths(
    resolver: &ConnectionTargetResolver,
    system: &System,
    system_path: &[String],
    boundary: SignalRole,
) -> BTreeSet<String> {
    system
        .blocks
        .iter()
        .filter(|block| block_traits(&block.block_type).signal_role == boundary)
        .map(|block| resolver.full_block_path(system_path, &block.name))
        .collect()
}

pub(crate) fn qualify_external_path(model_name: &str, raw_path: &str) -> String {
    let clean = normalize_path(raw_path);
    let normalized_model = normalize_path(model_name);
    if clean.is_empty()
        || normalized_model.is_empty()
        || clean.starts_with(&format!("{normalized_model}/"))
        || clean == normalized_model
    {
        clean
    } else {
        format!("{normalized_model}/{clean}")
    }
}

pub(crate) fn dashboard_binding_block_path(binding: &DashboardBinding) -> &str {
    match binding {
        DashboardBinding::ParamSource { block_path, .. } => block_path,
        DashboardBinding::SignalSpec { block_path, .. } => block_path,
    }
}

pub(crate) fn dashboard_binding_target_path(binding: &DashboardBinding) -> &DashboardTargetPath {
    match binding {
        DashboardBinding::ParamSource { target_path, .. } => target_path,
        DashboardBinding::SignalSpec { target_path, .. } => target_path,
    }
}
