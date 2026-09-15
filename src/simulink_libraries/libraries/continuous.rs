//! Continuous-time blocks: integrators, derivatives, transfer functions.
//!
//! Part of the block catalog: one file per Simulink library group.

#![cfg(feature = "egui")]

use super::prelude::math;
use crate::simulink_libraries::renderers;
use crate::simulink_libraries::types::{
    IOPorts, MetadataKey, PortLabelPolicy, SimulinkBlockDefinition,
};

#[rustfmt::skip]
pub static BLOCKS: &[SimulinkBlockDefinition] = &[
    // ═══════════════════════════════════════════════════════════════════════
    //  Continuous
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("Derivative", "Continuous")
        .with_description("Output the time derivative of the input")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(math("frac:\u{0394}u/\u{0394}t")),
    // Simulink draws the plain Integrator as `1/s`; enabling output limits
    // (`LimitOutput`) or state wrapping (`WrapState`) decorates it, and the
    // external reset / initial-condition sources add labelled input ports.
    SimulinkBlockDefinition::new("Integrator", "Continuous")
        .with_description("Integrate input signal over time")
        .with_ports(IOPorts::Variable(1), IOPorts::Fixed(1))
        .with_icon(math("frac:1/s"))
        .with_metadata_keys(&[
            MetadataKey::with_default("LimitOutput", "off"),
            MetadataKey::with_default("WrapState", "off"),
            MetadataKey::with_default("ExternalReset", "none"),
            MetadataKey::with_default("InitialConditionSource", "internal"),
        ])
        .with_static_renderer(renderers::static_integrator)
        .with_port_labels(
            PortLabelPolicy::MetadataDependent(renderers::integrator_port_labels),
            PortLabelPolicy::None,
        ),
    SimulinkBlockDefinition::new("TransferFcn", "Continuous")
        .with_aliases(&["Transfer Fcn", "Transfer Function"])
        .with_description("Linear transfer function (numerator/denominator in s)")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_metadata_keys(&[
            MetadataKey::with_default("Numerator", "[1]"),
            MetadataKey::with_default("Denominator", "[1 1]"),
        ])
        .with_static_renderer(renderers::static_transfer_fcn),
    SimulinkBlockDefinition::new("SecondOrderIntegrator", "Continuous")
        .with_aliases(&["Second-Order Integrator"])
        .with_description("Integrate twice: acceleration to position")
        .with_ports(IOPorts::Variable(1), IOPorts::Fixed(2))
        .with_metadata_keys(&[
            MetadataKey::with_default("LimitX", "off"),
            MetadataKey::with_default("WrapX", "off"),
            MetadataKey::with_default("LimitDXDT", "off"),
            MetadataKey::with_default("ICSourceX", "internal"),
            MetadataKey::with_default("ICSourceDXDT", "internal"),
        ])
        .with_static_renderer(renderers::static_second_order_integrator)
        .with_port_labels(
            PortLabelPolicy::MetadataDependent(renderers::second_order_integrator_port_labels),
            PortLabelPolicy::MetadataDependent(renderers::second_order_integrator_port_labels),
        ),
    SimulinkBlockDefinition::new("DescriptorStateSpace", "Continuous")
        .with_aliases(&["Descriptor State-Space"])
        .with_description("Descriptor state-space model E*dx = Ax + Bu")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(math("lines:E\u{1E8B} = Ax + Bu|y = Cx + Du")),
];
