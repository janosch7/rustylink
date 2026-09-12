//! Shared helpers for the per-group block definition files.
//!
//! The definitions themselves live in one file per Simulink library group
//! (`sources.rs`, `math.rs`, …); everything they have in common — the icon
//! constructors, the recurring line-art fragments and the default port-label
//! policy — is kept here so the group files contain nothing but data.

#![cfg(feature = "egui")]

use crate::simulink_libraries::types::SimulinkIcon;

/// A single glyph drawn centred in the block.
pub(super) const fn icon(glyph: &'static str) -> SimulinkIcon {
    SimulinkIcon::Utf8(glyph)
}

/// Typeset-math icon (fraction bar / superscript / overbar); see
/// [`crate::egui_app::render::draw_math_icon`] for the notation.
pub(super) const fn math(spec: &'static str) -> SimulinkIcon {
    SimulinkIcon::Math(spec)
}

/// Line-art icon; see [`crate::egui_app::render::draw_plot_icon`] for the
/// notation.  Simulink draws source waveforms, discontinuity curves and
/// verification plots as vector line art rather than as glyphs.
pub(super) const fn plot(spec: &'static str) -> SimulinkIcon {
    SimulinkIcon::Plot(spec)
}

/// The thin grey axis cross Simulink draws behind most line-art icons.
/// A macro (not a `const`) so it can be `concat!`-ed into an icon spec.
macro_rules! axes {
    () => {
        "a 0.02,0.5 0.98,0.5; a 0.5,0.02 0.5,0.98;"
    };
}

/// The plot frame (left/bottom rules) behind every Model Verification icon.
macro_rules! check_axes {
    () => {
        "a 0.10,0.10 0.10,0.92; a 0.04,0.80 0.96,0.80;"
    };
}

pub(super) use {axes, check_axes};

/// A jittery trace – Simulink's icon for the random / noise sources.
pub(super) const NOISE_TRACE: &str = concat!(
    "p 0.05,0.52 0.12,0.28 0.19,0.66 0.26,0.34 0.33,0.74 0.40,0.22 0.47,0.58",
    " 0.54,0.30 0.61,0.70 0.68,0.40 0.75,0.24 0.82,0.62 0.89,0.36 0.95,0.54"
);
/// A rising staircase – the counter sources.
pub(super) const STAIRCASE: &str = concat!(
    "p 0.06,0.88 0.24,0.88 0.24,0.68 0.42,0.68 0.42,0.48 0.60,0.48",
    " 0.60,0.28 0.78,0.28 0.78,0.12 0.94,0.12"
);
/// Repeating ramps that reset – the repeating-sequence sources.
pub(super) const SAWTOOTH: &str =
    "p 0.06,0.86 0.30,0.14 0.30,0.86 0.54,0.14 0.54,0.86 0.78,0.14 0.78,0.86 0.94,0.38";

/// Default port-label policy: take the labels from the parsed model.
///
/// Returning an empty vector tells the general renderer to fall back to its
/// per-port name resolution (port `Name` property, subsystem boundary names,
/// or generated `In1`/`Out1`).
pub(super) fn port_labels_from_model(
    _block: &crate::model::Block,
    _meta: &crate::simulink_libraries::metadata::BlockMetadata,
    _is_input: bool,
) -> Vec<String> {
    Vec::new()
}
