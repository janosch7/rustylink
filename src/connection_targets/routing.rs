//! Target resolution for the Signal Routing blocks (bus, mux, from).

use super::graph::{
    boundary_port_index, incoming_lines_for_block, input_port_indices, is_control_port_type,
    line_data_input_ports, outgoing_line_indices_for_block,
    outgoing_line_targets_for_block_on_port,
};
use super::metadata::{
    bus_selector_output_path, bus_signal_path_matches, explicit_line_signal_name,
    matches_resolve_signal, normalize_resolve_signal, port_signal_name, resolve_signal_value,
    set_signal_name_only, set_signal_resolve, signal_keys_match,
};
use super::resolver::{ChildSubsystemSummary, ConnectionTargetResolver, ParentSubsystemContext};
use super::target::{
    ConnectionTarget, ConnectionTargetOrigin, ConnectionTargetResolve, dedup_targets,
};
use super::topology::same_line;
use super::variants::active_variant_port_index;
use crate::model::{Block, EndpointRef, Line, System};
use crate::simulink_libraries::traits::{SignalRole, block_traits};
use std::collections::HashMap;

impl ConnectionTargetResolver {
    pub(crate) fn bus_creator_targets(
        &self,
        system: &System,
        _system_path: &[String],
        block: &Block,
        _line: &Line,
        line_targets: &[Vec<ConnectionTarget>],
    ) -> Vec<ConnectionTarget> {
        let mut targets = Vec::new();
        for incoming in incoming_lines_for_block(system, block) {
            let Some(line_index) = system
                .lines
                .iter()
                .position(|candidate| same_line(candidate, incoming))
            else {
                continue;
            };
            let signal_name = explicit_line_signal_name(incoming);
            for input_index in input_port_indices(block, incoming) {
                for mut target in line_targets[line_index].clone() {
                    let element_name = signal_name
                        .clone()
                        .or_else(|| target.signal_name.clone())
                        .or_else(|| Some(format!("signal{input_index}")));
                    let next_signal_name = element_name.clone();
                    // Only prepend to the resolve path when the target came
                    // from another bus block (nested bus).  Direct leaf inputs
                    // have their resolve set by `base_line_targets` to the
                    // line name, which is the same as `element_name` —
                    // prepending would double it.
                    let is_from_bus = matches!(
                        target.origin,
                        ConnectionTargetOrigin::BusCreator | ConnectionTargetOrigin::BusSelector
                    );
                    let next_resolve_signal = if is_from_bus {
                        if let Some(existing) =
                            resolve_signal_value(&target.resolve).map(str::to_string)
                        {
                            element_name.map(|en| format!("{en}.{existing}"))
                        } else {
                            element_name
                        }
                    } else {
                        element_name
                    };
                    set_signal_name_only(&mut target, next_signal_name);
                    set_signal_resolve(&mut target, next_resolve_signal);
                    target.origin = ConnectionTargetOrigin::BusCreator;
                    targets.push(target);
                }
            }
        }
        targets
    }

    pub(crate) fn bus_selector_targets(
        &self,
        system: &System,
        block: &Block,
        line: &Line,
        line_targets: &[Vec<ConnectionTarget>],
    ) -> Vec<ConnectionTarget> {
        // Hierarchical path from OutputSignals property (when available).
        let output_path = line
            .src
            .as_ref()
            .and_then(|src| bus_selector_output_path(block, src.port_index));

        // Flat selected name for backward-compat fallback.
        let selected_name = explicit_line_signal_name(line).or_else(|| {
            line.src
                .as_ref()
                .and_then(|src| port_signal_name(block, src.port_type.as_str(), src.port_index))
                .or_else(|| {
                    line.src
                        .as_ref()
                        .map(|src| format!("signal{}", src.port_index))
                })
        });

        let Some(incoming) = incoming_lines_for_block(system, block).into_iter().next() else {
            return Vec::new();
        };
        let Some(line_index) = system
            .lines
            .iter()
            .position(|candidate| same_line(candidate, incoming))
        else {
            return Vec::new();
        };

        line_targets[line_index]
            .iter()
            .filter(|target| {
                if let Some(ref path) = output_path {
                    // Hierarchical matching using OutputSignals.
                    bus_signal_path_matches(target, path)
                } else {
                    // Flat matching (backward compat, no OutputSignals).
                    let name = selected_name.as_deref().unwrap_or("");
                    matches_resolve_signal(target, name)
                        || target
                            .signal_name
                            .as_deref()
                            .is_some_and(|n| signal_keys_match(n, name))
                }
            })
            .cloned()
            .map(|mut target| {
                target.origin = ConnectionTargetOrigin::BusSelector;
                target
            })
            .collect()
    }

