//! The block-definition catalog: one folder, several files.
//!
//! Each submodule is a *library* contributing a `static` slice of
//! [`SimulinkBlockDefinition`]s.  To add your own blocks, create a new file
//! here, expose a `pub static BLOCKS: &[SimulinkBlockDefinition]`, and add it to
//! [`ALL_LIBRARIES`].  No renderer changes are required.

#![cfg(feature = "egui")]

pub mod core;
pub mod dashboard;

use crate::simulink_libraries::types::SimulinkLibrary;

/// Every library that contributes definitions to the unified catalog.
pub static ALL_LIBRARIES: &[SimulinkLibrary] = &[
    SimulinkLibrary {
        name: "simulink",
        blocks: self::core::BLOCKS,
    },
    SimulinkLibrary {
        name: "dashboard",
        blocks: self::dashboard::BLOCKS,
    },
];
