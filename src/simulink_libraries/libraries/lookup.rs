//! Lookup table blocks: n-D tables, interpolation, prelookup.
//!
//! Part of the block catalog: one file per Simulink library group.

#![cfg(feature = "egui")]

use crate::simulink_libraries::renderers;
use crate::simulink_libraries::types::{
    IOPorts, MetadataKey, PortLabelPolicy, SimulinkBlockDefinition,
};

#[rustfmt::skip]
pub static BLOCKS: &[SimulinkBlockDefinition] = &[
    // ═══════════════════════════════════════════════════════════════════════
    //  Lookup Tables
    // ═══════════════════════════════════════════════════════════════════════
    // Simulink captions the table with its dimensionality and plots the
    // breakpoint curve underneath.
    SimulinkBlockDefinition::new("Lookup_n-D", "Lookup Tables")
        .with_aliases(&["1-D Lookup Table", "2-D Lookup Table", "n-D Lookup Table"])
        .with_description("n-dimensional lookup table interpolation")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_metadata_keys(&[MetadataKey::with_default("NumberOfTableDimensions", "1")])
        .with_static_renderer(renderers::static_lookup_table),
    SimulinkBlockDefinition::new("Cosine", "Lookup Tables")
        .with_description("Sine and Cosine lookup-table function")
        .with_ports(IOPorts::Fixed(1), IOPorts::Variable(1))
        .with_metadata_keys(&[MetadataKey::with_default("Formula", "")])
        // Simulink draws no icon for these lookup-table Reference blocks; their
        // identity comes from the output port labels (`Formula`), so claim the
        // interior to keep the `?` placeholder away.
        .with_static_renderer(renderers::static_nothing)
        .with_port_labels(
            PortLabelPolicy::Fixed(&["u"]),
            PortLabelPolicy::MetadataDependent(renderers::sine_cosine_output_labels),
        ),
];