    pub(crate) fn bus_assignment_targets(
        &self,
        system: &System,
        block: &Block,
        _line: &Line,
        line_targets: &[Vec<ConnectionTarget>],
    ) -> Vec<ConnectionTarget> {
        // Parse AssignedSignals (comma-separated hierarchical paths).
        let assigned_signals: Vec<String> = block
            .properties
            .get("AssignedSignals")
            .map(|s| {
                s.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        // Targets from the main bus input (in:1).
        let main_targets = self.bus_assignment_input_targets(system, block, 1, line_targets);

        if assigned_signals.is_empty() {
            return main_targets;
        }

        let mut result = Vec::new();

        // Process replacement inputs (in:2, in:3, ...).
        for (i, assigned_path) in assigned_signals.iter().enumerate() {
            let replacement_port = (i + 2) as u32;
            let replacement_targets =
                self.bus_assignment_input_targets(system, block, replacement_port, line_targets);

            let normalized_assigned = normalize_resolve_signal(assigned_path);
            for mut target in replacement_targets {
                if let Some(existing) = resolve_signal_value(&target.resolve).map(str::to_string) {
                    // Sub-bus replacement: prepend assigned path to existing
                    // resolve so leaf identity is preserved.
                    if let Some(ref np) = normalized_assigned {
                        let new_resolve = format!("{np}.{existing}");
                        set_signal_resolve(&mut target, Some(new_resolve));
                    }
                } else {
                    // Leaf replacement: set resolve to the assigned path.
                    set_signal_resolve(&mut target, normalized_assigned.clone());
                }
                result.push(target);
            }
        }

        // Add pass-through targets, excluding those matching assigned signals.
        for target in main_targets {
            let is_assigned = assigned_signals
                .iter()
                .any(|assigned_path| bus_signal_path_matches(&target, assigned_path));
            if !is_assigned {
                result.push(target);
            }
        }

        result
    }

    /// Get the line targets for a specific input port of a block.
    fn bus_assignment_input_targets(
        &self,
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
        dedup_targets(targets)
    }

    pub(crate) fn mux_targets(
        &self,
        system: &System,
        block: &Block,
        line_targets: &[Vec<ConnectionTarget>],
    ) -> Vec<ConnectionTarget> {
        let mut targets = Vec::new();
        for incoming in incoming_lines_for_block(system, block) {
            let Some(line_index) = system
                .lines
                .iter()
                .position(|candidate| same_line(candidate, incoming))
            else {
                continue;
            };
            let signal_name = explicit_line_signal_name(incoming);
            for input_index in input_port_indices(block, incoming) {
                for mut target in line_targets[line_index].clone() {
                    target.resolve = Some(ConnectionTargetResolve::Index(input_index));
                    let next_signal_name = signal_name.clone().or(target.signal_name.clone());
                    set_signal_name_only(&mut target, next_signal_name);
                    target.origin = ConnectionTargetOrigin::Mux;
                    targets.push(target);
                }
            }
        }
        targets
    }

    pub(crate) fn demux_targets(
        &self,
        system: &System,
        block: &Block,
        output_index: u32,
        line_targets: &[Vec<ConnectionTarget>],
    ) -> Vec<ConnectionTarget> {
        let Some(incoming) = incoming_lines_for_block(system, block).into_iter().next() else {
            return Vec::new();
        };
        let Some(line_index) = system
            .lines
            .iter()
            .position(|candidate| same_line(candidate, incoming))
        else {
            return Vec::new();
        };

        line_targets[line_index]
            .iter()
            .filter(|target| {
                target.resolve == Some(ConnectionTargetResolve::Index(output_index))
                    || target.element_index == Some(output_index)
            })
            .cloned()
            .map(|mut target| {
                target.resolve = None;
                target.origin = ConnectionTargetOrigin::Demux;
                target
            })
            .collect()
    }

    pub(crate) fn resolve_from_block_targets(
        &self,
        system: &System,
        block: &Block,
        line_targets: &[Vec<ConnectionTarget>],
    ) -> Vec<ConnectionTarget> {
        let tag = block
            .properties
            .get("GotoTag")
            .map(|s| s.trim())
            .unwrap_or("A");

        let goto_blocks: Vec<&Block> = system
            .blocks
            .iter()
            .filter(|b| block_traits(&b.block_type).signal_role == SignalRole::Goto)
            .filter(|b| b.properties.get("GotoTag").map(|s| s.trim()).unwrap_or("A") == tag)
            .collect();

        let mut targets = Vec::new();
        for goto in goto_blocks {
            for incoming in incoming_lines_for_block(system, goto) {
                let Some(line_index) = system
                    .lines
                    .iter()
                    .position(|candidate| same_line(candidate, incoming))
                else {
                    continue;
                };
                targets.extend(line_targets[line_index].clone());
            }
        }
        targets
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn upstream_propagated_targets(
        &self,
        system: &System,
        _system_path: &[String],
        block: &Block,
        dst: &EndpointRef,
        parent_ctx: Option<&ParentSubsystemContext>,
        child_summaries: &HashMap<String, ChildSubsystemSummary>,
        line_targets: &[Vec<ConnectionTarget>],
    ) -> Vec<ConnectionTarget> {
        let traits = block_traits(&block.block_type);
        match traits.signal_role {
            SignalRole::BusCreator => {
                self.bus_creator_upstream_targets(system, block, line_targets)
            }
            SignalRole::BusSelector | SignalRole::BusAssignment => {
                self.bus_selector_upstream_targets(system, block, line_targets)
            }
            SignalRole::Mux => {
                self.mux_upstream_targets(system, block, dst.port_index, line_targets)
            }
            SignalRole::Demux => self.demux_upstream_targets(system, block, line_targets),
            SignalRole::BoundaryInput | SignalRole::ControlPort | SignalRole::VariantMerge => {
                outgoing_line_indices_for_block(system, block)
                    .into_iter()
                    .flat_map(|(line_index, _)| line_targets[line_index].clone())
                    .collect()
            }
            SignalRole::VariantSelect => {
                if let Some(active) = active_variant_port_index(block) {
                    outgoing_line_targets_for_block_on_port(system, block, active, line_targets)
                } else {
                    outgoing_line_indices_for_block(system, block)
                        .into_iter()
                        .flat_map(|(line_index, _)| line_targets[line_index].clone())
                        .collect()
                }
            }
            SignalRole::BoundaryOutput => parent_ctx
                .and_then(|ctx| ctx.outgoing_by_port.get(&boundary_port_index(block)))
                .cloned()
                .unwrap_or_default(),
            SignalRole::Container => child_summaries
                .get(block.sid.as_deref().unwrap_or_default())
                .filter(|_| !is_control_port_type(&dst.port_type))
                .and_then(|summary| summary.incoming_by_port.get(&dst.port_index))
                .map(|targets| {
                    let mut propagated = targets.clone();
                    if traits.path_prefix_matched {
                        for t in &mut propagated {
                            t.block_type = Some(block.block_type.clone());
                        }
                    }
                    propagated
                })
                .unwrap_or_default(),
            SignalRole::Plain | SignalRole::From | SignalRole::Goto => Vec::new(),
        }
    }

    fn bus_creator_upstream_targets(
        &self,
        system: &System,
        block: &Block,
        line_targets: &[Vec<ConnectionTarget>],
    ) -> Vec<ConnectionTarget> {
        outgoing_line_indices_for_block(system, block)
            .into_iter()
            .flat_map(|(line_index, _)| line_targets[line_index].clone())
            .collect()
    }

    fn bus_selector_upstream_targets(
        &self,
        system: &System,
        block: &Block,
        line_targets: &[Vec<ConnectionTarget>],
    ) -> Vec<ConnectionTarget> {
        outgoing_line_indices_for_block(system, block)
            .into_iter()
            .flat_map(|(line_index, _)| {
                line_targets[line_index]
                    .iter()
                    .cloned()
                    .map(|mut target| {
                        // BusSelector output names (e.g. "<bus_a>") are NOT the
                        // bus's signal names. Clear them so merge_upstream_metadata
                        // does not overwrite the bus line's signal_name. Testpoint
                        // is preserved for cross-boundary propagation.
                        target.signal_name = None;
                        target.signal_names.clear();
                        target
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn mux_upstream_targets(
        &self,
        system: &System,
        block: &Block,
        input_index: u32,
        line_targets: &[Vec<ConnectionTarget>],
    ) -> Vec<ConnectionTarget> {
        outgoing_line_indices_for_block(system, block)
            .into_iter()
            .flat_map(|(line_index, _)| {
                line_targets[line_index]
                    .iter()
                    .filter(move |target| {
                        target.resolve == Some(ConnectionTargetResolve::Index(input_index))
                            || target.element_index == Some(input_index)
                    })
                    .cloned()
                    .map(|mut target| {
                        target.resolve = None;
                        target.element_index = None;
                        target
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn demux_upstream_targets(
        &self,
        system: &System,
        block: &Block,
        line_targets: &[Vec<ConnectionTarget>],
    ) -> Vec<ConnectionTarget> {
        outgoing_line_indices_for_block(system, block)
            .into_iter()
            .flat_map(|(line_index, outgoing_line)| {
                let output_index = outgoing_line.src.as_ref().map(|src| src.port_index);
                line_targets[line_index]
                    .iter()
                    .cloned()
                    .map(move |mut target| {
                        target.resolve = output_index.map(ConnectionTargetResolve::Index);
                        target.element_index = None;
                        target
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}
