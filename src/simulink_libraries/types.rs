//! Core types for the unified Simulink block-definition catalog.
//!
//! A [`SimulinkBlockDefinition`] is the single source of truth describing how a
//! given Simulink block type is recognised, laid out, labelled and drawn.  Both
//! the viewer (`egui_app`) and the editor consume these definitions through the
//! general renderer in [`super::render`], so there is no block-specific code in
//! the renderers themselves.
//!
//! Definitions live in [`super::libraries`] – one file per library – and are
//! aggregated by [`super`].  Adding a new block is as simple as appending a
//! `SimulinkBlockDefinition` to the relevant library file (or registering a new
//! user library), no renderer changes required.

#![cfg(feature = "egui")]

use std::collections::HashMap;

use eframe::egui::Painter;

use crate::model::Block;

use super::metadata::BlockMetadata;

/// An icon drawn at the centre of a block when no custom renderer applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SimulinkIcon {
    /// A single UTF-8 glyph (e.g. `"×"` for Product).
    Utf8(&'static str),
    /// A path to an embedded SVG asset (see `egui_app::icon_assets`).
    Svg(&'static str),
    /// A Phosphor icon name.
    Phosphor(&'static str),
}

/// The body shape used to draw a block.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum SimulinkShape {
    /// Standard rounded rectangle (default).
    #[default]
    Rectangle,
    /// Right-pointing triangle (e.g. Gain).
    Triangle,
    /// Circle (e.g. Sum).
    Circle,
    /// Solid black rectangle with no interior (e.g. Bus Creator/Selector).
    FilledBlack,
    /// Rectangle with a triangular tab pointing left (Goto).
    Goto,
    /// Rectangle with a triangular tab pointing right (From).
    From,
}

/// Where a port sits on a block body.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PortPlacement {
    Left,
    Right,
    Top,
    Bottom,
}

/// Overrides the default evenly-distributed position of a single port.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PortPositionOverride {
    pub is_input: bool,
    /// 1-based port index.
    pub port_index: u32,
    pub placement: PortPlacement,
    /// Position along the chosen side, `0.0..=1.0`.
    pub fraction: f32,
}

/// Number of ports on one side of a block.
///
/// Mirrors the user-facing taxonomy: a block has either no ports, a fixed
/// number, or a variable number with a sensible default used when the model
/// data does not specify a count.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IOPorts {
    None,
    Fixed(u32),
    Variable(u32),
}

impl IOPorts {
    /// Default port count used for rendering when the model carries no count.
    pub const fn default_count(self) -> u32 {
        match self {
            IOPorts::None => 0,
            IOPorts::Fixed(n) | IOPorts::Variable(n) => n,
        }
    }
}

/// How the labels for a block's input or output ports are produced.
///
/// One of the three user-requested modes: none, fixed, or metadata-dependent.
#[derive(Clone, Copy, Default)]
pub enum PortLabelPolicy {
    /// No port labels.
    #[default]
    None,
    /// Fixed labels, indexed by `(port_index - 1)`.
    Fixed(&'static [&'static str]),
    /// Labels derived from the block's parsed metadata.
    ///
    /// Called with the block, its extracted metadata, and `is_input`; returns
    /// one label per port (index 0 == port 1).  An empty/short vector falls
    /// back to the generated default for the missing indices.
    MetadataDependent(fn(&Block, &BlockMetadata, bool) -> Vec<String>),
}

impl std::fmt::Debug for PortLabelPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortLabelPolicy::None => write!(f, "None"),
            PortLabelPolicy::Fixed(s) => f.debug_tuple("Fixed").field(s).finish(),
            PortLabelPolicy::MetadataDependent(_) => write!(f, "MetadataDependent(<fn>)"),
        }
    }
}

/// How the main label drawn inside/around a block is produced.
///
/// One of the three user-requested modes: none, fixed, or metadata-dependent.
#[derive(Clone, Copy, Default)]
pub enum BlockLabelPolicy {
    /// No textual block label (icon-only / custom renderer).
    #[default]
    None,
    /// A constant label.
    Fixed(&'static str),
    /// A label derived from the block's parsed metadata (e.g. a Gain value or
    /// a Goto tag).  Returning `None` falls back to the icon.
    MetadataDependent(fn(&Block, &BlockMetadata) -> Option<String>),
}

impl std::fmt::Debug for BlockLabelPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockLabelPolicy::None => write!(f, "None"),
            BlockLabelPolicy::Fixed(s) => f.debug_tuple("Fixed").field(s).finish(),
            BlockLabelPolicy::MetadataDependent(_) => write!(f, "MetadataDependent(<fn>)"),
        }
    }
}

