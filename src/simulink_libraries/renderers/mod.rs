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
