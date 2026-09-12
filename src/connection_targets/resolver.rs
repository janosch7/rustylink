//! The resolver itself: system traversal, caching and line propagation.

use super::graph::{
    boundary_port_index, build_block_lookup, child_incoming_targets_by_port,
    child_outgoing_targets_by_port, child_system_path, control_incoming_targets,
    dashboard_binding_block_path, dashboard_binding_target_path, incoming_line_targets_for_block,
    incoming_line_targets_for_block_on_port, incoming_lines_for_block, incoming_targets_by_port,
    line_destination_endpoints, normalized_path_segment, outgoing_targets_by_port,
    output_port_count, qualify_external_path,
};
use super::metadata::{
    apply_line_resolve_hint, apply_local_line_metadata, apply_source_port_testpoint,
    line_testpoint, merge_upstream_metadata, port_testpoint, routing_line_signal_name,
    set_signal_name_only,
};
use super::target::{
    ConnectionTarget, ConnectionTargetOrigin, ConnectionTargetResolve, dedup_targets,
};
use super::topology::{block_cache_key, line_cache_key, same_line};
use super::variants::{active_variant_child_sid, active_variant_port_index, is_variant_system};
use crate::model::{Block, DashboardBinding, Line, System};
use crate::simulink_libraries::traits::{SignalRole, block_traits};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

/// How many times the root-level resolution loop re-resolves the entire
/// tree, waiting for cross-boundary signal targets to settle.  Each pass
/// propagates information one subsystem level, so 32 passes handles models
/// up to ~16 levels deep with multi-hop sibling chains.
pub const MAX_GLOBAL_RESOLVE_PASSES: usize = 32;

/// Tracks progress of `ConnectionTargetResolver` construction for background
/// builds.  `tick()` is called on every `resolve_system` visit; the shared
/// atomics let the UI thread read live progress without locking.
struct ProgressTracker {
    /// Total `resolve_system` calls across all root-level passes.
    counter: AtomicUsize,
    /// Shared with the UI thread: 0..=1000 (0.0%..=100.0% for the bar fill).
    progress: Arc<AtomicU32>,
    /// Shared with the UI thread: total subsystems visited so far.
    visited: Arc<AtomicUsize>,
    /// Estimated total work: `estimated_passes × total_subsystems`.
    total: usize,
}

impl ProgressTracker {
    fn new(progress: Arc<AtomicU32>, visited: Arc<AtomicUsize>, total: usize) -> Self {
        Self {
            counter: AtomicUsize::new(0),
            progress,
            visited,
            total,
        }
    }
    fn tick(&self) {
        let count = self.counter.fetch_add(1, Ordering::Relaxed) + 1;
        self.visited.store(count, Ordering::Relaxed);
        let p = ((count as u32 * 1000) / self.total.max(1) as u32).min(1000);
        self.progress.store(p, Ordering::Relaxed);
    }
}

/// Count the total number of `System` nodes in the tree (root + all nested
/// subsystems).  Used to estimate progress for the background build.
pub fn count_subsystems(system: &System) -> usize {
    1 + system
        .blocks
        .iter()
        .filter_map(|b| b.subsystem.as_ref())
        .map(|sub| count_subsystems(sub))
        .sum::<usize>()
}