/// Context passed to renderers so they can adapt to live-mode and metadata.
pub struct RenderContext<'a> {
    /// Whether the viewer is currently in live (simulation playback) mode.
    pub live_mode: bool,
    /// Uniform font scaling factor for the current zoom level.
    pub font_scale: f32,
    /// Extra factor applied to in-block name labels.
    pub name_font_factor: f32,
    /// The block's extracted metadata (read model data, keyed in a HashMap).
    pub metadata: &'a BlockMetadata,
    /// The current live scalar value for this block, when available.
    pub live_value: Option<f64>,
    /// The current live text for this block, when available.
    pub live_text: Option<&'a str>,
    /// Display options for live value formatting (dashboard overlays).
    pub live_display_options: Option<&'a crate::live_values::LiveValueDisplayOptions>,
    /// Screen-space Y coordinates of the block's ports, when computed by the UI
    /// (used by geometry-aware renderers such as ManualSwitch).
    pub port_y: Option<&'a crate::egui_app::render::ComputedPortYCoordinates>,
    /// Max measured width of inside port labels, used to keep the icon clear of
    /// them.
    pub port_label_widths: Option<crate::egui_app::render::PortLabelMaxWidths>,
    /// Foreground/contrast color for plain-text labels drawn by renderers.
    pub text_color: eframe::egui::Color32,
}

/// Signature of an interior renderer used when **live mode is OFF**.
///
/// Draws the static representation of a block (icon, symbol, etc.).  Returning
/// `false` tells the general renderer to fall back to default icon/label
/// rendering; returning `true` means the renderer fully handled the interior.
pub type StaticRendererFn = fn(&Painter, &Block, &eframe::egui::Rect, &RenderContext<'_>) -> bool;

/// Signature of an interior renderer used when **live mode is ON**.
///
/// Draws the live representation (gauges, scopes, live values, …).  Returning
/// `false` falls back to the static renderer / default rendering.
pub type LiveRendererFn = fn(&Painter, &Block, &eframe::egui::Rect, &RenderContext<'_>) -> bool;

/// A single block type in the unified catalog.
///
/// This is intentionally a plain data struct built with `const` builder methods
/// so libraries can declare definitions in `static` arrays with zero runtime
/// cost.
#[derive(Clone, Copy)]
pub struct SimulinkBlockDefinition {
    /// Canonical block type / name (e.g. `"Gain"`).
    pub block_type: &'static str,
    /// Alternate names/paths that resolve to this definition.
    pub aliases: &'static [&'static str],
    /// Catalog category (used by the block browser).
    pub category: &'static str,
    /// Short human description (block browser tooltip).
    pub description: &'static str,
    /// Input port count policy.
    pub inputs: IOPorts,
    /// Output port count policy.
    pub outputs: IOPorts,
    /// Body shape.
    pub shape: SimulinkShape,
    /// Optional centre icon.
    pub icon: Option<SimulinkIcon>,
    /// How to label input ports.
    pub input_port_label: PortLabelPolicy,
    /// How to label output ports.
    pub output_port_label: PortLabelPolicy,
    /// How to produce the in-block label.
    pub block_label: BlockLabelPolicy,
    /// Interior renderer used when live mode is OFF.
    pub static_renderer: Option<StaticRendererFn>,
    /// Interior renderer used when live mode is ON.
    pub live_renderer: Option<LiveRendererFn>,
    /// Per-port position overrides.
    pub port_position_overrides: &'static [PortPositionOverride],
    /// Property keys copied verbatim from `block.properties` into metadata.
    pub metadata_keys: &'static [&'static str],
    /// Optional extra metadata computation hook.
    pub metadata_fn: Option<fn(&Block, &mut BlockMetadata)>,
    /// Optional per-instance label derived directly from the block (used by
    /// bridged virtual-library blocks that pre-date the metadata system).
    /// Consulted after [`BlockLabelPolicy`] when both are present.
    pub compute_instance_label: Option<fn(&Block) -> Option<String>>,
}

