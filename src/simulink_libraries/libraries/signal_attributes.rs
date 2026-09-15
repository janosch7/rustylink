//! Signal attribute blocks: data type, width and rate conversions.
//!
//! Part of the block catalog: one file per Simulink library group.

#![cfg(feature = "egui")]

use super::prelude::plot;
use crate::simulink_libraries::renderers;
use crate::simulink_libraries::types::{IOPorts, MetadataKey, SimulinkBlockDefinition};

#[rustfmt::skip]
pub static BLOCKS: &[SimulinkBlockDefinition] = &[
    // ═══════════════════════════════════════════════════════════════════════
    //  Signal Attributes
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("DataTypeConversion", "Signal Attributes")
        .with_aliases(&["Data Type Conversion"])
        .with_description("Convert signal to specified data type")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_metadata_keys(&[MetadataKey::with_default("OutDataTypeStr", "Inherit: Inherit via back propagation")])
        .with_static_renderer(renderers::static_data_type_conversion),
    // Simulink draws the propagated width above the signal line running from
    // the input to the output port, tapped by a short diagonal stroke.
    SimulinkBlockDefinition::new("Width", "Signal Attributes")
        .with_description("Output width (number of elements) of input signal")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(plot(
            "t 0.45,0.28,0.36 -1; p 0.00,0.50 1.00,0.50; p 0.32,0.78 0.58,0.36",
        )),
    SimulinkBlockDefinition::new("BusToVector", "Signal Attributes")
        .with_aliases(&["Bus to Vector"])
        .with_description("Convert bus to a vector signal")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(plot(concat!(
            "p 0.05,0.30 0.40,0.30; p 0.05,0.50 0.40,0.50; p 0.05,0.70 0.40,0.70;",
            "f 0.40,0.10 0.56,0.90;",
            "p 0.56,0.50 0.95,0.50"
        ))),
];
