//! Discrete-time blocks: unit delays, discrete integrators and filters.
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
    //  Discrete
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("Delay", "Discrete")
        .with_description("Delay input by variable number of sample periods")
        .with_ports(IOPorts::Variable(1), IOPorts::Fixed(1))
        .with_metadata_keys(&[
            MetadataKey::with_default("DelayLength", "2"),
            MetadataKey::with_default("DelayLengthSource", "Dialog"),
            MetadataKey::with_default("InputPortMap", "u0"),
            MetadataKey::with_default("ExternalReset", "None"),
        ])
        .with_static_renderer(renderers::static_delay)
        .with_port_labels(
            PortLabelPolicy::MetadataDependent(renderers::delay_port_labels),
            PortLabelPolicy::None,
        ),
    SimulinkBlockDefinition::new("DiscreteIntegrator", "Discrete")
        .with_aliases(&["Discrete-Time Integrator", "Discrete Time Integrator"])
        .with_description("Discrete-time integrator / accumulator")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_metadata_keys(&[MetadataKey::with_default("IntegratorMethod", "Integration: Forward Euler")])
        .with_static_renderer(renderers::static_discrete_integrator),
    SimulinkBlockDefinition::new("UnitDelay", "Discrete")
        .with_aliases(&["Unit Delay"])
        .with_description("Delay input by one sample period")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(math("frac:1/z")),
    SimulinkBlockDefinition::new("Difference", "Discrete")
        .with_description("Compute difference between successive samples")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(math("frac:z-1/z")),
    SimulinkBlockDefinition::new("Discrete Derivative", "Discrete")
        .with_aliases(&["DiscreteDerivative"])
        .with_description("Discrete-time derivative K(z-1)/(Ts z)")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(math("frac:K(z-1)/Ts z")),
];
