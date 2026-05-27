use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use crate::model::{Block, Branch, DashboardBinding, DashboardTargetPath, Line, System};

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
    pub resolve: Option<ConnectionTargetResolve>,
    pub element_index: Option<u32>,
    pub origin: ConnectionTargetOrigin,
    pub signals_only: bool,
    pub testpoint: bool,
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

#[derive(Debug, Clone, Default)]
struct ParentSubsystemContext {
    incoming_by_port: BTreeMap<u32, Vec<ConnectionTarget>>,
    outgoing_by_port: BTreeMap<u32, Vec<ConnectionTarget>>,
}

#[derive(Debug, Clone, Default)]
struct ChildSubsystemSummary {
    incoming_by_port: BTreeMap<u32, Vec<ConnectionTarget>>,
    outgoing_by_port: BTreeMap<u32, Vec<ConnectionTarget>>,
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionTargetResolver {
    block_targets: HashMap<String, Vec<ConnectionTarget>>,
    line_targets: HashMap<String, Vec<ConnectionTarget>>,
    model_name: String,
}

impl ConnectionTargetResolver {
    pub fn new(root: &System) -> Self {
        let mut resolver = Self {
            block_targets: HashMap::new(),
            line_targets: HashMap::new(),
            model_name: root.properties.get("Name").cloned().unwrap_or_default(),
        };
        let empty_path: Vec<String> = Vec::new();
        resolver.resolve_system(root, &empty_path, None);
        resolver
    }

    pub fn block_targets_for_block(
        &self,
        system_path: &[String],
        block: &Block,
    ) -> Vec<ConnectionTarget> {
        let key = block_cache_key(system_path, block);
        self.block_targets.get(&key).cloned().unwrap_or_default()
    }

    pub fn line_targets_for_line(
        &self,
        system_path: &[String],
        line: &Line,
    ) -> Vec<ConnectionTarget> {
        let key = line_cache_key(system_path, line);
        self.line_targets.get(&key).cloned().unwrap_or_default()
    }

    fn resolve_system(
        &mut self,
        system: &System,
        system_path: &[String],
        parent_ctx: Option<&ParentSubsystemContext>,
    ) -> ChildSubsystemSummary {
        let block_lookup = build_block_lookup(system);
        let mut line_targets: Vec<Vec<ConnectionTarget>> = system
            .lines
            .iter()
            .map(|line| self.base_line_targets(system, system_path, &block_lookup, line))
            .collect();

        self.propagate_line_targets(
            system,
            system_path,
            &block_lookup,
            parent_ctx,
            &HashMap::new(),
            &mut line_targets,
        );
        self.propagate_line_metadata_upward(
            system,
            &block_lookup,
            parent_ctx,
            &HashMap::new(),
            &mut line_targets,
        );

        let mut child_summaries: HashMap<String, ChildSubsystemSummary> = HashMap::new();
        for block in &system.blocks {
            if let Some(subsystem) = &block.subsystem {
                let child_path = child_system_path(system_path, &block.name);
                let parent_ctx = ParentSubsystemContext {
                    incoming_by_port: incoming_targets_by_port(system, block, &line_targets),
                    outgoing_by_port: outgoing_targets_by_port(system, block, &line_targets),
                };
                let summary = self.resolve_system(subsystem, &child_path, Some(&parent_ctx));
                if let Some(sid) = &block.sid {
                    child_summaries.insert(sid.clone(), summary);
                }
            }
        }

        self.propagate_line_targets(
            system,
            system_path,
            &block_lookup,
            parent_ctx,
            &child_summaries,
            &mut line_targets,
        );
        self.propagate_line_metadata_upward(
            system,
            &block_lookup,
            parent_ctx,
            &child_summaries,
            &mut line_targets,
        );

        for (line, targets) in system.lines.iter().zip(line_targets.iter()) {
            self.line_targets.insert(
                line_cache_key(system_path, line),
                dedup_targets(targets.clone()),
            );
        }

        for block in &system.blocks {
            let mut targets = Vec::new();
            targets.push(ConnectionTarget::new(
                self.full_block_path(system_path, &block.name),
                ConnectionTargetOrigin::SelfBlock,
            ));

            targets.extend(self.direct_internal_block_targets(system_path, block));

            for incoming in incoming_lines_for_block(system, block) {
                if let Some(index) = system
                    .lines
                    .iter()
                    .position(|candidate| same_line(candidate, incoming))
                {
                    targets.extend(line_targets[index].clone());
                }
            }

            if let Some(binding) = &block.dashboard_binding {
                let target_path =
                    qualify_external_path(&self.model_name, dashboard_binding_block_path(binding));
                let mut target =
                    ConnectionTarget::new(target_path, ConnectionTargetOrigin::DashboardBinding);
                if let DashboardBinding::SignalSpec { signal_name, .. } = binding {
                    set_signal_name_only(&mut target, Some(signal_name.clone()));
                }
                target.signals_only = matches!(binding, DashboardBinding::SignalSpec { .. });
                if let DashboardBinding::SignalSpec {
                    target_path, ..
                } = binding
                {
                    target.element_index = target_path.port_index;
                }
                let binding_target_path = dashboard_binding_target_path(binding);
                if !binding_target_path.is_empty() {
                    target.resolve = Some(ConnectionTargetResolve::TargetPath(
                        binding_target_path.clone(),
                    ));
                }
                targets.push(target);
            }

            let deduped = dedup_targets(targets);
            self.block_targets
                .insert(block_cache_key(system_path, block), deduped);
        }

        ChildSubsystemSummary {
            incoming_by_port: child_incoming_targets_by_port(system, &line_targets),
            outgoing_by_port: child_outgoing_targets_by_port(
                self,
                system,
                system_path,
                &line_targets,
            ),
        }
    }

    fn propagate_line_targets(
        &self,
        system: &System,
        system_path: &[String],
        block_lookup: &HashMap<&str, &Block>,
        parent_ctx: Option<&ParentSubsystemContext>,
        child_summaries: &HashMap<String, ChildSubsystemSummary>,
        line_targets: &mut [Vec<ConnectionTarget>],
    ) {
        for _ in 0..8 {
            let mut changed = false;
            for (index, line) in system.lines.iter().enumerate() {
                let Some(src) = &line.src else {
                    continue;
                };
                let Some(block) = block_lookup.get(src.sid.as_str()).copied() else {
                    continue;
                };

                let mut new_targets = match block.block_type.as_str() {
                    "BusCreator" => {
                        self.bus_creator_targets(system, system_path, block, line, line_targets)
                    }
                    "BusSelector" => self.bus_selector_targets(system, block, line, line_targets),
                    "Mux" => self.mux_targets(system, block, line_targets),
                    "Demux" => self.demux_targets(system, block, src.port_index, line_targets),
                    "Inport" => parent_ctx
                        .and_then(|ctx| ctx.incoming_by_port.get(&boundary_port_index(block)))
                        .map(|targets| {
                            boundary_targets(
                                targets,
                                self.full_block_path(system_path, &block.name),
                            )
                        })
                        .unwrap_or_else(|| {
                            self.base_line_targets(system, system_path, block_lookup, line)
                        }),
                    "SubSystem" | "Reference" => child_summaries
                        .get(&src.sid)
                        .and_then(|summary| summary.outgoing_by_port.get(&src.port_index))
                        .map(|targets| targets.clone())
                        .unwrap_or_else(|| {
                            self.base_line_targets(system, system_path, block_lookup, line)
                        }),
                    _ => self.base_line_targets(system, system_path, block_lookup, line),
                };

                if matches!(
                    block.block_type.as_str(),
                    "BusCreator"
                        | "BusSelector"
                        | "Mux"
                        | "Demux"
                        | "Inport"
                        | "SubSystem"
                        | "Reference"
                ) {
                    apply_local_line_metadata(line, &mut new_targets);
                    apply_source_port_testpoint(block, line, &mut new_targets);
                }

                let deduped = dedup_targets(new_targets);
                if deduped != line_targets[index] {
                    line_targets[index] = deduped;
                    changed = true;
                }
            }

            if !changed {
                break;
            }
        }
    }

