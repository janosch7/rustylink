//! Logic and bit operation blocks: comparisons, bitwise math, detectors.
//!
//! Part of the block catalog: one file per Simulink library group.

#![cfg(feature = "egui")]

use super::prelude::icon;
use crate::simulink_libraries::labels;
use crate::simulink_libraries::renderers;
use crate::simulink_libraries::types::{
    BlockLabelPolicy, IOPorts, MetadataKey, SimulinkBlockDefinition, SimulinkShape,
};

#[rustfmt::skip]
pub static BLOCKS: &[SimulinkBlockDefinition] = &[
    // ═══════════════════════════════════════════════════════════════════════
    //  Logic and Bit Operations
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("Logic", "Logic and Bit Operations")
        .with_aliases(&["Logical Operator"])
        .with_description("Perform logical operation (AND, OR, NOT, ...)")
        .with_ports(IOPorts::Variable(2), IOPorts::Fixed(1))
        .with_shape(SimulinkShape::None)
        .with_metadata_keys(&[
            MetadataKey::with_default("Operator", "AND"),
            MetadataKey::with_default("IconShape", "rectangular"),
        ])
        .with_static_renderer(renderers::static_logic),
    SimulinkBlockDefinition::new("RelationalOperator", "Logic and Bit Operations")
        .with_aliases(&["Relational Operator"])
        .with_description("Compare two inputs (<=, >=, ==, ~=)")
        .with_ports(IOPorts::Fixed(2), IOPorts::Fixed(1))
        .with_metadata_keys(&[MetadataKey::with_default("Operator", "<=")])
        .with_block_label(BlockLabelPolicy::MetadataDependent(labels::relational_operator)),
    SimulinkBlockDefinition::new("BitClear", "Logic and Bit Operations")
        .with_aliases(&["Bit Clear"])
        .with_description("Clear specified bit of stored integer")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_metadata_keys(&[MetadataKey::with_default("iBit", "0")])
        .with_block_label(BlockLabelPolicy::MetadataDependent(labels::bit_clear)),
    SimulinkBlockDefinition::new("BitSet", "Logic and Bit Operations")
        .with_aliases(&["Bit Set"])
        .with_description("Set specified bit of stored integer")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_metadata_keys(&[MetadataKey::with_default("iBit", "0")])
        .with_block_label(BlockLabelPolicy::MetadataDependent(labels::bit_set)),
    SimulinkBlockDefinition::new("CompareToZero", "Logic and Bit Operations")
        .with_aliases(&["Compare To Zero", "Compare"])
        .with_description("Compare input signal to zero")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("<= 0"))
        .with_instance_label(labels::compare_to_zero),
    SimulinkBlockDefinition::new("CompareToConstant", "Logic and Bit Operations")
        .with_aliases(&["Compare To Constant"])
        .with_description("Compare input signal to a constant")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("<= K"))
        .with_instance_label(labels::compare_to_constant),
    // Simulink labels these with the comparison they perform against the
    // previous sample (`U/z`), not with an arrow.
    SimulinkBlockDefinition::new("DetectDecrease", "Logic and Bit Operations")
        .with_aliases(&["Detect Decrease"])
        .with_description("Detect decrease in signal value")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("U < U/z")),
    SimulinkBlockDefinition::new("DetectIncrease", "Logic and Bit Operations")
        .with_aliases(&["Detect Increase"])
        .with_description("Detect increase in signal value")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("U > U/z")),
    SimulinkBlockDefinition::new("DetectChange", "Logic and Bit Operations")
        .with_aliases(&["Detect Change"])
        .with_description("Detect change in signal value")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("U ~= U/z")),
];
