//! Discontinuity blocks: saturation, dead zone, rate limiter, backlash.
//!
//! Part of the block catalog: one file per Simulink library group.

#![cfg(feature = "egui")]

use super::prelude::{axes, plot};
use crate::simulink_libraries::types::{IOPorts, SimulinkBlockDefinition};

#[rustfmt::skip]
pub static BLOCKS: &[SimulinkBlockDefinition] = &[
    // ═══════════════════════════════════════════════════════════════════════
    //  Discontinuities
    // ═══════════════════════════════════════════════════════════════════════
    // Simulink draws the discontinuity icons as transfer curves over a faint
    // axis cross: a slanted hysteresis ladder, a friction curve with a jump at
    // the origin, and the flat-rise-flat saturation curve.
    SimulinkBlockDefinition::new("Backlash", "Discontinuities")
        .with_description("Model backlash (dead-zone in a mechanical gear)")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(plot(concat!(
            axes!(),
            "p 0.13,0.90 0.50,0.90 0.87,0.12 0.50,0.12 0.13,0.90;",
            "p 0.25,0.64 0.62,0.64; p 0.38,0.38 0.75,0.38"
        ))),
    SimulinkBlockDefinition::new("Saturate", "Discontinuities")
        .with_aliases(&["Saturation"])
        .with_description("Limit input signal to upper and lower bounds")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(plot(concat!(
            axes!(),
            "p 0.06,0.82 0.32,0.82 0.68,0.16 0.94,0.16"
        ))),
    SimulinkBlockDefinition::new("CoulombViscousFriction", "Discontinuities")
        .with_aliases(&["Coulomb & Viscous Friction", "Coulomb"])
        .with_description("Coulomb and viscous friction model")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(plot(concat!(
            axes!(),
            "p 0.06,0.94 0.50,0.62; p 0.50,0.38 0.94,0.06"
        ))),
];