    fn propagate_line_metadata_upward(
        &self,
        system: &System,
        block_lookup: &HashMap<&str, &Block>,
        parent_ctx: Option<&ParentSubsystemContext>,
        child_summaries: &HashMap<String, ChildSubsystemSummary>,
        line_targets: &mut [Vec<ConnectionTarget>],
    ) {
        for _ in 0..8 {
            let mut changed = false;

            for (index, line) in system.lines.iter().enumerate() {
                let Some(dst) = &line.dst else {
                    continue;
                };
                let Some(block) = block_lookup.get(dst.sid.as_str()).copied() else {
                    continue;
                };

                let propagated = self.upstream_propagated_targets(
                    system,
                    block,
                    line,
                    parent_ctx,
                    child_summaries,
                    line_targets,
                );
                if propagated.is_empty() {
                    continue;
                }

                let merged = merge_upstream_metadata(
                    line,
                    &line_targets[index],
                    &propagated,
                    matches!(
                        block.block_type.as_str(),
                        "SubSystem" | "Reference" | "Outport"
                    ),
                );
                if merged != line_targets[index] {
                    line_targets[index] = merged;
                    changed = true;
                }
            }

            if !changed {
                break;
            }
        }
    }

    fn base_line_targets(
        &self,
        system: &System,
        system_path: &[String],
        block_lookup: &HashMap<&str, &Block>,
        line: &Line,
    ) -> Vec<ConnectionTarget> {
        let Some(src) = &line.src else {
            return Vec::new();
        };
        let Some(block) = block_lookup.get(src.sid.as_str()).copied() else {
            return Vec::new();
        };

        let signal_name = routing_line_signal_name(system, line);
        let mut target = ConnectionTarget::new(
            self.full_block_path(system_path, &block.name),
            ConnectionTargetOrigin::SourceBlock,
        );
        target.signals_only = true;
        target.testpoint =
            port_testpoint(block, src.port_type.as_str(), src.port_index) || line_testpoint(line);
        if src.port_type == "out" && output_port_count(block) > 1 {
            target.element_index = Some(src.port_index);
        }
        set_signal_name_only(&mut target, signal_name);
        apply_line_resolve_hint(line, block_lookup, &mut target);
        vec![target]
    }

    fn bus_creator_targets(
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
            let input_index = incoming.dst.as_ref().map(|dst| dst.port_index).unwrap_or(1);
            let signal_name = explicit_line_signal_name(incoming);
            for mut target in line_targets[line_index].clone() {
                let next_signal_name = signal_name.clone().or(target.signal_name.clone());
                let next_resolve_signal = signal_name
                    .clone()
                    .or_else(|| target.signal_name.clone())
                    .or_else(|| resolve_signal_value(&target.resolve).map(str::to_string))
                    .or_else(|| Some(format!("signal{input_index}")));
                set_signal_name_only(&mut target, next_signal_name);
                set_signal_resolve(&mut target, next_resolve_signal);
                target.origin = ConnectionTargetOrigin::BusCreator;
                targets.push(target);
            }
        }
        targets
    }

