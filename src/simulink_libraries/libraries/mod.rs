//! The block-definition catalog: one folder, several files.
//!
//! Each submodule is a *library* contributing a `static` slice of
//! [`SimulinkBlockDefinition`]s.  To add your own blocks, create a new file
//! here, expose a `pub static BLOCKS: &[SimulinkBlockDefinition]`, and add it to
//! [`ALL_LIBRARIES`].  No renderer changes are required.

#![cfg(feature = "egui")]

pub mod catalog;
pub mod core;
pub mod dashboard;
pub mod matrix;
pub mod simulink_blocks;

use crate::simulink_libraries::types::SimulinkLibrary;

/// Hand-written libraries with full rendering metadata (icons, shapes, custom
/// static/live renderers).  Registered first so they win on key collisions.
pub static ALL_LIBRARIES: &[SimulinkLibrary] = &[
    SimulinkLibrary {
        name: "simulink",
        blocks: self::core::BLOCKS,
    },
    SimulinkLibrary {
        name: "simulink",
        blocks: self::simulink_blocks::BLOCKS,
    },
    SimulinkLibrary {
        name: "dashboard",
        blocks: self::dashboard::BLOCKS,
    },
    SimulinkLibrary {
        name: "matrix_library",
        blocks: self::matrix::BLOCKS,
    },
];

/// The metadata-only browser/palette catalog (~786 entries).  Registered after
/// the rich libraries and the bridged virtual libraries, so it only fills in
/// block types those do not already provide.
pub static PALETTE: &[crate::simulink_libraries::types::SimulinkBlockDefinition] =
    self::catalog::BLOCKS;
