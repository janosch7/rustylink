//! Feature-independent structural facts about Simulink block types.
//!
//! This is the core (non-`egui`) counterpart to the rich rendering definitions
//! in [`super::libraries`]: the parser, the model and the signal resolver need
//! to know *what a block does* without pulling in a painter.  Every such fact
//! is declared here, so the code outside `simulink_libraries` never has to name
//! a block type.
//!
//! Adding behaviour for a new block type means extending the tables below – no
//! `block.block_type == "…"` anywhere else.

use crate::model::Block;

/// What a block does to the signals that pass through it.
///
/// The signal resolver ([`crate::connection_targets`]) dispatches on this role
/// instead of on the block type, so the type → behaviour mapping stays in the
/// catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum SignalRole {
    /// Nothing special: the block's outputs are its own.
    #[default]
    Plain,
    /// Bundles its inputs into a bus (`BusCreator`).
    BusCreator,
    /// Picks elements out of a bus (`BusSelector`).
    BusSelector,
    /// Replaces elements of a bus (`BusAssignment`).
    BusAssignment,
    /// Bundles its inputs into a vector (`Mux`).
    Mux,
    /// Splits a vector into its elements (`Demux`).
    Demux,
    /// The inside end of a parent system's input port.
    BoundaryInput,
    /// The inside end of a parent system's output port.
    BoundaryOutput,
    /// Holds a child system whose boundaries continue the signal.
    Container,
    /// Receives the signal of the matching [`SignalRole::Goto`] tag.
    From,
    /// Sends its input to the matching [`SignalRole::From`] tags.
    Goto,
    /// Enable/trigger/reset port: fed by the parent's control signal.
    ControlPort,
    /// Passes the active variant's input through (`VariantStart`,
    /// `VariantSink`).
    VariantSelect,
    /// Emits the active variant's branch (`VariantEnd`, `VariantSource`).
    VariantMerge,
}

impl SignalRole {
    /// Whether a line leaving this block carries the line's own metadata
    /// (signal name, test point) on top of the propagated targets.
    pub const fn propagates_local_metadata(self) -> bool {
        !matches!(self, SignalRole::Plain | SignalRole::Goto)
    }

    /// Whether a line *ending* at this block leaves the current system, so
    /// metadata travelling back upstream crosses a system boundary.
    pub const fn crosses_system_boundary(self) -> bool {
        matches!(self, SignalRole::Container | SignalRole::BoundaryOutput)
    }
}

/// The kind of source code a block carries, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CodeKind {
    /// MATLAB code (`MATLAB Function`, `MATLABSystem`, `MATLABFcn`).
    Matlab,
    /// A single-line expression (`Fcn`).
    Expression,
    /// C code (`CFunction`).
    C,
}

/// How a block presents live simulation data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LiveRole {
    /// Prints the current value as text (`Display`).
    Value,
    /// Plots the signal over time (`Scope`, `DashboardScope`).
    Trace,
    /// The user drives the value from the canvas (`ManualSwitch`, `Constant`).
    Input,
}

/// The structural facts about one block type.
#[derive(Clone, Copy, Debug, Default)]
pub struct BlockTraits {
    /// What the block does to signals passing through it.
    pub signal_role: SignalRole,
    /// Whether the block owns a child system the user can navigate into.
    pub container: bool,
    /// Whether paths below this block are matched by prefix rather than
    /// exactly (library references resolve into a file of their own).
    pub path_prefix_matched: bool,
    /// The source code the block carries, if any.
    pub code: Option<CodeKind>,
    /// How the block shows live data, if at all.
    pub live: Option<LiveRole>,
    /// Value assumed when the model omits the block's `Value` property.
    pub implicit_value: Option<&'static str>,
    /// Port counts a dashboard block gets when the model states none.
    pub dashboard_ports: Option<(u32, u32)>,
}

impl BlockTraits {
    const PLAIN: Self = Self {
        signal_role: SignalRole::Plain,
        container: false,
        path_prefix_matched: false,
        code: None,
        live: None,
        implicit_value: None,
        dashboard_ports: None,
    };

    const fn signal(role: SignalRole) -> Self {
        Self {
            signal_role: role,
            ..Self::PLAIN
        }
    }
}

