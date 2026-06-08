//! Bridge that imports the built-in virtual libraries (matrix, discrete, logic
//! & bit ops, math operations, signal routing, …) into the unified catalog.
//!
//! These libraries carry SVG icons, shapes, port counts and per-instance label
//! functions.  Rather than rewriting all of them by hand, this module adapts
//! each `VirtualBlock` into a [`SimulinkBlockDefinition`] at registry-build
//! time.  The definitions are leaked once (process lifetime) so they can live
//! in the global registry with a `'static` lifetime and zero per-frame cost.
//!
//! The built-in library modules are retained for their parser role (generating
//! stub subsystems for libraries that are not present on disk); this bridge
//! consumes only their rendering metadata.

#![cfg(feature = "egui")]

use crate::builtin_libraries::virtual_library::{
    BlockShape, PortPlacement as CfgPlacement, PortPositionOverride as CfgOverride, VirtualBlock,
};

use super::types::{
    BlockLabelPolicy, IOPorts, PortLabelPolicy, PortPlacement, PortPositionOverride,
    SimulinkBlockDefinition, SimulinkIcon, SimulinkShape,
};

fn map_placement(p: CfgPlacement) -> PortPlacement {
    match p {
        CfgPlacement::Left => PortPlacement::Left,
        CfgPlacement::Right => PortPlacement::Right,
        CfgPlacement::Top => PortPlacement::Top,
        CfgPlacement::Bottom => PortPlacement::Bottom,
    }
}

fn map_overrides(overrides: &[CfgOverride]) -> &'static [PortPositionOverride] {
    if overrides.is_empty() {
        return &[];
    }
    let mapped: Vec<PortPositionOverride> = overrides
        .iter()
        .map(|o| PortPositionOverride {
            is_input: o.is_input,
            port_index: o.port_index,
            placement: map_placement(o.placement),
            fraction: o.fraction,
        })
        .collect();
    Box::leak(mapped.into_boxed_slice())
}

fn map_shape(shape: BlockShape) -> SimulinkShape {
    match shape {
        BlockShape::Rectangle => SimulinkShape::Rectangle,
        BlockShape::Triangle => SimulinkShape::Triangle,
        BlockShape::Circle => SimulinkShape::Circle,
        BlockShape::FilledBlack => SimulinkShape::FilledBlack,
        BlockShape::Goto => SimulinkShape::Goto,
        BlockShape::From => SimulinkShape::From,
    }
}

fn port_label_policy(names: &'static [&'static str]) -> PortLabelPolicy {
    if names.is_empty() {
        PortLabelPolicy::None
    } else {
        PortLabelPolicy::Fixed(names)
    }
}

/// Convert one built-in `VirtualBlock` into a leaked, `'static` definition.
fn definition_from(vb: &VirtualBlock, lib_name: &'static str) -> &'static SimulinkBlockDefinition {
    let mut def = SimulinkBlockDefinition::new(vb.name, lib_name)
        .with_aliases(vb.aliases)
        .with_ports(IOPorts::Variable(vb.ins), IOPorts::Variable(vb.outs))
        .with_shape(map_shape(vb.shape))
        .with_port_labels(
            port_label_policy(vb.input_port_names),
            port_label_policy(vb.output_port_names),
        )
        .with_port_overrides(map_overrides(vb.port_position_overrides));
    if let Some(path) = vb.icon {
        def = def.with_icon(SimulinkIcon::Svg(path));
    }
    if let Some(f) = vb.compute_instance_label {
        def = def.with_instance_label(f);
        def.block_label = BlockLabelPolicy::None;
    }
    Box::leak(Box::new(def))
}

/// Build all bridged definitions from the built-in virtual libraries.
pub fn bridged_definitions() -> Vec<&'static SimulinkBlockDefinition> {
    let mut out = Vec::new();
    for lib in crate::builtin_libraries::VIRTUAL_LIBRARIES {
        for vb in (lib.get_blocks)() {
            out.push(definition_from(vb, lib.name));
        }
    }
    out
}