/// Maximum nesting depth of subsystems in the tree (root = 1).
pub fn max_subsystem_depth(system: &System) -> usize {
    1 + system
        .blocks
        .iter()
        .filter_map(|b| b.subsystem.as_ref())
        .map(|sub| max_subsystem_depth(sub))
        .max()
        .unwrap_or(0)
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct ParentSubsystemContext {
    pub(crate) incoming_by_port: BTreeMap<u32, Vec<ConnectionTarget>>,
    pub(crate) outgoing_by_port: BTreeMap<u32, Vec<ConnectionTarget>>,
    /// Targets from lines connecting to the subsystem's control ports
    /// (enable/trigger/reset).  Used by EnablePort/TriggerPort/ResetPort
    /// blocks that expose an output port (`ShowOutputPort=on`).
    pub(crate) control_incoming_targets: Vec<ConnectionTarget>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ChildSubsystemSummary {
    pub(crate) incoming_by_port: BTreeMap<u32, Vec<ConnectionTarget>>,
    pub(crate) outgoing_by_port: BTreeMap<u32, Vec<ConnectionTarget>>,
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionTargetResolver {
    pub(crate) block_targets: HashMap<String, Vec<ConnectionTarget>>,
    pub(crate) line_targets: HashMap<String, Vec<ConnectionTarget>>,
    pub(crate) model_name: String,
    /// Child subsystem summaries keyed by block SID, persisted across root-level
    /// resolution passes so that information from a previous pass can seed the
    /// next one without re-resolving every child at every recursion level.
    pub(crate) child_summaries: HashMap<String, ChildSubsystemSummary>,
}

impl ConnectionTargetResolver {
    pub fn new(root: &System) -> Self {
        let mut resolver = Self {
            block_targets: HashMap::new(),
            line_targets: HashMap::new(),
            model_name: root.properties.get("Name").cloned().unwrap_or_default(),
            child_summaries: HashMap::new(),
        };
        let empty_path: Vec<String> = Vec::new();
        // Re-resolve the entire tree until the cached targets stop changing.
        // Each pass propagates cross-boundary signal information one subsystem
        // level, so this converges in O(depth) passes instead of the
        // exponential 8^depth that a per-recursion-level loop would cost.
        for _ in 0..MAX_GLOBAL_RESOLVE_PASSES {
            let prev_block = resolver.block_targets.clone();
            let prev_line = resolver.line_targets.clone();
            resolver.resolve_system(root, &empty_path, None, None, None);
            if resolver.block_targets == prev_block && resolver.line_targets == prev_line {
                break;
            }
        }
        resolver
    }

    /// Like [`new`](Self::new) but reports build progress through shared atomics.
    ///
    /// `progress` is written as 0..=1000 (0.0%..=100.0%) and `visited` as the
    /// total number of `resolve_system` calls so far.  Both use
    /// `Ordering::Relaxed` so the UI thread can read them without locking.
    pub fn new_with_progress(
        root: &System,
        progress: Arc<AtomicU32>,
        visited: Arc<AtomicUsize>,
    ) -> Self {
        let total_subsystems = count_subsystems(root).max(1);
        let max_depth = max_subsystem_depth(root);
        let estimated_passes = MAX_GLOBAL_RESOLVE_PASSES.min(max_depth + 2).max(1);
        let total_work = estimated_passes * total_subsystems;
        let tracker = ProgressTracker::new(progress, visited, total_work);

        let mut resolver = Self {
            block_targets: HashMap::new(),
            line_targets: HashMap::new(),
            model_name: root.properties.get("Name").cloned().unwrap_or_default(),
            child_summaries: HashMap::new(),
        };
        let empty_path: Vec<String> = Vec::new();
        for _ in 0..MAX_GLOBAL_RESOLVE_PASSES {
            let prev_block = resolver.block_targets.clone();
            let prev_line = resolver.line_targets.clone();
            resolver.resolve_system(root, &empty_path, None, None, Some(&tracker));
            if resolver.block_targets == prev_block && resolver.line_targets == prev_line {
                break;
            }
        }
        // Ensure progress shows 100% when done.
        tracker.progress.store(1000, Ordering::Relaxed);
        tracker.visited.store(total_work, Ordering::Relaxed);
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

    /// Like `block_targets_for_block` but returns a reference to avoid cloning.
    pub fn block_targets_for_block_ref(
        &self,
        system_path: &[String],
        block: &Block,
    ) -> &[ConnectionTarget] {
        let key = block_cache_key(system_path, block);
        self.block_targets
            .get(&key)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn line_targets_for_line(
        &self,
        system_path: &[String],
        line: &Line,
    ) -> Vec<ConnectionTarget> {
        let key = line_cache_key(system_path, line);
        self.line_targets.get(&key).cloned().unwrap_or_default()
    }

    /// Like `line_targets_for_line` but returns a reference to avoid cloning.
    pub fn line_targets_for_line_ref(
        &self,
        system_path: &[String],
        line: &Line,
    ) -> &[ConnectionTarget] {
        let key = line_cache_key(system_path, line);
        self.line_targets
            .get(&key)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn resolve_system(
        &mut self,
        system: &System,
        system_path: &[String],
        parent_ctx: Option<&ParentSubsystemContext>,
        parent_block: Option<&Block>,
        progress: Option<&ProgressTracker>,
    ) -> ChildSubsystemSummary {
        if let Some(p) = progress {
            p.tick();
        }
        let block_lookup = build_block_lookup(system);
        let mut line_targets: Vec<Vec<ConnectionTarget>> = system
            .lines
            .iter()
            .map(|line| self.base_line_targets(system, system_path, &block_lookup, line))
            .collect();

        // Initial propagation with child summaries from the previous root-level
        // pass.  On the first pass this is empty, so lines fall back to their
        // base targets; subsequent passes see the summaries computed below.
        self.propagate_line_targets(
            system,
            system_path,
            &block_lookup,
            parent_ctx,
            &self.child_summaries,
            &mut line_targets,
        );
        self.propagate_line_metadata_upward(
            system,
            system_path,
            &block_lookup,
            parent_ctx,
            &self.child_summaries,
            &mut line_targets,
        );

        // Resolve each child subsystem once.  The root-level loop in `new`
        // re-invokes `resolve_system` until the cached targets converge, which
        // is what lets cross-boundary signal information propagate across
        // sibling subsystems without an exponential per-level loop here.
        for block in &system.blocks {
            if let Some(subsystem) = &block.subsystem {
                // Variant children (those with a `VariantControl` property)
                // implicitly connect to the variant subsystem's ports by port
                // index, so they inherit the variant subsystem's own parent
                // context rather than building one from internal lines.
                let is_variant_child = block.properties.contains_key("VariantControl");
                let child_ctx = if is_variant_child {
                    parent_ctx.cloned().unwrap_or_default()
                } else {
                    ParentSubsystemContext {
                        incoming_by_port: incoming_targets_by_port(system, block, &line_targets),
                        outgoing_by_port: outgoing_targets_by_port(system, block, &line_targets),
                        control_incoming_targets: control_incoming_targets(
                            system,
                            block,
                            &line_targets,
                        ),
                    }
                };
                let child_path = child_system_path(system_path, &block.name);
                let summary = self.resolve_system(
                    subsystem,
                    &child_path,
                    Some(&child_ctx),
                    Some(block),
                    progress,
                );
                if let Some(sid) = &block.sid {
                    self.child_summaries.insert(sid.clone(), summary);
                }
            }
        }

        // Final propagation with the freshly computed child summaries.
        self.propagate_line_targets(
            system,
            system_path,
            &block_lookup,
            parent_ctx,
            &self.child_summaries,
            &mut line_targets,
        );
        self.propagate_line_metadata_upward(
            system,
            system_path,
            &block_lookup,
            parent_ctx,
            &self.child_summaries,
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

            // Tag every target that belongs to this block with its block type so
            // downstream matchers (e.g. ASX Simulink plugin) can decide whether
            // to use exact or prefix path matching.
            let block_type = block.block_type.clone();
            for target in &mut targets {
                target.block_type = Some(block_type.clone());
            }

            if let Some(binding) = &block.dashboard_binding {
                let target_path =
                    qualify_external_path(&self.model_name, dashboard_binding_block_path(binding));
                let mut target =
                    ConnectionTarget::new(target_path, ConnectionTargetOrigin::DashboardBinding);
                target.block_type = Some(block.block_type.clone());
                if let DashboardBinding::SignalSpec { signal_name, .. } = binding {
                    set_signal_name_only(&mut target, Some(signal_name.clone()));
                }
                target.signals_only = matches!(binding, DashboardBinding::SignalSpec { .. });
                if let DashboardBinding::SignalSpec { target_path, .. } = binding {
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

        // Variant subsystems have no internal lines.  Their child summary is
        // built by merging the summaries of the active (or all) variant
        // children instead of from Inport/Outport blocks.
        if is_variant_system(system) {
            let active = active_variant_child_sid(system, parent_block);
            let mut incoming: BTreeMap<u32, Vec<ConnectionTarget>> = BTreeMap::new();
            let mut outgoing: BTreeMap<u32, Vec<ConnectionTarget>> = BTreeMap::new();
            for block in &system.blocks {
                if !block.properties.contains_key("VariantControl") {
                    continue;
                }
                let sid = block.sid.as_deref();
                let is_active = active.as_deref() == sid;
                if (active.is_none() || is_active)
                    && let Some(summary) = sid.and_then(|s| self.child_summaries.get(s))
                {
                    for (port, targets) in &summary.incoming_by_port {
                        incoming.entry(*port).or_default().extend(targets.clone());
                    }
                    for (port, targets) in &summary.outgoing_by_port {
                        outgoing.entry(*port).or_default().extend(targets.clone());
                    }
                }
            }
            for targets in incoming.values_mut() {
                *targets = dedup_targets(std::mem::take(targets));
            }
            for targets in outgoing.values_mut() {
                *targets = dedup_targets(std::mem::take(targets));
            }
            return ChildSubsystemSummary {
                incoming_by_port: incoming,
                outgoing_by_port: outgoing,
            };
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

                let traits = block_traits(&block.block_type);
                let mut new_targets = match traits.signal_role {
                    SignalRole::BusCreator => {
                        self.bus_creator_targets(system, system_path, block, line, line_targets)
                    }
                    SignalRole::BusSelector => {
                        self.bus_selector_targets(system, block, line, line_targets)
                    }
                    SignalRole::BusAssignment => {
                        self.bus_assignment_targets(system, block, line, line_targets)
                    }
                    SignalRole::Mux => self.mux_targets(system, block, line_targets),
                    SignalRole::Demux => {
                        self.demux_targets(system, block, src.port_index, line_targets)
                    }
                    SignalRole::BoundaryInput => parent_ctx
                        .and_then(|ctx| ctx.incoming_by_port.get(&boundary_port_index(block)))
                        .cloned()
                        .unwrap_or_else(|| {
                            self.base_line_targets(system, system_path, block_lookup, line)
                        }),
                    SignalRole::Container => child_summaries
                        .get(&src.sid)
                        .and_then(|summary| summary.outgoing_by_port.get(&src.port_index))
                        .map(|targets| {
                            let mut propagated = targets.clone();
                            // A block whose paths are prefix-matched tags the
                            // propagated targets, so downstream matchers know
                            // not to compare the paths exactly.
                            if traits.path_prefix_matched {
                                for t in &mut propagated {
                                    t.block_type = Some(block.block_type.clone());
                                }
                            }
                            propagated
                        })
                        .unwrap_or_else(|| {
                            self.base_line_targets(system, system_path, block_lookup, line)
                        }),
                    SignalRole::From => {
                        self.resolve_from_block_targets(system, block, line_targets)
                    }
                    SignalRole::ControlPort => parent_ctx
                        .and_then(|ctx| {
                            if ctx.control_incoming_targets.is_empty() {
                                None
                            } else {
                                Some(ctx.control_incoming_targets.clone())
                            }
                        })
                        .unwrap_or_else(|| {
                            self.base_line_targets(system, system_path, block_lookup, line)
                        }),
                    SignalRole::VariantSelect => {
                        let input_targets =
                            incoming_line_targets_for_block(system, block, line_targets);
                        if let Some(active) = active_variant_port_index(block) {
                            if src.port_index == active {
                                input_targets
                            } else {
                                Vec::new()
                            }
                        } else {
                            input_targets
                        }
                    }
                    SignalRole::VariantMerge => {
                        if let Some(active) = active_variant_port_index(block) {
                            incoming_line_targets_for_block_on_port(
                                system,
                                block,
                                active,
                                line_targets,
                            )
                        } else {
                            incoming_line_targets_for_block(system, block, line_targets)
                        }
                    }
                    SignalRole::Plain | SignalRole::Goto | SignalRole::BoundaryOutput => {
                        self.base_line_targets(system, system_path, block_lookup, line)
                    }
                };

                if traits.signal_role.propagates_local_metadata() {
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
        system_path: &[String],
        block_lookup: &HashMap<&str, &Block>,
        parent_ctx: Option<&ParentSubsystemContext>,
        child_summaries: &HashMap<String, ChildSubsystemSummary>,
        line_targets: &mut [Vec<ConnectionTarget>],
    ) {
        for _ in 0..8 {
            let mut changed = false;

            for (index, line) in system.lines.iter().enumerate() {
                // A branched line ends at several blocks at once, and each of
                // them can hand metadata back upstream.
                let mut propagated = Vec::new();
                let mut crosses_boundary = false;
                for dst in line_destination_endpoints(line) {
                    let Some(block) = block_lookup.get(dst.sid.as_str()).copied() else {
                        continue;
                    };
                    propagated.extend(self.upstream_propagated_targets(
                        system,
                        system_path,
                        block,
                        dst,
                        parent_ctx,
                        child_summaries,
                        line_targets,
                    ));
                    crosses_boundary |= block_traits(&block.block_type)
                        .signal_role
                        .crosses_system_boundary();
                }
                if propagated.is_empty() {
                    continue;
                }

                let merged = merge_upstream_metadata(
                    line,
                    &line_targets[index],
                    &propagated,
                    crosses_boundary,
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

    pub(crate) fn base_line_targets(
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
        target.block_type = Some(block.block_type.clone());
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

    pub(crate) fn full_block_path(&self, system_path: &[String], block_name: &str) -> String {
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
                let mut target = ConnectionTarget::new(
                    self.full_block_path(&child_path, &child.name),
                    ConnectionTargetOrigin::Internal,
                );
                target.block_type = Some(child.block_type.clone());
                target
            })
            .collect()
    }
}