/// The structural facts about `block_type`.
///
/// Unknown types are [`BlockTraits::PLAIN`]: a block with no special
/// behaviour, which is what the great majority of Simulink blocks are.
pub fn block_traits(block_type: &str) -> BlockTraits {
    match block_type {
        "BusCreator" => BlockTraits::signal(SignalRole::BusCreator),
        "BusSelector" => BlockTraits::signal(SignalRole::BusSelector),
        "BusAssignment" => BlockTraits::signal(SignalRole::BusAssignment),
        "Mux" => BlockTraits::signal(SignalRole::Mux),
        "Demux" => BlockTraits::signal(SignalRole::Demux),
        "Inport" | "InportShadow" => BlockTraits::signal(SignalRole::BoundaryInput),
        "Outport" => BlockTraits::signal(SignalRole::BoundaryOutput),
        "From" => BlockTraits::signal(SignalRole::From),
        "Goto" => BlockTraits::signal(SignalRole::Goto),
        "EnablePort" | "TriggerPort" | "ResetPort" => BlockTraits::signal(SignalRole::ControlPort),
        "VariantStart" | "VariantSink" => BlockTraits::signal(SignalRole::VariantSelect),
        "VariantEnd" | "VariantSource" => BlockTraits::signal(SignalRole::VariantMerge),
        "SubSystem" => BlockTraits {
            signal_role: SignalRole::Container,
            container: true,
            ..BlockTraits::PLAIN
        },
        "Reference" => BlockTraits {
            signal_role: SignalRole::Container,
            container: true,
            path_prefix_matched: true,
            ..BlockTraits::PLAIN
        },
        "MATLAB Function" | "MATLABSystem" | "MATLABFcn" => BlockTraits {
            code: Some(CodeKind::Matlab),
            ..BlockTraits::PLAIN
        },
        "Fcn" => BlockTraits {
            code: Some(CodeKind::Expression),
            ..BlockTraits::PLAIN
        },
        "CFunction" => BlockTraits {
            code: Some(CodeKind::C),
            ..BlockTraits::PLAIN
        },
        "Constant" => BlockTraits {
            live: Some(LiveRole::Input),
            implicit_value: Some("1"),
            ..BlockTraits::PLAIN
        },
        "ManualSwitch" => BlockTraits {
            live: Some(LiveRole::Input),
            ..BlockTraits::PLAIN
        },
        "Display" => BlockTraits {
            live: Some(LiveRole::Value),
            dashboard_ports: Some((1, 0)),
            ..BlockTraits::PLAIN
        },
        "Scope" | "DashboardScope" => BlockTraits {
            live: Some(LiveRole::Trace),
            dashboard_ports: Some((0, 0)),
            ..BlockTraits::PLAIN
        },
        _ => BlockTraits::PLAIN,
    }
}

/// The facts about this block instance, including the ones that depend on its
/// properties rather than only on its type.
pub fn traits_of(block: &Block) -> BlockTraits {
    let mut traits = block_traits(&block.block_type);
    if block.is_matlab_function {
        traits.code = Some(CodeKind::Matlab);
    }
    traits
}

/// Whether the block owns a child system the user can navigate into.
pub fn is_container(block: &Block) -> bool {
    block_traits(&block.block_type).container
}

/// Whether the block runs MATLAB code the user can open in an editor.
pub fn is_matlab_function(block: &Block) -> bool {
    traits_of(block).code == Some(CodeKind::Matlab)
}

/// Default port counts for a dashboard block the model gives no ports for.
pub fn dashboard_port_counts(block_type: &str) -> (u32, u32) {
    block_traits(block_type).dashboard_ports.unwrap_or((0, 0))
}

/// The block types the SLX scanner recognises, used to report the ones this
/// crate does not model yet.
pub const KNOWN_BLOCK_TYPES: &[&str] = &[
    "Abs",
    "ActionPort",
    "BusAssignment",
    "BusCreator",
    "BusElement",
    "BusSelector",
    "BusToVector",
    "CompareToConstant",
    "CompareToZero",
    "Constant",
    "DataStoreMemory",
    "DataStoreRead",
    "DataStoreWrite",
    "Demux",
    "DiscreteFilter",
    "DiscreteStateSpace",
    "DiscreteTransferFcn",
    "Display",
    "EnabledSubsystem",
    "Fcn",
    "ForEach",
    "ForEachSubsystem",
    "From",
    "Gain",
    "Goto",
    "If",
    "IfActionSubsystem",
    "Inport",
    "InportShadow",
    "Integrator",
    "LogicalOperator",
    "Lookup",
    "Lookup_n-D",
    "MATLABFcn",
    "Max",
    "MaxMin",
    "Merge",
    "Min",
    "MinMax",
    "ModelReference",
    "MultiPortSwitch",
    "Mux",
    "Outport",
    "Product",
    "PulseGenerator",
    "Ramp",
    "RandomNumber",
    "RateTransition",
    "RelationalOperator",
    "RepeatingSequence",
    "RepeatingSequenceRamp",
    "RepeatingSequenceStair",
    "S-Function",
    "Saturate",
    "Scope",
    "Selector",
    "SignalConversion",
    "SineWave",
    "Sqrt",
    "StateSpace",
    "Step",
    "SubSystem",
    "Sum",
    "SumOfElements",
    "Switch",
    "TransferFcn",
    "TriggeredDelay",
    "TriggeredFromWorkspace",
    "TriggeredReadFromFile",
    "TriggeredSampleAndHold",
    "TriggeredSubsystem",
    "TriggeredToWorkspace",
    "TriggeredWriteToFile",
    "UniformRandomNumber",
    "UnitDelay",
    "VariantEnd",
    "VariantSink",
    "VariantSource",
    "VariantStart",
    "VectorToBus",
    "WhileIterator",
    "WhileSubsystem",
    "ZeroOrderHold",
];

/// Whether the SLX scanner models `block_type`.
pub fn is_known_block_type(block_type: &str) -> bool {
    KNOWN_BLOCK_TYPES.binary_search(&block_type).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_block_types_are_sorted_for_binary_search() {
        let mut sorted = KNOWN_BLOCK_TYPES.to_vec();
        sorted.sort_unstable();
        assert_eq!(sorted, KNOWN_BLOCK_TYPES);
        assert!(is_known_block_type("Gain"));
        assert!(!is_known_block_type("NotABlock"));
    }

    #[test]
    fn roles_describe_signal_behaviour() {
        assert_eq!(block_traits("Mux").signal_role, SignalRole::Mux);
        assert!(block_traits("Reference").path_prefix_matched);
        assert!(!block_traits("SubSystem").path_prefix_matched);
        assert!(SignalRole::Container.crosses_system_boundary());
        assert!(!SignalRole::Mux.crosses_system_boundary());
        assert!(!SignalRole::Plain.propagates_local_metadata());
    }
}
