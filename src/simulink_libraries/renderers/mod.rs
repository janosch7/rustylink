//! Per-block drawing routines referenced from the catalog definitions.
//!
//! One module per Simulink library group, mirroring the catalog layout in
//! [`crate::simulink_libraries::libraries`].  The definitions refer to these
//! functions by path, so every renderer is re-exported here.

#![cfg(feature = "egui")]

mod common;
pub mod continuous;
pub mod discontinuities;
pub mod discrete;
pub mod logic;
pub mod lookup;
pub mod math;
pub mod matrix;
pub mod ports_subsystems;
pub mod signal_attributes;
pub mod signal_routing;
pub mod sinks;
pub mod user_defined;

pub use common::*;
pub use continuous::*;
pub use discrete::*;
pub use logic::*;
pub use lookup::*;
pub use math::*;
pub use matrix::*;
pub use ports_subsystems::*;
pub use signal_attributes::*;
pub use signal_routing::*;
pub use sinks::*;
pub use user_defined::*;

#[cfg(test)]
mod tests {
    use super::continuous::{format_coeff, format_polynomial};
    use super::signal_routing::{
        bus_assignment_port_labels, compute_multiport_selection, evaluate_switch_criteria,
        multiport_switch_port_labels,
    };
    use super::sinks::owner_caption;
    use crate::model::{Block, PortCounts};
    use crate::simulink_libraries::metadata::BlockMetadata;

    #[test]
    fn owner_caption_uses_the_owner_name_and_parameter() {
        assert_eq!(
            owner_caption(Some("../Delay"), None),
            Some("Delay".to_string())
        );
        assert_eq!(
            owner_caption(Some("../Add Constant"), Some("Bias")),
            Some("Add Constant.Bias".to_string())
        );
        assert_eq!(owner_caption(Some("  "), Some("Bias")), None);
        assert_eq!(owner_caption(None, None), None);
    }

    #[test]
    fn polynomial_from_bracketed_vector() {
        assert_eq!(format_polynomial("[1 2 1]", 's'), "s^2+2s+1");
        assert_eq!(format_polynomial("[1 1]", 's'), "s+1");
        assert_eq!(format_polynomial("[1]", 's'), "1");
    }

    #[test]
    fn polynomial_handles_commas_zeros_and_signs() {
        assert_eq!(format_polynomial("1,0,-4", 's'), "s^2-4");
        assert_eq!(format_polynomial("[2 0 0]", 's'), "2s^2");
        assert_eq!(format_polynomial("[]", 's'), "1");
        assert_eq!(format_polynomial("[0 0]", 's'), "0");
    }

    #[test]
    fn coefficient_formatting_trims_trailing_zeros() {
        assert_eq!(format_coeff(3.0), "3");
        assert_eq!(format_coeff(2.5), "2.5");
    }

    /// Build a minimal MultiPortSwitch block with `ins` input ports.
    fn multiport_block(ins: u32) -> Block {
        let mut block = super::super::stubs::create_stub_block("MultiPortSwitch", ins, 1);
        block.port_counts = Some(PortCounts {
            ins: Some(ins),
            outs: Some(1),
            ..Default::default()
        });
        block
    }

    #[test]
    fn multiport_switch_one_based_default_last_port() {
        let block = multiport_block(4);
        let mut meta = BlockMetadata::default();
        meta.insert("Inputs", "3");
        // Default: One-based contiguous, last port is default.
        let labels = multiport_switch_port_labels(&block, &meta, true);
        assert_eq!(labels, vec!["", "1", "2", "*, 3"]);
    }

    #[test]
    fn multiport_switch_zero_based_default_last_port() {
        let block = multiport_block(4);
        let mut meta = BlockMetadata::default();
        meta.insert("DataPortOrder", "Zero-based contiguous");
        // No Inputs property → falls back to port_counts.ins - 1 = 3.
        let labels = multiport_switch_port_labels(&block, &meta, true);
        assert_eq!(labels, vec!["", "0", "1", "*, 2"]);
    }

    #[test]
    fn multiport_switch_specify_indices_default_last_port() {
        let block = multiport_block(4);
        let mut meta = BlockMetadata::default();
        meta.insert("DataPortOrder", "Specify indices");
        meta.insert("DataPortIndices", "{6,8,15}");
        let labels = multiport_switch_port_labels(&block, &meta, true);
        assert_eq!(labels, vec!["", "6", "8", "*, 15"]);
    }

    #[test]
    fn multiport_switch_specify_indices_additional_default() {
        let block = multiport_block(5);
        let mut meta = BlockMetadata::default();
        meta.insert("DataPortOrder", "Specify indices");
        meta.insert("DataPortIndices", "{6,8,15}");
        meta.insert("DataPortForDefault", "Additional data port");
        let labels = multiport_switch_port_labels(&block, &meta, true);
        assert_eq!(labels, vec!["", "6", "8", "15", "*"]);
    }

    #[test]
    fn multiport_switch_one_based_additional_default() {
        let block = multiport_block(5);
        let mut meta = BlockMetadata::default();
        meta.insert("Inputs", "4");
        meta.insert("DataPortForDefault", "Additional data port");
        let labels = multiport_switch_port_labels(&block, &meta, true);
        assert_eq!(labels, vec!["", "1", "2", "3", "4", "*"]);
    }

