//! Core Simulink blocks: sources, sinks, ports, routing primitives and the
//! common math blocks that carry custom icons, shapes or renderers.
//!
//! These definitions previously lived as hardcoded entries in `block_types.rs`
//! and as interior-renderer registrations in `egui_app::render`.  They are now
//! data in the single catalog.

#![cfg(feature = "egui")]

use crate::simulink_libraries::labels;
use crate::simulink_libraries::renderers;
use crate::simulink_libraries::types::{
    BlockLabelPolicy, IOPorts, PortLabelPolicy, PortPlacement, PortPositionOverride,
    SimulinkBlockDefinition, SimulinkIcon, SimulinkShape,
};

const fn icon(glyph: &'static str) -> SimulinkIcon {
    SimulinkIcon::Utf8(glyph)
}

/// Place the Sum block's second input at the bottom (classic Simulink layout).
const SUM_PORT_OVERRIDES: &[PortPositionOverride] = &[PortPositionOverride {
    is_input: true,
    port_index: 2,
    placement: PortPlacement::Bottom,
    fraction: 0.5,
}];

pub static BLOCKS: &[SimulinkBlockDefinition] = &[
    // ── Math operations ────────────────────────────────────────────────
    SimulinkBlockDefinition::new("Product", "Math Operations")
        .with_description("Multiply or divide inputs")
        .with_ports(IOPorts::Variable(2), IOPorts::Fixed(1))
        .with_icon(icon("×")),
    SimulinkBlockDefinition::new("Sum", "Math Operations")
        .with_description("Add or subtract inputs")
        .with_ports(IOPorts::Variable(2), IOPorts::Fixed(1))
        .with_shape(SimulinkShape::Circle)
        .with_port_overrides(SUM_PORT_OVERRIDES)
        .with_static_renderer(renderers::static_sum),
    SimulinkBlockDefinition::new("Gain", "Math Operations")
        .with_description("Multiply input by a constant")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_shape(SimulinkShape::Triangle)
        .with_metadata_keys(&["Gain"])
        .with_block_label(BlockLabelPolicy::MetadataDependent(labels::gain_value)),
    // ── Sources / sinks ────────────────────────────────────────────────
    SimulinkBlockDefinition::new("Constant", "Sources")
        .with_description("Output a constant value")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("C")),
    SimulinkBlockDefinition::new("Scope", "Sinks")
        .with_description("Display signals over time")
        .with_ports(IOPorts::Fixed(1), IOPorts::None)
        .with_icon(icon("〰"))
        .with_static_renderer(renderers::static_scope),
    SimulinkBlockDefinition::new("Terminator", "Sinks")
        .with_description("Terminate an unconnected output port")
        .with_ports(IOPorts::Fixed(1), IOPorts::None)
        .with_icon(icon("⏹")),
    // ── Ports & subsystems ─────────────────────────────────────────────
    SimulinkBlockDefinition::new("Inport", "Ports & Subsystems")
        .with_description("Create an input port for a subsystem")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("⬅")),
    SimulinkBlockDefinition::new("Outport", "Ports & Subsystems")
        .with_description("Create an output port for a subsystem")
        .with_ports(IOPorts::Fixed(1), IOPorts::None)
        .with_icon(icon("➡")),
    SimulinkBlockDefinition::new("SubSystem", "Ports & Subsystems")
        .with_description("Group blocks into a subsystem")
        .with_ports(IOPorts::Variable(1), IOPorts::Variable(1))
        .with_icon(icon(""))
        .with_port_labels(
            PortLabelPolicy::MetadataDependent(port_labels_from_model),
            PortLabelPolicy::MetadataDependent(port_labels_from_model),
        ),
    SimulinkBlockDefinition::new("MATLAB Function", "User-Defined Functions")
        .with_description("Author block behaviour in MATLAB")
        .with_ports(IOPorts::Variable(1), IOPorts::Variable(1))
        .with_icon(icon("🖹"))
        .with_port_labels(
            PortLabelPolicy::MetadataDependent(port_labels_from_model),
            PortLabelPolicy::MetadataDependent(port_labels_from_model),
        ),
    SimulinkBlockDefinition::new("CFunction", "User-Defined Functions")
        .with_description("Author block behaviour in C")
        .with_ports(IOPorts::Variable(1), IOPorts::Variable(1))
        .with_icon(icon("📁"))
        .with_port_labels(
            PortLabelPolicy::MetadataDependent(port_labels_from_model),
            PortLabelPolicy::MetadataDependent(port_labels_from_model),
        ),
    // ── Signal routing ─────────────────────────────────────────────────
    SimulinkBlockDefinition::new("Concatenate", "Signal Routing")
        .with_description("Concatenate input signals")
        .with_ports(IOPorts::Variable(2), IOPorts::Fixed(1))
        .with_icon(icon("☰")),
    SimulinkBlockDefinition::new("Mux", "Signal Routing")
        .with_description("Combine signals into a vector")
        .with_ports(IOPorts::Variable(2), IOPorts::Fixed(1))
        .with_icon(icon("☰")),
    SimulinkBlockDefinition::new("Demux", "Signal Routing")
        .with_description("Split a vector into signals")
        .with_ports(IOPorts::Fixed(1), IOPorts::Variable(2))
        .with_icon(icon("☰")),
    SimulinkBlockDefinition::new("BusCreator", "Signal Routing")
        .with_description("Combine signals into a bus")
        .with_ports(IOPorts::Variable(2), IOPorts::Fixed(1))
        .with_icon(icon("☰")),
    SimulinkBlockDefinition::new("BusSelector", "Signal Routing")
        .with_description("Select signals from a bus")
        .with_ports(IOPorts::Fixed(1), IOPorts::Variable(2))
        .with_icon(icon("☰")),
    SimulinkBlockDefinition::new("ComplexToRealImag", "Math Operations")
        .with_description("Split a complex signal into real and imaginary parts")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(2))
        .with_icon(icon(""))
        .with_port_labels(
            PortLabelPolicy::Fixed(&["Re+Im"]),
            PortLabelPolicy::Fixed(&["Re", "Im"]),
        ),
    SimulinkBlockDefinition::new("ManualSwitch", "Signal Routing")
        .with_aliases(&["Manual Switch"])
        .with_description("Manually switch between two inputs")
        .with_ports(IOPorts::Fixed(2), IOPorts::Fixed(1))
        .with_icon(icon("🕂"))
        .with_static_renderer(renderers::static_manual_switch)
        .with_live_renderer(renderers::live_manual_switch),
    SimulinkBlockDefinition::new("Goto", "Signal Routing")
        .with_description("Send a signal to a matching From block")
        .with_ports(IOPorts::Fixed(1), IOPorts::None)
        .with_shape(SimulinkShape::Goto)
        .with_icon(SimulinkIcon::Phosphor(egui_phosphor::regular::ARROW_RIGHT))
        .with_metadata_keys(&["GotoTag"])
        .with_block_label(BlockLabelPolicy::MetadataDependent(labels::goto_tag))
        .with_static_renderer(renderers::static_goto_from),
    SimulinkBlockDefinition::new("From", "Signal Routing")
        .with_description("Receive a signal from a matching Goto block")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_shape(SimulinkShape::From)
        .with_icon(SimulinkIcon::Phosphor(egui_phosphor::regular::ARROW_LEFT))
        .with_metadata_keys(&["GotoTag"])
        .with_block_label(BlockLabelPolicy::MetadataDependent(labels::goto_tag))
        .with_static_renderer(renderers::static_goto_from),
];

/// Default port-label policy: take the labels from the parsed model.
///
/// Returning an empty vector tells the general renderer to fall back to its
/// per-port name resolution (port `Name` property, subsystem boundary names,
/// or generated `In1`/`Out1`).
fn port_labels_from_model(
    _block: &crate::model::Block,
    _meta: &crate::simulink_libraries::metadata::BlockMetadata,
    _is_input: bool,
) -> Vec<String> {
    Vec::new()
}
