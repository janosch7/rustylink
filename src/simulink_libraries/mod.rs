//! Unified Simulink block-definition catalog.
//!
//! This module is the single source of truth describing how each block is
//! recognised, laid out, labelled and drawn.  A [`SimulinkBlockDefinition`]
//! carries the block's shape, optional icon, port-label and block-label
//! policies, optional metadata extraction, and optional static (live-off) and
//! live (live-on) interior renderers.
//!
//! Layout:
//! - [`types`]      – the definition struct, enums and renderer signatures.
//! - [`metadata`]   – per-instance metadata extraction into a `HashMap`.
//! - [`labels`]     – metadata-dependent label helpers.
//! - [`renderers`]  – reusable static/live interior renderers.
//! - [`libraries`]  – the catalog itself, split into one file per library so
//!   users can drop in their own.
//! - [`resolver`]   – O(1) resolution of a parsed block to its definition.
//! - [`bridge`]     – adapts the legacy virtual libraries into definitions.
//! - [`config`]     – bridges definitions to the legacy `BlockTypeConfig`.
//! - [`render`]     – the single general interior renderer.

#![cfg(feature = "egui")]

pub mod bridge;
pub mod config;
pub mod labels;
pub mod libraries;
pub mod metadata;
pub mod render;
pub mod renderers;
pub mod resolver;
pub mod types;

pub use render::{InteriorParams, render_block_interior};
pub use resolver::{register_user_definition, resolve_definition};
pub use types::{
    BlockLabelPolicy, IOPorts, PortLabelPolicy, SimulinkBlockDefinition, SimulinkIcon,
    SimulinkShape,
};