impl std::fmt::Debug for SimulinkBlockDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimulinkBlockDefinition")
            .field("block_type", &self.block_type)
            .field("aliases", &self.aliases)
            .field("category", &self.category)
            .field("inputs", &self.inputs)
            .field("outputs", &self.outputs)
            .field("shape", &self.shape)
            .field("icon", &self.icon)
            .field("input_port_label", &self.input_port_label)
            .field("output_port_label", &self.output_port_label)
            .field("block_label", &self.block_label)
            .field("static_renderer", &self.static_renderer.map(|_| "<fn>"))
            .field("live_renderer", &self.live_renderer.map(|_| "<fn>"))
            .finish()
    }
}

impl SimulinkBlockDefinition {
    /// Construct a minimal definition; refine with the `with_*` builders.
    pub const fn new(block_type: &'static str, category: &'static str) -> Self {
        Self {
            block_type,
            aliases: &[],
            category,
            description: "",
            inputs: IOPorts::None,
            outputs: IOPorts::None,
            shape: SimulinkShape::Rectangle,
            icon: None,
            input_port_label: PortLabelPolicy::None,
            output_port_label: PortLabelPolicy::None,
            block_label: BlockLabelPolicy::None,
            static_renderer: None,
            live_renderer: None,
            port_position_overrides: &[],
            metadata_keys: &[],
            metadata_fn: None,
            compute_instance_label: None,
        }
    }

    pub const fn with_aliases(mut self, aliases: &'static [&'static str]) -> Self {
        self.aliases = aliases;
        self
    }

    pub const fn with_description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    pub const fn with_ports(mut self, inputs: IOPorts, outputs: IOPorts) -> Self {
        self.inputs = inputs;
        self.outputs = outputs;
        self
    }

    pub const fn with_shape(mut self, shape: SimulinkShape) -> Self {
        self.shape = shape;
        self
    }

    pub const fn with_icon(mut self, icon: SimulinkIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub const fn with_port_labels(
        mut self,
        input: PortLabelPolicy,
        output: PortLabelPolicy,
    ) -> Self {
        self.input_port_label = input;
        self.output_port_label = output;
        self
    }

    pub const fn with_block_label(mut self, label: BlockLabelPolicy) -> Self {
        self.block_label = label;
        self
    }

    pub const fn with_static_renderer(mut self, f: StaticRendererFn) -> Self {
        self.static_renderer = Some(f);
        self
    }

    pub const fn with_live_renderer(mut self, f: LiveRendererFn) -> Self {
        self.live_renderer = Some(f);
        self
    }

    pub const fn with_port_overrides(mut self, overrides: &'static [PortPositionOverride]) -> Self {
        self.port_position_overrides = overrides;
        self
    }

    pub const fn with_metadata_keys(mut self, keys: &'static [&'static str]) -> Self {
        self.metadata_keys = keys;
        self
    }

    pub const fn with_metadata_fn(mut self, f: fn(&Block, &mut BlockMetadata)) -> Self {
        self.metadata_fn = Some(f);
        self
    }

    pub const fn with_instance_label(mut self, f: fn(&Block) -> Option<String>) -> Self {
        self.compute_instance_label = Some(f);
        self
    }
}

/// A named library that contributes block definitions to the catalog.
#[derive(Clone, Copy, Debug)]
pub struct SimulinkLibrary {
    /// Library name as it appears in SLX source-block paths (e.g. `"simulink"`).
    pub name: &'static str,
    /// Block definitions provided by this library.
    pub blocks: &'static [SimulinkBlockDefinition],
}

/// The fallback definition used for unrecognised block types.
pub fn unknown_block_definition() -> &'static SimulinkBlockDefinition {
    static UNKNOWN: SimulinkBlockDefinition = SimulinkBlockDefinition::new("Unknown", "Unknown");
    &UNKNOWN
}

/// A runtime registry mapping resolution keys to definitions.
///
/// Built once from all libraries; lookups are O(1).
#[derive(Default)]
pub struct DefinitionRegistry {
    pub(crate) by_key: HashMap<String, &'static SimulinkBlockDefinition>,
    pub(crate) all: Vec<&'static SimulinkBlockDefinition>,
}