    fn bus_selector_targets(
        &self,
        system: &System,
        block: &Block,
        line: &Line,
        line_targets: &[Vec<ConnectionTarget>],
    ) -> Vec<ConnectionTarget> {
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
        let Some(selected_name) = selected_name else {
            return Vec::new();
        };

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
                matches_resolve_signal(target, &selected_name)
                    || target
                        .signal_name
                        .as_deref()
                        .is_some_and(|name| signal_keys_match(name, &selected_name))
            })
            .cloned()
            .map(|mut target| {
                target.origin = ConnectionTargetOrigin::BusSelector;
                target
            })
            .collect()
    }

    fn mux_targets(
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
            let input_index = incoming.dst.as_ref().map(|dst| dst.port_index).unwrap_or(1);
            let signal_name = explicit_line_signal_name(incoming);
            for mut target in line_targets[line_index].clone() {
                target.resolve = Some(ConnectionTargetResolve::Index(input_index));
                let next_signal_name = signal_name.clone().or(target.signal_name.clone());
                set_signal_name_only(&mut target, next_signal_name);
                target.origin = ConnectionTargetOrigin::Mux;
                targets.push(target);
            }
        }
        targets
    }

    fn demux_targets(
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

    fn upstream_propagated_targets(
        &self,
        system: &System,
        block: &Block,
        line: &Line,
        parent_ctx: Option<&ParentSubsystemContext>,
        child_summaries: &HashMap<String, ChildSubsystemSummary>,
        line_targets: &[Vec<ConnectionTarget>],
    ) -> Vec<ConnectionTarget> {
        match block.block_type.as_str() {
            "BusCreator" => self.bus_creator_upstream_targets(system, block, line_targets),
            "BusSelector" => self.bus_selector_upstream_targets(system, block, line_targets),
            "Mux" => self.mux_upstream_targets(system, block, line, line_targets),
            "Demux" => self.demux_upstream_targets(system, block, line_targets),
            "Inport" => outgoing_line_indices_for_block(system, block)
                .into_iter()
                .flat_map(|(line_index, _)| line_targets[line_index].clone())
                .collect(),
            "Outport" => parent_ctx
                .and_then(|ctx| ctx.outgoing_by_port.get(&boundary_port_index(block)))
                .cloned()
                .unwrap_or_default(),
            "SubSystem" | "Reference" => child_summaries
                .get(block.sid.as_deref().unwrap_or_default())
                .and_then(|summary| {
                    line.dst
                        .as_ref()
                        .and_then(|dst| summary.incoming_by_port.get(&dst.port_index))
                })
                .cloned()
                .unwrap_or_default(),
            _ => Vec::new(),
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
            .flat_map(|(line_index, _)| line_targets[line_index].clone())
            .collect()
    }

    fn mux_upstream_targets(
        &self,
        system: &System,
        block: &Block,
        line: &Line,
        line_targets: &[Vec<ConnectionTarget>],
    ) -> Vec<ConnectionTarget> {
        let Some(input_index) = line.dst.as_ref().map(|dst| dst.port_index) else {
            return Vec::new();
        };

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

    fn full_block_path(&self, system_path: &[String], block_name: &str) -> String {
        let mut parts = Vec::new();
        if let Some(model_name) = normalized_path_segment(&self.model_name) {
            parts.push(model_name);
        }
        parts.extend(
            system_path
                .iter()
                .filter_map(|part| normalized_path_segment(part)),
        );
        if let Some(block_name) = normalized_path_segment(block_name) {
            parts.push(block_name);
        }
        parts.join("/")
    }

    fn direct_internal_block_targets(
        &self,
        system_path: &[String],
        block: &Block,
    ) -> Vec<ConnectionTarget> {
        let Some(subsystem) = &block.subsystem else {
            return Vec::new();
        };

        let child_path = child_system_path(system_path, &block.name);
        subsystem
            .blocks
            .iter()
            .map(|child| {
                ConnectionTarget::new(
                    self.full_block_path(&child_path, &child.name),
                    ConnectionTargetOrigin::Internal,
                )
            })
            .collect()
    }
}

pub fn debug_print_block_targets(root: &System, system_path: &[String], block: &Block) {
    let resolver = ConnectionTargetResolver::new(root);
    let targets = resolver.block_targets_for_block(system_path, block);
    println!("  [Targets] block '{}'", block.name);
    print_targets(&targets);
}

pub fn debug_print_line_targets(root: &System, system_path: &[String], line: &Line) {
    let resolver = ConnectionTargetResolver::new(root);
    let targets = resolver.line_targets_for_line(system_path, line);
    println!("  [Targets] line {}", line_identity(line));
    print_targets(&targets);
}

fn print_targets(targets: &[ConnectionTarget]) {
    if targets.is_empty() {
        println!("    (no targets)");
        return;
    }

    for target in targets {
        println!(
            "    - path='{}' origin={:?} signal={:?} resolve={:?} index={:?} signals_only={} testpoint={}",
            target.path,
            target.origin,
            target.signal_name,
            target.resolve,
            target.element_index,
            target.signals_only,
            target.testpoint
        );
    }
}

fn build_block_lookup(system: &System) -> HashMap<&str, &Block> {
    system
        .blocks
        .iter()
        .filter_map(|block| block.sid.as_deref().map(|sid| (sid, block)))
        .collect()
}

fn child_system_path(system_path: &[String], block_name: &str) -> Vec<String> {
    let mut path = system_path.to_vec();
    path.push(block_name.to_string());
    path
}

fn boundary_port_index(block: &Block) -> u32 {
    block
        .properties
        .get("Port")
        .or_else(|| block.properties.get("PortNumber"))
        .and_then(|value| value.trim().parse::<u32>().ok())
        .unwrap_or(1)
}

fn incoming_targets_by_port(
    system: &System,
    block: &Block,
    line_targets: &[Vec<ConnectionTarget>],
) -> BTreeMap<u32, Vec<ConnectionTarget>> {
    let mut by_port = BTreeMap::new();
    let Some(block_sid) = block.sid.as_deref() else {
        return by_port;
    };

    for (line, targets) in system.lines.iter().zip(line_targets.iter()) {
        if line_targets_block_sid(line, block_sid) {
            let port_index = line
                .dst
                .as_ref()
                .filter(|dst| dst.sid == block_sid)
                .map(|dst| dst.port_index)
                .unwrap_or(1);
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

fn child_outgoing_targets_by_port(
    resolver: &ConnectionTargetResolver,
    system: &System,
    system_path: &[String],
    line_targets: &[Vec<ConnectionTarget>],
) -> BTreeMap<u32, Vec<ConnectionTarget>> {
    let mut by_port = BTreeMap::new();
    let inport_boundary_paths = subsystem_boundary_paths(resolver, system, system_path, "Inport");
    for block in &system.blocks {
        if block.block_type != "Outport" {
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
            by_port.insert(
                port_index,
                boundary_targets(&targets, resolver.full_block_path(system_path, &block.name)),
            );
        }
    }
    by_port
}

fn child_incoming_targets_by_port(
    system: &System,
    line_targets: &[Vec<ConnectionTarget>],
) -> BTreeMap<u32, Vec<ConnectionTarget>> {
    let mut by_port = BTreeMap::new();
    for block in &system.blocks {
        if block.block_type != "Inport" {
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

fn boundary_targets(targets: &[ConnectionTarget], boundary_path: String) -> Vec<ConnectionTarget> {
    let mut combined = targets.to_vec();
    combined.extend(targets.iter().cloned().map(|mut target| {
        target.path = boundary_path.clone();
        target
    }));
    dedup_targets(combined)
}

fn apply_local_line_metadata(line: &Line, targets: &mut [ConnectionTarget]) {
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

fn apply_source_port_testpoint(block: &Block, line: &Line, targets: &mut [ConnectionTarget]) {
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

fn merge_upstream_metadata(
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
        let propagated = propagated_targets
            .iter()
            .filter(|candidate| {
                metadata_paths_match(
                    target,
                    candidate,
                    path_counts.get(target.path.as_str()).copied().unwrap_or(0),
                    allow_cross_path,
                )
            })
            .collect::<Vec<_>>();
        if propagated.is_empty() {
            continue;
        }

        let propagated_name = propagated
            .iter()
            .find_map(|candidate| candidate.signal_name.clone());
        set_signal_name_only(
            target,
            explicit_name
                .clone()
                .or(propagated_name)
                .or(target.signal_name.clone()),
        );
        target.testpoint = explicit_testpoint
            || target.testpoint
            || propagated.iter().any(|candidate| candidate.testpoint);
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

fn set_signal_name_only(target: &mut ConnectionTarget, signal_name: Option<String>) {
    target.signal_name = signal_name.and_then(|signal_name| normalized_path_segment(&signal_name));
}

fn set_signal_resolve(target: &mut ConnectionTarget, signal_name: Option<String>) {
    target.resolve = signal_name
        .and_then(|signal_name| normalize_resolve_signal(&signal_name))
        .map(ConnectionTargetResolve::Signal);
}

fn normalize_resolve_signal(signal_name: &str) -> Option<String> {
    let trimmed = signal_name
        .trim()
        .trim_start_matches('<')
        .trim_end_matches('>');
    normalized_path_segment(trimmed)
}

fn resolve_signal_value(resolve: &Option<ConnectionTargetResolve>) -> Option<&str> {
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

fn matches_resolve_signal(target: &ConnectionTarget, selected_name: &str) -> bool {
    resolve_signal_value(&target.resolve)
        .is_some_and(|signal_name| signal_keys_match(signal_name, selected_name))
}

fn signal_keys_match(left: &str, right: &str) -> bool {
    let Some(left) = normalize_resolve_signal(left) else {
        return false;
    };
    let Some(right) = normalize_resolve_signal(right) else {
        return false;
    };
    left.eq_ignore_ascii_case(&right)
}

fn apply_line_resolve_hint(
    line: &Line,
    block_lookup: &HashMap<&str, &Block>,
    target: &mut ConnectionTarget,
) {
    if let Some(signal_name) = explicit_line_signal_name(line) {
        set_signal_resolve(target, Some(signal_name));
        return;
    }

    if let Some(dst) = &line.dst {
        if let Some(block) = block_lookup.get(dst.sid.as_str()) {
            if block.block_type == "Mux" {
                target.resolve = Some(ConnectionTargetResolve::Index(dst.port_index));
                return;
            }
        }
    }

    if target.resolve.is_none() && target.element_index.is_some() {
        target.resolve = target.element_index.map(ConnectionTargetResolve::Index);
    }
}

fn normalized_path_segment(segment: &str) -> Option<String> {
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

fn incoming_lines_for_block<'a>(system: &'a System, block: &Block) -> Vec<&'a Line> {
    let Some(block_sid) = block.sid.as_deref() else {
        return Vec::new();
    };
    system
        .lines
        .iter()
        .filter(|line| line_targets_block_sid(line, block_sid))
        .collect()
}

fn outgoing_line_indices_for_block<'a>(
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

fn outgoing_targets_by_port(
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

fn port_signal_name(block: &Block, port_type: &str, port_index: u32) -> Option<String> {
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

fn port_testpoint(block: &Block, port_type: &str, port_index: u32) -> bool {
    block
        .ports
        .iter()
        .find(|port| port.port_type == port_type && port.index.unwrap_or(0) == port_index)
        .and_then(|port| port.properties.get("TestPoint"))
        .is_some_and(|value| matches!(value.trim(), "on" | "true" | "1" | "On" | "True"))
}

fn line_testpoint(line: &Line) -> bool {
    line.properties
        .get("TestPoint")
        .is_some_and(|value| matches!(value.trim(), "on" | "true" | "1" | "On" | "True"))
}

fn output_port_count(block: &Block) -> u32 {
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
    boundary_type: &str,
) -> BTreeSet<String> {
    system
        .blocks
        .iter()
        .filter(|block| block.block_type == boundary_type)
        .map(|block| resolver.full_block_path(system_path, &block.name))
        .collect()
}

fn explicit_line_signal_name(line: &Line) -> Option<String> {
    line.name
        .as_ref()
        .map(|name| name.trim())
        .filter(|name| !name.is_empty())
        .and_then(normalized_path_segment)
}

fn routing_line_signal_name(_system: &System, line: &Line) -> Option<String> {
    explicit_line_signal_name(line)
}

fn block_cache_key(system_path: &[String], block: &Block) -> String {
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

fn line_cache_key(system_path: &[String], line: &Line) -> String {
    let mut key = system_path.join("/");
    key.push('|');
    key.push_str(&line_identity(line));
    key
}

fn line_identity(line: &Line) -> String {
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

fn same_line(left: &Line, right: &Line) -> bool {
    line_identity(left) == line_identity(right)
}

fn qualify_external_path(model_name: &str, raw_path: &str) -> String {
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

fn dashboard_binding_block_path(binding: &DashboardBinding) -> &str {
    match binding {
        DashboardBinding::ParamSource { block_path, .. } => block_path,
        DashboardBinding::SignalSpec { block_path, .. } => block_path,
    }
}

fn dashboard_binding_target_path(binding: &DashboardBinding) -> &DashboardTargetPath {
    match binding {
        DashboardBinding::ParamSource { target_path, .. } => target_path,
        DashboardBinding::SignalSpec { target_path, .. } => target_path,
    }
}

fn dedup_targets(targets: Vec<ConnectionTarget>) -> Vec<ConnectionTarget> {
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
            target.origin.clone(),
            target.signals_only,
        );
        if let Some(index) = seen.get(&key).copied() {
            if let Some(existing) = out.get_mut(index) {
                existing.testpoint = existing.testpoint || target.testpoint;
            }
        } else {
            seen.insert(key, out.len());
            out.push(target);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::{
        ConnectionTarget, ConnectionTargetOrigin, ConnectionTargetResolve, ConnectionTargetResolver,
    };
    use crate::model::{Block, EndpointRef, Line, NameLocation, Point, Port, System, ValueKind};

    #[test]
    fn bus_selector_uses_named_bus_creator_input() {
        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![
                block("Constant", "A", "1", vec![port("out", 1, None)], None, &[]),
                block("Constant", "B", "2", vec![port("out", 1, None)], None, &[]),
                block(
                    "BusCreator",
                    "BusCreator",
                    "3",
                    vec![
                        port("in", 1, None),
                        port("in", 2, None),
                        port("out", 1, None),
                    ],
                    None,
                    &[],
                ),
                block(
                    "BusSelector",
                    "BusSelector",
                    "4",
                    vec![port("in", 1, None), port("out", 1, Some("beta"))],
                    None,
                    &[],
                ),
                block("Display", "Sink", "5", vec![port("in", 1, None)], None, &[]),
            ],
            lines: vec![
                line("1", 1, "3", 1, Some("alpha")),
                line("2", 1, "3", 2, Some("beta")),
                line("3", 1, "4", 1, None),
                line("4", 1, "5", 1, None),
            ],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.line_targets_for_line(&[], &system.lines[3]);

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].path, "model/B");
        assert_eq!(targets[0].signal_name.as_deref(), Some("beta"));
        assert_eq!(targets[0].origin, ConnectionTargetOrigin::BusSelector);
    }

    #[test]
    fn demux_uses_matching_mux_input_index() {
        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![
                block("Constant", "A", "1", vec![port("out", 1, None)], None, &[]),
                block("Constant", "B", "2", vec![port("out", 1, None)], None, &[]),
                block(
                    "Mux",
                    "Mux",
                    "3",
                    vec![
                        port("in", 1, None),
                        port("in", 2, None),
                        port("out", 1, None),
                    ],
                    None,
                    &[],
                ),
                block(
                    "Demux",
                    "Demux",
                    "4",
                    vec![
                        port("in", 1, None),
                        port("out", 1, None),
                        port("out", 2, None),
                    ],
                    None,
                    &[],
                ),
                block(
                    "Display",
                    "Sink1",
                    "5",
                    vec![port("in", 1, None)],
                    None,
                    &[],
                ),
                block(
                    "Display",
                    "Sink2",
                    "6",
                    vec![port("in", 1, None)],
                    None,
                    &[],
                ),
            ],
            lines: vec![
                line("1", 1, "3", 1, Some("alpha")),
                line("2", 1, "3", 2, Some("beta")),
                line("3", 1, "4", 1, None),
                line("4", 1, "5", 1, None),
                line("4", 2, "6", 1, None),
            ],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.line_targets_for_line(&[], &system.lines[4]);

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].path, "model/B");
        assert_eq!(targets[0].signal_name.as_deref(), Some("beta"));
        assert_eq!(targets[0].origin, ConnectionTargetOrigin::Demux);
    }

    #[test]
    fn subsystem_outport_forwards_parent_input_target() {
        let child_system = System {
            properties: IndexMap::new(),
            blocks: vec![
                block(
                    "Inport",
                    "In1",
                    "10",
                    vec![port("out", 1, None)],
                    None,
                    &[("Port", "1")],
                ),
                block(
                    "Outport",
                    "Out1",
                    "11",
                    vec![port("in", 1, None)],
                    None,
                    &[("Port", "1")],
                ),
            ],
            lines: vec![line("10", 1, "11", 1, None)],
            annotations: Vec::new(),
            chart: None,
        };

        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![
                block(
                    "Constant",
                    "Src",
                    "1",
                    vec![port("out", 1, None)],
                    None,
                    &[],
                ),
                block(
                    "SubSystem",
                    "Sub",
                    "2",
                    vec![port("in", 1, None), port("out", 1, None)],
                    Some(child_system),
                    &[],
                ),
                block("Display", "Sink", "3", vec![port("in", 1, None)], None, &[]),
            ],
            lines: vec![
                line("1", 1, "2", 1, Some("input")),
                line("2", 1, "3", 1, None),
            ],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.line_targets_for_line(&[], &system.lines[1]);

        assert!(targets.iter().any(|target| {
            target.path == "model/Src"
                && target.signal_name.as_deref() == Some("input")
                && target.signals_only
        }));
        assert!(targets.iter().any(|target| {
            target.path == "model/Sub/Out1" && target.signal_name.as_deref() == Some("input")
        }));
        assert!(!targets.iter().any(|target| target.path == "model/Sub/In1"));
    }

    #[test]
    fn mux_does_not_append_explicit_names_to_forwarded_boundary_paths() {
        let child_system = System {
            properties: IndexMap::new(),
            blocks: vec![
                block(
                    "Inport",
                    "In1",
                    "10",
                    vec![port("out", 1, None)],
                    None,
                    &[("Port", "1")],
                ),
                block(
                    "Outport",
                    "Out1",
                    "11",
                    vec![port("in", 1, None)],
                    None,
                    &[("Port", "1")],
                ),
            ],
            lines: vec![line("10", 1, "11", 1, None)],
            annotations: Vec::new(),
            chart: None,
        };

        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![
                block(
                    "Constant",
                    "Src",
                    "1",
                    vec![port("out", 1, None)],
                    None,
                    &[],
                ),
                block(
                    "SubSystem",
                    "Sub",
                    "2",
                    vec![port("in", 1, None), port("out", 1, None)],
                    Some(child_system),
                    &[],
                ),
                block(
                    "Constant",
                    "Other",
                    "3",
                    vec![port("out", 1, None)],
                    None,
                    &[],
                ),
                block(
                    "Mux",
                    "Mux",
                    "4",
                    vec![
                        port("in", 1, None),
                        port("in", 2, None),
                        port("out", 1, None),
                    ],
                    None,
                    &[],
                ),
                block("Display", "Sink", "5", vec![port("in", 1, None)], None, &[]),
            ],
            lines: vec![
                line("1", 1, "2", 1, Some("input")),
                line("2", 1, "4", 1, Some("alpha")),
                line("3", 1, "4", 2, Some("beta")),
                line("4", 1, "5", 1, None),
            ],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.line_targets_for_line(&[], &system.lines[3]);

        assert!(
            targets.iter().any(|target| {
                target.path == "model/Sub/Out1"
                    && target.signal_name.as_deref() == Some("alpha")
                    && target.origin == ConnectionTargetOrigin::Mux
            }),
            "targets: {targets:?}"
        );
        assert!(
            !targets
                .iter()
                .any(|target| target.path == "model/Sub/Out1/alpha")
        );
    }

    #[test]
    fn subsystem_block_includes_direct_internal_block_targets_only() {
        let nested_system = System {
            properties: IndexMap::new(),
            blocks: vec![block(
                "Constant",
                "Deep",
                "20",
                vec![port("out", 1, None)],
                None,
                &[],
            )],
            lines: Vec::new(),
            annotations: Vec::new(),
            chart: None,
        };

        let child_system = System {
            properties: IndexMap::new(),
            blocks: vec![
                block(
                    "Constant",
                    "InnerDirect",
                    "10",
                    vec![port("out", 1, None)],
                    None,
                    &[],
                ),
                block(
                    "SubSystem",
                    "Nested",
                    "11",
                    vec![port("in", 1, None), port("out", 1, None)],
                    Some(nested_system),
                    &[],
                ),
            ],
            lines: Vec::new(),
            annotations: Vec::new(),
            chart: None,
        };

        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![block(
                "SubSystem",
                "Sub",
                "1",
                vec![port("in", 1, None), port("out", 1, None)],
                Some(child_system),
                &[],
            )],
            lines: Vec::new(),
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.block_targets_for_block(&[], &system.blocks[0]);

        assert!(targets.iter().any(|target| {
            target.path == "model/Sub/InnerDirect"
                && target.origin == ConnectionTargetOrigin::Internal
        }));
        assert!(targets.iter().any(|target| {
            target.path == "model/Sub/Nested" && target.origin == ConnectionTargetOrigin::Internal
        }));
        assert!(
            !targets
                .iter()
                .any(|target| target.path == "model/Sub/Nested/Deep")
        );
    }

    #[test]
    fn dashboard_signal_bindings_are_marked_signals_only() {
        let mut signal_block = block("DisplayBlock", "Gauge", "1", vec![], None, &[]);
        signal_block.dashboard_binding = Some(crate::model::DashboardBinding::SignalSpec {
            block_path: "Source".to_string(),
            signal_name: "sig".to_string(),
            target_path: crate::model::DashboardTargetPath::default(),
            uuid: "uuid-1".to_string(),
        });

        let mut param_block = block("KnobBlock", "Knob", "2", vec![], None, &[]);
        param_block.dashboard_binding = Some(crate::model::DashboardBinding::ParamSource {
            block_path: "ParamBlock".to_string(),
            param_name: "Value".to_string(),
            target_path: crate::model::DashboardTargetPath::default(),
            uuid: "uuid-2".to_string(),
        });

        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![signal_block, param_block],
            lines: Vec::new(),
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let signal_targets = resolver.block_targets_for_block(&[], &system.blocks[0]);
        let param_targets = resolver.block_targets_for_block(&[], &system.blocks[1]);

        assert!(signal_targets.iter().any(|target| {
            target.origin == ConnectionTargetOrigin::DashboardBinding
                && target.path == "model/Source"
                && target.signals_only
        }));
        assert!(param_targets.iter().any(|target| {
            target.origin == ConnectionTargetOrigin::DashboardBinding
                && target.path == "model/ParamBlock"
                && !target.signals_only
        }));
    }

    #[test]
    fn dashboard_binding_target_uses_binding_payload() {
        let source = block(
            "Gain",
            "Source",
            "1",
            vec![port("out", 1, Some("other")), port("out", 2, Some("src_signal"))],
            None,
            &[],
        );
        let sink = block(
            "Terminator",
            "Sink",
            "2",
            vec![port("in", 1, None)],
            None,
            &[],
        );
        let mut dashboard = block("DisplayBlock", "Gauge", "3", vec![], None, &[]);
        dashboard.dashboard_binding = Some(crate::model::DashboardBinding::SignalSpec {
            block_path: "Source".to_string(),
            signal_name: "src_signal".to_string(),
            target_path: crate::model::DashboardTargetPath {
                port_index: Some(2),
                ..Default::default()
            },
            uuid: "uuid-propagate".to_string(),
        });

        let mut line = line("1", 2, "2", 1, Some("src_signal"));
        line.properties.insert("TestPoint".to_string(), "on".to_string());

        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![source, sink, dashboard],
            lines: vec![line],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let dashboard_targets = resolver.block_targets_for_block(&[], &system.blocks[2]);
        let dashboard_target = dashboard_targets
            .iter()
            .find(|target| target.origin == ConnectionTargetOrigin::DashboardBinding)
            .expect("dashboard target");

        assert_eq!(dashboard_target.path, "model/Source");
        assert_eq!(dashboard_target.signal_name.as_deref(), Some("src_signal"));
        assert_eq!(
            dashboard_target.resolve,
            Some(ConnectionTargetResolve::TargetPath(
                crate::model::DashboardTargetPath {
                    port_index: Some(2),
                    ..Default::default()
                }
            ))
        );
        assert_eq!(dashboard_target.element_index, Some(2));
        assert!(!dashboard_target.testpoint);
        assert!(dashboard_target.signals_only);
    }

    #[test]
    fn dashboard_binding_target_propagates_source_index_without_incoming_line() {
        let source = block(
            "ComplexToRealImag",
            "Complex to Real-Imag1",
            "1",
            vec![port("out", 1, Some("re")), port("out", 2, Some("im"))],
            None,
            &[],
        );
        let mut dashboard = block("DisplayBlock", "Gauge", "2", vec![], None, &[]);
        dashboard.dashboard_binding = Some(crate::model::DashboardBinding::SignalSpec {
            block_path: "Complex to Real-Imag1".to_string(),
            signal_name: "im".to_string(),
            target_path: crate::model::DashboardTargetPath {
                port_index: Some(2),
                ..Default::default()
            },
            uuid: "uuid-dashboard-only".to_string(),
        });

        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![source, dashboard],
            lines: Vec::new(),
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let dashboard_targets = resolver.block_targets_for_block(&[], &system.blocks[1]);
        let dashboard_target = dashboard_targets
            .iter()
            .find(|target| target.origin == ConnectionTargetOrigin::DashboardBinding)
            .expect("dashboard target");

        assert_eq!(dashboard_target.path, "model/Complex to Real-Imag1");
        assert_eq!(dashboard_target.signal_name.as_deref(), Some("im"));
        assert_eq!(
            dashboard_target.resolve,
            Some(ConnectionTargetResolve::TargetPath(
                crate::model::DashboardTargetPath {
                    port_index: Some(2),
                    ..Default::default()
                }
            ))
        );
        assert_eq!(dashboard_target.element_index, Some(2));
        assert!(dashboard_target.signals_only);
    }

    #[test]
    fn dashboard_binding_target_path_index_wins_over_same_path_signal_name_match() {
        let source = block(
            "Demux",
            "Source",
            "1",
            vec![port("out", 1, Some("requested_signal")), port("out", 2, None)],
            None,
            &[],
        );
        let mut dashboard = block("DisplayBlock", "Gauge", "2", vec![], None, &[]);
        dashboard.dashboard_binding = Some(crate::model::DashboardBinding::SignalSpec {
            block_path: "Source".to_string(),
            signal_name: "requested_signal".to_string(),
            target_path: crate::model::DashboardTargetPath {
                port_index: Some(2),
                ..Default::default()
            },
            uuid: "uuid-index-wins".to_string(),
        });

        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![source, dashboard],
            lines: Vec::new(),
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let dashboard_targets = resolver.block_targets_for_block(&[], &system.blocks[1]);
        let dashboard_target = dashboard_targets
            .iter()
            .find(|target| target.origin == ConnectionTargetOrigin::DashboardBinding)
            .expect("dashboard target");

        assert_eq!(dashboard_target.path, "model/Source");
        assert_eq!(dashboard_target.element_index, Some(2));
        assert_eq!(dashboard_target.signal_name.as_deref(), Some("requested_signal"));
        assert_eq!(
            dashboard_target.resolve,
            Some(ConnectionTargetResolve::TargetPath(
                crate::model::DashboardTargetPath {
                    port_index: Some(2),
                    ..Default::default()
                }
            ))
        );
        assert!(dashboard_target.signals_only);
    }

    #[test]
    fn base_line_targets_preserve_source_port_testpoint() {
        let mut source = block(
            "Constant",
            "Source",
            "1",
            vec![port("out", 1, Some("sig"))],
            None,
            &[],
        );
        source.ports[0]
            .properties
            .insert("TestPoint".to_string(), "on".to_string());

        let sink = block("Display", "Sink", "2", vec![port("in", 1, None)], None, &[]);
        let wire = line("1", 1, "2", 1, Some("sig"));
        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![source, sink],
            lines: vec![wire.clone()],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.line_targets_for_line(&[], &wire);

        assert!(targets.iter().any(|target| target.testpoint));
        assert!(targets.iter().any(|target| {
            target.path == "model/Source" && target.signal_name.as_deref() == Some("sig")
        }));
    }

    #[test]
    fn base_line_targets_use_line_testpoint_property() {
        let source = block(
            "Constant",
            "Source",
            "1",
            vec![port("out", 1, None)],
            None,
            &[],
        );
        let sink = block("Display", "Sink", "2", vec![port("in", 1, None)], None, &[]);
        let mut wire = line("1", 1, "2", 1, None);
        wire.properties
            .insert("TestPoint".to_string(), "on".to_string());
        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![source, sink],
            lines: vec![wire.clone()],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.line_targets_for_line(&[], &wire);

        assert!(targets.iter().any(|target| target.testpoint));
    }

    #[test]
    fn canonical_signal_paths_strip_newlines_and_merge_testpoints() {
        let source = block(
            "Constant",
            "Source\nBlock",
            "1",
            vec![port("out", 1, Some("sig\nname"))],
            None,
            &[],
        );
        let sink = block("Display", "Sink", "2", vec![port("in", 1, None)], None, &[]);
        let wire = line("1", 1, "2", 1, None);
        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![source, sink],
            lines: vec![wire.clone()],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let mut targets = resolver.line_targets_for_line(&[], &wire);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].path, "model/Source Block");
        assert_eq!(targets[0].signal_name, None);

        targets.push(ConnectionTarget {
            path: "model/Source Block".to_string(),
            signal_name: None,
            resolve: None,
            element_index: None,
            origin: ConnectionTargetOrigin::SourceBlock,
            signals_only: true,
            testpoint: true,
        });
        let deduped = super::dedup_targets(targets);
        assert_eq!(deduped.len(), 1);
        assert!(deduped[0].testpoint);
    }

    #[test]
    fn base_line_targets_do_not_invent_signal_names_without_explicit_line_name() {
        let source = block(
            "Constant",
            "Source",
            "1",
            vec![port("out", 1, Some("sig name"))],
            None,
            &[],
        );
        let sink = block("Display", "Sink", "2", vec![port("in", 1, None)], None, &[]);
        let wire = line("1", 1, "2", 1, None);
        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![source, sink],
            lines: vec![wire.clone()],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.line_targets_for_line(&[], &wire);

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].path, "model/Source");
        assert_eq!(targets[0].signal_name, None);
    }

    #[test]
    fn base_line_targets_set_output_index_for_multi_output_source() {
        let source = block(
            "ComplexToRealImag",
            "ComplexToRealImag",
            "1",
            vec![
                port("in", 1, None),
                port("out", 1, None),
                port("out", 2, None),
            ],
            None,
            &[],
        );
        let sink = block("Display", "Sink", "2", vec![port("in", 1, None)], None, &[]);
        let wire = line("1", 2, "2", 1, None);
        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![source, sink],
            lines: vec![wire.clone()],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.line_targets_for_line(&[], &wire);

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].path, "model/ComplexToRealImag");
        assert_eq!(targets[0].element_index, Some(2));
    }

    #[test]
    fn bus_selector_ignores_propagated_signal_fallbacks() {
        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![
                block("Constant", "A", "1", vec![port("out", 1, None)], None, &[]),
                block("Constant", "B", "2", vec![port("out", 1, None)], None, &[]),
                block(
                    "BusCreator",
                    "BusCreator",
                    "3",
                    vec![
                        port("in", 1, None),
                        port("in", 2, None),
                        port("out", 1, None),
                    ],
                    None,
                    &[],
                ),
                block(
                    "BusSelector",
                    "BusSelector",
                    "4",
                    vec![port("in", 1, None), propagated_port("out", 1, "beta")],
                    None,
                    &[],
                ),
                block("Display", "Sink", "5", vec![port("in", 1, None)], None, &[]),
            ],
            lines: vec![
                line("1", 1, "3", 1, Some("alpha")),
                line("2", 1, "3", 2, Some("beta")),
                line("3", 1, "4", 1, None),
                line("4", 1, "5", 1, None),
            ],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.line_targets_for_line(&[], &system.lines[3]);

        assert!(targets.is_empty(), "targets: {targets:?}");
    }

    #[test]
    fn bus_selector_uses_explicit_output_line_name_for_target_paths() {
        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![
                block("Constant", "A", "1", vec![port("out", 1, None)], None, &[]),
                block("Constant", "B", "2", vec![port("out", 1, None)], None, &[]),
                block(
                    "BusCreator",
                    "BusCreator",
                    "3",
                    vec![
                        port("in", 1, None),
                        port("in", 2, None),
                        port("out", 1, None),
                    ],
                    None,
                    &[],
                ),
                block(
                    "BusSelector",
                    "BusSelector",
                    "4",
                    vec![port("in", 1, None), port("out", 1, None)],
                    None,
                    &[],
                ),
                block("Display", "Sink", "5", vec![port("in", 1, None)], None, &[]),
            ],
            lines: vec![
                line("1", 1, "3", 1, Some("alpha")),
                line("2", 1, "3", 2, Some("beta")),
                line("3", 1, "4", 1, None),
                line("4", 1, "5", 1, Some("beta")),
            ],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.line_targets_for_line(&[], &system.lines[3]);

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].path, "model/B");
        assert_eq!(targets[0].signal_name.as_deref(), Some("beta"));
        assert_eq!(targets[0].origin, ConnectionTargetOrigin::BusSelector);
        assert_eq!(
            targets[0].resolve,
            Some(ConnectionTargetResolve::Signal("beta".to_string()))
        );
    }

    #[test]
    fn bus_selector_matches_angled_line_names_against_bus_creator_inputs() {
        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![
                block("Constant", "A", "1", vec![port("out", 1, None)], None, &[]),
                block("Constant", "B", "2", vec![port("out", 1, None)], None, &[]),
                block(
                    "BusCreator",
                    "BusCreator",
                    "3",
                    vec![
                        port("in", 1, None),
                        port("in", 2, None),
                        port("out", 1, None),
                    ],
                    None,
                    &[],
                ),
                block(
                    "BusSelector",
                    "BusSelector",
                    "4",
                    vec![port("in", 1, None), port("out", 1, None)],
                    None,
                    &[],
                ),
                block("Display", "Sink", "5", vec![port("in", 1, None)], None, &[]),
            ],
            lines: vec![
                line("1", 1, "3", 1, Some("a")),
                line("2", 1, "3", 2, Some("signal1")),
                line("3", 1, "4", 1, None),
                line("4", 1, "5", 1, Some("<signal1>")),
            ],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.line_targets_for_line(&[], &system.lines[3]);

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].path, "model/B");
        assert_eq!(
            targets[0].resolve,
            Some(ConnectionTargetResolve::Signal("signal1".to_string()))
        );
    }

    #[test]
    fn demux_propagates_names_and_testpoints_back_upstream_and_strips_indices_after_split() {
        let source = block(
            "Constant",
            "Source",
            "1",
            vec![port("out", 1, None)],
            None,
            &[],
        );
        let mux = block(
            "Mux",
            "Mux",
            "2",
            vec![port("in", 1, None), port("out", 1, None)],
            None,
            &[],
        );
        let demux = block(
            "Demux",
            "Demux",
            "3",
            vec![port("in", 1, None), port("out", 1, None)],
            None,
            &[],
        );
        let sink = block("Display", "Sink", "4", vec![port("in", 1, None)], None, &[]);

        let upstream = line("1", 1, "2", 1, None);
        let middle = line("2", 1, "3", 1, None);
        let mut downstream = line("3", 1, "4", 1, Some("selected"));
        downstream
            .properties
            .insert("TestPoint".to_string(), "on".to_string());

        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![source, mux, demux, sink],
            lines: vec![upstream.clone(), middle, downstream.clone()],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let upstream_targets = resolver.line_targets_for_line(&[], &upstream);
        let downstream_targets = resolver.line_targets_for_line(&[], &downstream);

        assert_eq!(upstream_targets.len(), 1);
        assert_eq!(upstream_targets[0].path, "model/Source");
        assert_eq!(upstream_targets[0].signal_name.as_deref(), Some("selected"));
        assert!(upstream_targets[0].testpoint);

        assert_eq!(downstream_targets.len(), 1);
        assert_eq!(downstream_targets[0].path, "model/Source");
        assert_eq!(
            downstream_targets[0].signal_name.as_deref(),
            Some("selected")
        );
        assert_eq!(downstream_targets[0].origin, ConnectionTargetOrigin::Demux);
        assert_eq!(downstream_targets[0].element_index, None);
        assert!(downstream_targets[0].testpoint);
    }

    #[test]
    fn upstream_propagation_preserves_explicit_local_line_names() {
        let source = block(
            "Constant",
            "Source",
            "1",
            vec![port("out", 1, None)],
            None,
            &[],
        );
        let mux = block(
            "Mux",
            "Mux",
            "2",
            vec![port("in", 1, None), port("out", 1, None)],
            None,
            &[],
        );
        let demux = block(
            "Demux",
            "Demux",
            "3",
            vec![port("in", 1, None), port("out", 1, None)],
            None,
            &[],
        );
        let sink = block("Display", "Sink", "4", vec![port("in", 1, None)], None, &[]);

        let upstream = line("1", 1, "2", 1, Some("local"));
        let middle = line("2", 1, "3", 1, None);
        let downstream = line("3", 1, "4", 1, Some("remote"));

        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![source, mux, demux, sink],
            lines: vec![upstream.clone(), middle, downstream],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let upstream_targets = resolver.line_targets_for_line(&[], &upstream);

        assert_eq!(upstream_targets.len(), 1);
        assert_eq!(upstream_targets[0].signal_name.as_deref(), Some("local"));
    }

    #[test]
    fn subsystem_input_line_receives_child_metadata_upstream() {
        let mut child_wire = line("10", 1, "11", 1, Some("child_name"));
        child_wire
            .properties
            .insert("TestPoint".to_string(), "on".to_string());
        let child_system = System {
            properties: IndexMap::new(),
            blocks: vec![
                block(
                    "Inport",
                    "In1",
                    "10",
                    vec![port("out", 1, None)],
                    None,
                    &[("Port", "1")],
                ),
                block(
                    "Outport",
                    "Out1",
                    "11",
                    vec![port("in", 1, None)],
                    None,
                    &[("Port", "1")],
                ),
            ],
            lines: vec![child_wire],
            annotations: Vec::new(),
            chart: None,
        };

        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![
                block(
                    "Constant",
                    "Src",
                    "1",
                    vec![port("out", 1, None)],
                    None,
                    &[],
                ),
                block(
                    "SubSystem",
                    "Sub",
                    "2",
                    vec![port("in", 1, None), port("out", 1, None)],
                    Some(child_system),
                    &[],
                ),
            ],
            lines: vec![line("1", 1, "2", 1, None)],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.line_targets_for_line(&[], &system.lines[0]);

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].signal_name.as_deref(), Some("child_name"));
        assert!(targets[0].testpoint);
    }

    #[test]
    fn subsystem_output_metadata_flows_back_into_child_outport_line() {
        let child_system = System {
            properties: IndexMap::new(),
            blocks: vec![
                block(
                    "Inport",
                    "In1",
                    "10",
                    vec![port("out", 1, None)],
                    None,
                    &[("Port", "1")],
                ),
                block(
                    "Outport",
                    "Out1",
                    "11",
                    vec![port("in", 1, None)],
                    None,
                    &[("Port", "1")],
                ),
            ],
            lines: vec![line("10", 1, "11", 1, None)],
            annotations: Vec::new(),
            chart: None,
        };

        let mut parent_output = line("2", 1, "3", 1, Some("outer_name"));
        parent_output
            .properties
            .insert("TestPoint".to_string(), "on".to_string());
        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![
                block(
                    "SubSystem",
                    "Sub",
                    "2",
                    vec![port("in", 1, None), port("out", 1, None)],
                    Some(child_system),
                    &[],
                ),
                block("Display", "Sink", "3", vec![port("in", 1, None)], None, &[]),
            ],
            lines: vec![parent_output],
            annotations: Vec::new(),
            chart: None,
        };

        let child_line = system.blocks[0]
            .subsystem
            .as_ref()
            .and_then(|child| child.lines.first())
            .cloned()
            .expect("child line");
        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.line_targets_for_line(&["Sub".to_string()], &child_line);

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].signal_name.as_deref(), Some("outer_name"));
        assert!(targets[0].testpoint);
    }

    #[test]
    fn subsystem_named_child_line_keeps_name_and_inherits_outer_testpoint() {
        let child_system = System {
            properties: IndexMap::new(),
            blocks: vec![
                block(
                    "Inport",
                    "In1",
                    "10",
                    vec![port("out", 1, None)],
                    None,
                    &[("Port", "1")],
                ),
                block(
                    "Outport",
                    "Out1",
                    "11",
                    vec![port("in", 1, None)],
                    None,
                    &[("Port", "1")],
                ),
            ],
            lines: vec![line("10", 1, "11", 1, Some("inner_name"))],
            annotations: Vec::new(),
            chart: None,
        };

        let mut parent_output = line("2", 1, "3", 1, Some("outer_name"));
        parent_output
            .properties
            .insert("TestPoint".to_string(), "on".to_string());
        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![
                block(
                    "SubSystem",
                    "Sub",
                    "2",
                    vec![port("in", 1, None), port("out", 1, None)],
                    Some(child_system),
                    &[],
                ),
                block("Display", "Sink", "3", vec![port("in", 1, None)], None, &[]),
            ],
            lines: vec![parent_output],
            annotations: Vec::new(),
            chart: None,
        };

        let child_line = system.blocks[0]
            .subsystem
            .as_ref()
            .and_then(|child| child.lines.first())
            .cloned()
            .expect("child line");
        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.line_targets_for_line(&["Sub".to_string()], &child_line);

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].signal_name.as_deref(), Some("inner_name"));
        assert!(targets[0].testpoint);
    }

    #[test]
    fn subsystem_child_line_inherits_outer_testpoint_when_outer_line_is_unnamed() {
        let child_system = System {
            properties: IndexMap::new(),
            blocks: vec![
                block(
                    "Inport",
                    "In1",
                    "10",
                    vec![port("out", 1, None)],
                    None,
                    &[("Port", "1")],
                ),
                block(
                    "Outport",
                    "Out1",
                    "11",
                    vec![port("in", 1, None)],
                    None,
                    &[("Port", "1")],
                ),
            ],
            lines: vec![line("10", 1, "11", 1, Some("inner_name"))],
            annotations: Vec::new(),
            chart: None,
        };

        let mut parent_output = line("2", 1, "3", 1, None);
        parent_output
            .properties
            .insert("TestPoint".to_string(), "on".to_string());
        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![
                block(
                    "SubSystem",
                    "Sub",
                    "2",
                    vec![port("in", 1, None), port("out", 1, None)],
                    Some(child_system),
                    &[],
                ),
                block("Display", "Sink", "3", vec![port("in", 1, None)], None, &[]),
            ],
            lines: vec![parent_output],
            annotations: Vec::new(),
            chart: None,
        };

        let child_line = system.blocks[0]
            .subsystem
            .as_ref()
            .and_then(|child| child.lines.first())
            .cloned()
            .expect("child line");
        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.line_targets_for_line(&["Sub".to_string()], &child_line);

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].signal_name.as_deref(), Some("inner_name"));
        assert!(targets[0].testpoint);
    }

    #[test]
    fn bus_selector_uses_default_signal_name_for_unnamed_bus_input() {
        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![
                block("Constant", "A", "1", vec![port("out", 1, None)], None, &[]),
                block(
                    "BusCreator",
                    "BusCreator",
                    "2",
                    vec![port("in", 1, None), port("out", 1, None)],
                    None,
                    &[],
                ),
                block(
                    "BusSelector",
                    "BusSelector",
                    "3",
                    vec![port("in", 1, None), port("out", 1, None)],
                    None,
                    &[],
                ),
                block("Display", "Sink", "4", vec![port("in", 1, None)], None, &[]),
            ],
            lines: vec![
                line("1", 1, "2", 1, None),
                line("2", 1, "3", 1, None),
                line("3", 1, "4", 1, None),
            ],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.line_targets_for_line(&[], &system.lines[2]);

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].path, "model/A");
        assert_eq!(
            targets[0].resolve,
            Some(ConnectionTargetResolve::Signal("signal1".to_string()))
        );
    }

    #[test]
    fn bus_selector_output_port_testpoint_is_visible_on_output_line() {
        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![
                block("Constant", "A", "1", vec![port("out", 1, None)], None, &[]),
                block(
                    "BusCreator",
                    "BusCreator",
                    "2",
                    vec![port("in", 1, None), port("out", 1, None)],
                    None,
                    &[],
                ),
                block(
                    "BusSelector",
                    "BusSelector",
                    "3",
                    vec![
                        port("in", 1, None),
                        testpoint_port("out", 1, Some("signal1")),
                    ],
                    None,
                    &[],
                ),
                block("Display", "Sink", "4", vec![port("in", 1, None)], None, &[]),
            ],
            lines: vec![
                line("1", 1, "2", 1, Some("signal1")),
                line("2", 1, "3", 1, None),
                line("3", 1, "4", 1, None),
            ],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.line_targets_for_line(&[], &system.lines[2]);

        assert_eq!(targets.len(), 1);
        assert!(targets[0].testpoint);
        assert_eq!(targets[0].origin, ConnectionTargetOrigin::BusSelector);
    }

    #[test]
    fn bus_selector_output_testpoint_propagates_back_to_bus_creator_inputs() {
        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![
                block("Constant", "A", "1", vec![port("out", 1, None)], None, &[]),
                block(
                    "BusCreator",
                    "BusCreator",
                    "2",
                    vec![port("in", 1, None), port("out", 1, None)],
                    None,
                    &[],
                ),
                block(
                    "BusSelector",
                    "BusSelector",
                    "3",
                    vec![
                        port("in", 1, None),
                        testpoint_port("out", 1, Some("signal1")),
                    ],
                    None,
                    &[],
                ),
                block("Display", "Sink", "4", vec![port("in", 1, None)], None, &[]),
            ],
            lines: vec![
                line("1", 1, "2", 1, None),
                line("2", 1, "3", 1, None),
                line("3", 1, "4", 1, None),
            ],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let upstream_targets = resolver.line_targets_for_line(&[], &system.lines[0]);
        let downstream_targets = resolver.line_targets_for_line(&[], &system.lines[2]);

        assert_eq!(upstream_targets.len(), 1);
        assert!(upstream_targets[0].testpoint);
        assert_eq!(downstream_targets.len(), 1);
        assert!(downstream_targets[0].testpoint);
    }

    #[test]
    fn demux_output_port_testpoint_propagates_back_to_mux_input() {
        let source = block(
            "Constant",
            "Source",
            "1",
            vec![port("out", 1, None)],
            None,
            &[],
        );
        let mux = block(
            "Mux",
            "Mux",
            "2",
            vec![port("in", 1, None), port("out", 1, None)],
            None,
            &[],
        );
        let demux = block(
            "Demux",
            "Demux",
            "3",
            vec![port("in", 1, None), testpoint_port("out", 1, None)],
            None,
            &[],
        );
        let sink = block("Display", "Sink", "4", vec![port("in", 1, None)], None, &[]);

        let upstream = line("1", 1, "2", 1, None);
        let middle = line("2", 1, "3", 1, None);
        let downstream = line("3", 1, "4", 1, None);

        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![source, mux, demux, sink],
            lines: vec![upstream.clone(), middle, downstream.clone()],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let upstream_targets = resolver.line_targets_for_line(&[], &upstream);
        let downstream_targets = resolver.line_targets_for_line(&[], &downstream);

        assert!(upstream_targets[0].testpoint);
        assert!(downstream_targets[0].testpoint);
        assert_eq!(downstream_targets[0].origin, ConnectionTargetOrigin::Demux);
    }

    #[test]
    fn mux_targets_use_resolve_without_element_index() {
        let system = System {
            properties: props(&[("Name", "model")]),
            blocks: vec![
                block("Constant", "A", "1", vec![port("out", 1, None)], None, &[]),
                block(
                    "Mux",
                    "Mux",
                    "2",
                    vec![port("in", 1, None), port("out", 1, None)],
                    None,
                    &[],
                ),
                block("Display", "Sink", "3", vec![port("in", 1, None)], None, &[]),
            ],
            lines: vec![line("1", 1, "2", 1, None), line("2", 1, "3", 1, None)],
            annotations: Vec::new(),
            chart: None,
        };

        let resolver = ConnectionTargetResolver::new(&system);
        let targets = resolver.line_targets_for_line(&[], &system.lines[1]);

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].resolve, Some(ConnectionTargetResolve::Index(1)));
        assert_eq!(targets[0].element_index, None);
        assert_eq!(targets[0].origin, ConnectionTargetOrigin::Mux);
    }

    fn block(
        block_type: &str,
        name: &str,
        sid: &str,
        ports: Vec<Port>,
        subsystem: Option<System>,
        properties: &[(&str, &str)],
    ) -> Block {
        Block {
            block_type: block_type.to_string(),
            name: name.to_string(),
            sid: Some(sid.to_string()),
            tag_name: "Block".to_string(),
            position: None,
            zorder: None,
            commented: false,
            name_location: NameLocation::Bottom,
            is_matlab_function: false,
            value: None,
            value_kind: ValueKind::default(),
            value_rows: None,
            value_cols: None,
            properties: props(properties),
            ref_properties: Default::default(),
            port_counts: None,
            ports,
            subsystem: subsystem.map(Box::new),
            system_ref: None,
            c_function: None,
            instance_data: None,
            link_data: None,
            mask: None,
            annotations: Vec::new(),
            background_color: None,
            show_name: None,
            font_size: None,
            font_weight: None,
            mask_display_text: None,
            current_setting: None,
            block_mirror: None,
            library_source: None,
            library_block_path: None,
            dashboard_binding: None,
            child_order: Vec::new(),
        }
    }

    fn port(port_type: &str, index: u32, name: Option<&str>) -> Port {
        let mut properties = IndexMap::new();
        if let Some(name) = name {
            properties.insert("Name".to_string(), name.to_string());
        }
        Port {
            port_type: port_type.to_string(),
            index: Some(index),
            properties,
        }
    }

    fn propagated_port(port_type: &str, index: u32, propagated_signal: &str) -> Port {
        Port {
            port_type: port_type.to_string(),
            index: Some(index),
            properties: IndexMap::from_iter([(
                "PropagatedSignals".to_string(),
                propagated_signal.to_string(),
            )]),
        }
    }

    fn testpoint_port(port_type: &str, index: u32, name: Option<&str>) -> Port {
        let mut port = port(port_type, index, name);
        port.properties
            .insert("TestPoint".to_string(), "on".to_string());
        port
    }

    fn line(
        src_sid: &str,
        src_port: u32,
        dst_sid: &str,
        dst_port: u32,
        name: Option<&str>,
    ) -> Line {
        Line {
            name: name.map(str::to_string),
            zorder: None,
            src: Some(endpoint(src_sid, "out", src_port)),
            dst: Some(endpoint(dst_sid, "in", dst_port)),
            points: vec![Point { x: 0, y: 0 }, Point { x: 10, y: 0 }],
            labels: None,
            branches: Vec::new(),
            properties: IndexMap::new(),
        }
    }

    fn endpoint(sid: &str, port_type: &str, port_index: u32) -> EndpointRef {
        EndpointRef {
            sid: sid.to_string(),
            port_type: port_type.to_string(),
            port_index,
        }
    }

    fn props(entries: &[(&str, &str)]) -> IndexMap<String, String> {
        let mut properties = IndexMap::new();
        for (key, value) in entries {
            properties.insert((*key).to_string(), (*value).to_string());
        }
        properties
    }
}
