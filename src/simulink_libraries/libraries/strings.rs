//! String operation blocks.
//!
//! Part of the block catalog: one file per Simulink library group.

#![cfg(feature = "egui")]

use crate::simulink_libraries::labels;
use crate::simulink_libraries::types::{
    BlockLabelPolicy, IOPorts, MetadataKey, SimulinkBlockDefinition,
};

#[rustfmt::skip]
pub static BLOCKS: &[SimulinkBlockDefinition] = &[
    // ═══════════════════════════════════════════════════════════════════════
    //  String Operations
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("ASCIIToString", "String Operations")
        .with_aliases(&["ASCII to String"])
        .with_description("Convert ASCII codes to string")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_block_label(BlockLabelPolicy::Fixed("ASCII \u{27F6} string")),
    SimulinkBlockDefinition::new("ToString", "String Operations")
        .with_aliases(&["To String"])
        .with_description("Convert input to string representation")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_block_label(BlockLabelPolicy::Fixed("\u{27F6} string")),
    // Simulink shows the configured literal, defaulting to `"Hello!"`.
    SimulinkBlockDefinition::new("StringConstant", "String Operations")
        .with_aliases(&["String Constant"])
        .with_description("Output a constant string value")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_metadata_keys(&[MetadataKey::with_default("String", "\"Hello!\"")])
        .with_block_label(BlockLabelPolicy::MetadataDependent(labels::string_constant)),
];
