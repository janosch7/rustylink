//! Shared line art for the Discontinuities blocks.
//!
//! Part of the renderer set: one module per Simulink library group.

#![cfg(feature = "egui")]

/// A saturation curve (flat, ramp, flat) normalised to the icon area – the
/// marker Simulink adds to an integrator whose output is limited.
pub(super) const SATURATION_CURVE: &str = "p 0.04,0.84 0.30,0.84 0.72,0.16 0.96,0.16";
