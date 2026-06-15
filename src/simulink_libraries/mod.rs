//! Unified Simulink block-definition catalog.
//!
//! This module is the single source of truth describing how each block is
//! recognised, laid out, labelled and drawn.  A [`SimulinkBlockDefinition`]
//! carries the block's shape, optional icon, port-label and block-label
//! policies, optional metadata extraction, and optional static (live-off) and
//! live (live-on) interior renderers.
//!
//! Layout:
//! - [`stubs`]      – core (non-`egui`) parser-facing port/stub metadata.
//! - [`types`]      – the definition struct, enums and renderer signatures.
//! - [`metadata`]   – per-instance metadata extraction into a `HashMap`.
//! - [`labels`]     – metadata-dependent label helpers.
//! - [`renderers`]  – reusable static/live interior renderers.
//! - [`libraries`]  – the catalog itself, split into one file per library so
//!   users can drop in their own.
//! - [`resolver`]   – O(1) resolution of a parsed block to its definition.
//! - [`config`]     – bridges definitions to the legacy `BlockTypeConfig`.
//! - [`render`]     – the single general interior renderer.
//!
//! The catalog's rich definitions (icons, shapes, painter-based renderers)
//! require the `egui` feature; only the lightweight [`stubs`] data is available
//! in core builds, so the parser can resolve library port counts without
//! pulling in `egui`.

pub mod stubs;

#[cfg(feature = "egui")]
pub mod browser;
#[cfg(feature = "egui")]
pub mod config;
#[cfg(feature = "egui")]
pub mod labels;
#[cfg(feature = "egui")]
pub mod libraries;
#[cfg(feature = "egui")]
pub mod metadata;
#[cfg(feature = "egui")]
pub mod render;
#[cfg(feature = "egui")]
pub mod renderers;
#[cfg(feature = "egui")]
pub mod resolver;
#[cfg(feature = "egui")]
pub mod types;

#[cfg(feature = "egui")]
pub use render::{InteriorParams, render_block_interior};
#[cfg(feature = "egui")]
pub use resolver::{register_user_definition, resolve_definition};
#[cfg(feature = "egui")]
pub use types::{
    BlockLabelPolicy, IOPorts, PortLabelPolicy, SimulinkBlockDefinition, SimulinkIcon,
    SimulinkShape,
};