    #[test]
    fn multiport_switch_output_labels_are_empty() {
        let block = multiport_block(4);
        let meta = BlockMetadata::default();
        let labels = multiport_switch_port_labels(&block, &meta, false);
        assert!(labels.is_empty());
    }

    #[test]
    fn switch_criteria_ge_threshold() {
        assert!(evaluate_switch_criteria("u2 >= Threshold", 5.0, 0.0));
        assert!(evaluate_switch_criteria("u2 >= Threshold", 0.0, 0.0));
        assert!(!evaluate_switch_criteria("u2 >= Threshold", -1.0, 0.0));
    }

    #[test]
    fn switch_criteria_gt_threshold() {
        assert!(evaluate_switch_criteria("u2 > Threshold", 5.0, 0.0));
        assert!(!evaluate_switch_criteria("u2 > Threshold", 0.0, 0.0));
    }

    #[test]
    fn switch_criteria_ne_literal() {
        assert!(evaluate_switch_criteria("u2 ~= 0", 5.0, 0.0));
        assert!(!evaluate_switch_criteria("u2 ~= 0", 0.0, 0.0));
    }

    #[test]
    fn switch_criteria_le_threshold() {
        assert!(evaluate_switch_criteria("u2 <= Threshold", -1.0, 0.0));
        assert!(evaluate_switch_criteria("u2 <= Threshold", 0.0, 0.0));
        assert!(!evaluate_switch_criteria("u2 <= Threshold", 1.0, 0.0));
    }

    #[test]
    fn multiport_selection_one_based() {
        let block = multiport_block(4);
        let mut meta = BlockMetadata::default();
        meta.insert("Inputs", "3");
        // control=2 → select data port index 1 (0-based)
        assert_eq!(compute_multiport_selection(&block, &meta, 2.0, 3), 1);
        // control=1 → select data port index 0
        assert_eq!(compute_multiport_selection(&block, &meta, 1.0, 3), 0);
        // control=5 (out of range) → default = last port (index 2)
        assert_eq!(compute_multiport_selection(&block, &meta, 5.0, 3), 2);
    }

    #[test]
    fn multiport_selection_zero_based() {
        let block = multiport_block(4);
        let mut meta = BlockMetadata::default();
        meta.insert("DataPortOrder", "Zero-based contiguous");
        // control=0 → select data port index 0
        assert_eq!(compute_multiport_selection(&block, &meta, 0.0, 3), 0);
        // control=2 → select data port index 2
        assert_eq!(compute_multiport_selection(&block, &meta, 2.0, 3), 2);
        // control=5 (out of range) → default = last port (index 2)
        assert_eq!(compute_multiport_selection(&block, &meta, 5.0, 3), 2);
    }

    #[test]
    fn multiport_selection_specify_indices() {
        let block = multiport_block(4);
        let mut meta = BlockMetadata::default();
        meta.insert("DataPortOrder", "Specify indices");
        meta.insert("DataPortIndices", "{6,8,15}");
        // control=8 → select index 1 (the "8" port)
        assert_eq!(compute_multiport_selection(&block, &meta, 8.0, 3), 1);
        // control=6 → select index 0
        assert_eq!(compute_multiport_selection(&block, &meta, 6.0, 3), 0);
        // control=10 (no match) → default = last numbered port (index 2)
        assert_eq!(compute_multiport_selection(&block, &meta, 10.0, 3), 2);
    }

    #[test]
    fn multiport_selection_specify_indices_additional_default() {
        let block = multiport_block(5);
        let mut meta = BlockMetadata::default();
        meta.insert("DataPortOrder", "Specify indices");
        meta.insert("DataPortIndices", "{6,8,15}");
        meta.insert("DataPortForDefault", "Additional data port");
        // control=10 (no match) → additional default port (index 3)
        assert_eq!(compute_multiport_selection(&block, &meta, 10.0, 4), 3);
    }

    #[test]
    fn bus_assignment_port_labels_basic() {
        let block = multiport_block(4);
        let mut meta = BlockMetadata::default();
        meta.insert("AssignedSignals", "bus_b.e,bus_c.d,bus_c.bus_a");
        let labels = bus_assignment_port_labels(&block, &meta, true);
        assert_eq!(
            labels,
            vec!["Bus", ":= bus_b.e", ":= bus_c.d", ":= bus_c.bus_a"]
        );
    }

    #[test]
    fn bus_assignment_port_labels_empty() {
        let block = multiport_block(2);
        let meta = BlockMetadata::default();
        let labels = bus_assignment_port_labels(&block, &meta, true);
        assert_eq!(labels, vec!["Bus"]);
    }

    #[test]
    fn bus_assignment_output_port_is_labelled_bus() {
        let block = multiport_block(4);
        let mut meta = BlockMetadata::default();
        meta.insert("AssignedSignals", "a,b");
        let labels = bus_assignment_port_labels(&block, &meta, false);
        assert_eq!(labels, vec!["Bus"]);
    }
}
