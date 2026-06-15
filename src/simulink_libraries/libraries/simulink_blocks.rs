//! Extended Simulink block definitions with icons, shapes, and metadata labels.
//!
//! Covers the full range of standard Simulink library blocks so they render
//! with meaningful icons instead of the `?` placeholder.  Registered before
//! the metadata-only palette so these richer definitions take priority.

#![cfg(feature = "egui")]

use crate::simulink_libraries::labels;
use crate::simulink_libraries::types::{
    BlockLabelPolicy, IOPorts, MetadataKey, SimulinkBlockDefinition, SimulinkIcon, SimulinkShape,
};

const fn icon(glyph: &'static str) -> SimulinkIcon {
    SimulinkIcon::Utf8(glyph)
}

#[rustfmt::skip]
pub static BLOCKS: &[SimulinkBlockDefinition] = &[
    // ═══════════════════════════════════════════════════════════════════════
    //  Continuous
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("Derivative", "Continuous")
        .with_description("Output the time derivative of the input")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("du/dt")),

    SimulinkBlockDefinition::new("Integrator", "Continuous")
        .with_description("Integrate input signal over time")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("1/s")),

    SimulinkBlockDefinition::new("SecondOrderIntegrator", "Continuous")
        .with_aliases(&["Second-Order Integrator"])
        .with_description("Integrate twice: acceleration to position")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(2))
        .with_icon(icon("1/s\u{00B2}")),

    SimulinkBlockDefinition::new("DescriptorStateSpace", "Continuous")
        .with_aliases(&["Descriptor State-Space"])
        .with_description("Descriptor state-space model E*dx = Ax + Bu")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_block_label(BlockLabelPolicy::Fixed("E\u{1E8B}=Ax+Bu")),

    // ═══════════════════════════════════════════════════════════════════════
    //  Discontinuities
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("Backlash", "Discontinuities")
        .with_description("Model backlash (dead-zone in a mechanical gear)")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{27F2}")),

    SimulinkBlockDefinition::new("Saturate", "Discontinuities")
        .with_aliases(&["Saturation"])
        .with_description("Limit input signal to upper and lower bounds")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("sat")),

    SimulinkBlockDefinition::new("CoulombViscousFriction", "Discontinuities")
        .with_aliases(&["Coulomb & Viscous Friction", "Coulomb"])
        .with_description("Coulomb and viscous friction model")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("fric")),

    // ═══════════════════════════════════════════════════════════════════════
    //  Discrete
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("Delay", "Discrete")
        .with_description("Delay input by variable number of sample periods")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("z\u{207B}\u{207F}")),

    SimulinkBlockDefinition::new("UnitDelay", "Discrete")
        .with_aliases(&["Unit Delay"])
        .with_description("Delay input by one sample period")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("1/z")),

    SimulinkBlockDefinition::new("Difference", "Discrete")
        .with_description("Compute difference between successive samples")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{0394}")),

    SimulinkBlockDefinition::new("DiscretePulseGenerator", "Sources")
        .with_aliases(&["Discrete Pulse Generator", "Pulse Generator"])
        .with_description("Generate discrete square-pulse signal")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("\u{2393}")),

    // ═══════════════════════════════════════════════════════════════════════
    //  Logic and Bit Operations
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("Logic", "Logic and Bit Operations")
        .with_aliases(&["Logical Operator"])
        .with_description("Perform logical operation (AND, OR, NOT, ...)")
        .with_ports(IOPorts::Variable(2), IOPorts::Fixed(1))
        .with_metadata_keys(&[MetadataKey::with_default("Operator", "AND")])
        .with_block_label(BlockLabelPolicy::MetadataDependent(labels::logic_operator)),

    SimulinkBlockDefinition::new("RelationalOperator", "Logic and Bit Operations")
        .with_aliases(&["Relational Operator"])
        .with_description("Compare two inputs (<=, >=, ==, ~=)")
        .with_ports(IOPorts::Fixed(2), IOPorts::Fixed(1))
        .with_metadata_keys(&[MetadataKey::with_default("Operator", "<=")])
        .with_block_label(BlockLabelPolicy::MetadataDependent(labels::relational_operator)),

    SimulinkBlockDefinition::new("BitClear", "Logic and Bit Operations")
        .with_aliases(&["Bit Clear"])
        .with_description("Clear specified bit of stored integer")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_block_label(BlockLabelPolicy::Fixed("Clear\nbit")),

    SimulinkBlockDefinition::new("BitSet", "Logic and Bit Operations")
        .with_aliases(&["Bit Set"])
        .with_description("Set specified bit of stored integer")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_block_label(BlockLabelPolicy::Fixed("Set\nbit")),

    SimulinkBlockDefinition::new("CompareToZero", "Logic and Bit Operations")
        .with_aliases(&["Compare To Zero", "Compare"])
        .with_description("Compare input signal to zero")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{2264}0")),

    SimulinkBlockDefinition::new("CompareToConstant", "Logic and Bit Operations")
        .with_aliases(&["Compare To Constant"])
        .with_description("Compare input signal to a constant")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{2264}K")),

    SimulinkBlockDefinition::new("DetectDecrease", "Logic and Bit Operations")
        .with_aliases(&["Detect Decrease"])
        .with_description("Detect decrease in signal value")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{2198}")),

    SimulinkBlockDefinition::new("DetectIncrease", "Logic and Bit Operations")
        .with_aliases(&["Detect Increase"])
        .with_description("Detect increase in signal value")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{2197}")),

    // ═══════════════════════════════════════════════════════════════════════
    //  Lookup Tables
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("Lookup_n-D", "Lookup Tables")
        .with_aliases(&["1-D Lookup Table", "2-D Lookup Table", "n-D Lookup Table"])
        .with_description("n-dimensional lookup table interpolation")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("T(u)")),

    SimulinkBlockDefinition::new("Cosine", "Lookup Tables")
        .with_description("Cosine function via lookup table")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("cos")),

    // ═══════════════════════════════════════════════════════════════════════
    //  Math Operations
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("Abs", "Math Operations")
        .with_description("Output absolute value of input")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("|u|")),

    SimulinkBlockDefinition::new("Bias", "Math Operations")
        .with_aliases(&["Add Constant"])
        .with_description("Add a bias (constant) to the input")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("u+K")),

    SimulinkBlockDefinition::new("DotProduct", "Math Operations")
        .with_aliases(&["Dot Product"])
        .with_description("Compute dot product of two vectors")
        .with_ports(IOPorts::Fixed(2), IOPorts::Fixed(1))
        .with_icon(icon("\u{2022}")),

    SimulinkBlockDefinition::new("Math", "Math Operations")
        .with_aliases(&["Math Function"])
        .with_description("Apply mathematical function (exp, log, sqrt, ...)")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_metadata_keys(&[MetadataKey::with_default("Operator", "exp")])
        .with_block_label(BlockLabelPolicy::MetadataDependent(labels::math_function)),

    SimulinkBlockDefinition::new("Trigonometry", "Math Operations")
        .with_aliases(&["Trigonometric Function"])
        .with_description("Trigonometric function (sin, cos, tan, acos, ...)")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_metadata_keys(&[MetadataKey::with_default("Operator", "sin")])
        .with_block_label(BlockLabelPolicy::MetadataDependent(labels::trig_function)),

    SimulinkBlockDefinition::new("MinMax", "Math Operations")
        .with_description("Output minimum or maximum of inputs")
        .with_ports(IOPorts::Variable(2), IOPorts::Fixed(1))
        .with_metadata_keys(&[MetadataKey::with_default("Function", "min")])
        .with_block_label(BlockLabelPolicy::MetadataDependent(labels::minmax_function)),

    SimulinkBlockDefinition::new("ComplexToMagnitudeAngle", "Math Operations")
        .with_aliases(&["Complex to Magnitude-Angle"])
        .with_description("Split complex signal to magnitude and angle")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(2))
        .with_icon(icon("|u|\u{2220}"))
        .with_port_labels(
            crate::simulink_libraries::types::PortLabelPolicy::None,
            crate::simulink_libraries::types::PortLabelPolicy::Fixed(&["Mag", "Ang"]),
        ),

    SimulinkBlockDefinition::new("MagnitudeAngleToComplex", "Math Operations")
        .with_aliases(&["Magnitude-Angle to Complex"])
        .with_description("Combine magnitude and angle into complex signal")
        .with_ports(IOPorts::Fixed(2), IOPorts::Fixed(1))
        .with_icon(icon("\u{2220}\u{2192}C"))
        .with_port_labels(
            crate::simulink_libraries::types::PortLabelPolicy::Fixed(&["Mag", "Ang"]),
            crate::simulink_libraries::types::PortLabelPolicy::None,
        ),

    SimulinkBlockDefinition::new("RealImagToComplex", "Math Operations")
        .with_aliases(&["Real-Imag to Complex"])
        .with_description("Combine real and imaginary into complex signal")
        .with_ports(IOPorts::Fixed(2), IOPorts::Fixed(1))
        .with_icon(icon("Re+jIm")),

    SimulinkBlockDefinition::new("AlgebraicConstraint", "Math Operations")
        .with_aliases(&["Algebraic Constraint"])
        .with_description("Solve algebraic loop: f(z) = 0")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_block_label(BlockLabelPolicy::Fixed("f(z)=0")),

    SimulinkBlockDefinition::new("MinMaxRunningResettable", "Math Operations")
        .with_aliases(&["MinMax Running Resettable"])
        .with_description("Running min/max with external reset")
        .with_ports(IOPorts::Fixed(2), IOPorts::Fixed(1))
        .with_icon(icon("min(u,y)")),

    SimulinkBlockDefinition::new("Sin", "Sources")
        .with_aliases(&["Sine Wave"])
        .with_description("Generate sine wave using internal time source")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("\u{223F}")),

    // ═══════════════════════════════════════════════════════════════════════
    //  Matrix Operations  (bridged virtual library fills most; hermitian gap)
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("IsHermitian", "Matrix Operations")
        .with_aliases(&["Is Hermitian"])
        .with_description("Test whether matrix is Hermitian")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("Aᴴ")),

    // ═══════════════════════════════════════════════════════════════════════
    //  Model Verification / Testing
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("Assertion", "Testing & Verification")
        .with_description("Assert that input is nonzero")
        .with_ports(IOPorts::Fixed(1), IOPorts::None)
        .with_icon(icon("\u{2713}")),

    SimulinkBlockDefinition::new("CheckDynamicRange", "Testing & Verification")
        .with_aliases(&["Check Dynamic Range"])
        .with_description("Verify signal stays within dynamic range")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{2713}")),

    SimulinkBlockDefinition::new("CheckStaticGap", "Testing & Verification")
        .with_aliases(&["Check Static Gap"])
        .with_description("Verify no static gap in signal")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{2713}")),

    SimulinkBlockDefinition::new("CheckStaticRange", "Testing & Verification")
        .with_aliases(&["Check Static Range"])
        .with_description("Verify signal stays within static range")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{2713}")),

    SimulinkBlockDefinition::new("CheckDynamicGap", "Testing & Verification")
        .with_aliases(&["Check Dynamic Gap"])
        .with_description("Verify no dynamic gap in signal")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{2713}")),

    SimulinkBlockDefinition::new("CheckDiscreteGradient", "Testing & Verification")
        .with_aliases(&["Check Discrete Gradient"])
        .with_description("Verify discrete gradient within bounds")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{2713}")),

    SimulinkBlockDefinition::new("CheckDynamicLowerBound", "Testing & Verification")
        .with_aliases(&["Check Dynamic Lower Bound"])
        .with_description("Verify signal above dynamic lower bound")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{2713}")),

    SimulinkBlockDefinition::new("CheckDynamicUpperBound", "Testing & Verification")
        .with_aliases(&["Check Dynamic Upper Bound"])
        .with_description("Verify signal below dynamic upper bound")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{2713}")),

    SimulinkBlockDefinition::new("CheckInputResolution", "Testing & Verification")
        .with_aliases(&["Check Input Resolution"])
        .with_description("Verify signal resolution meets requirement")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{2713}")),

    SimulinkBlockDefinition::new("CheckStaticLowerBound", "Testing & Verification")
        .with_aliases(&["Check Static Lower Bound"])
        .with_description("Verify signal above static lower bound")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{2713}")),

    SimulinkBlockDefinition::new("CheckStaticUpperBound", "Testing & Verification")
        .with_aliases(&["Check Static Upper Bound"])
        .with_description("Verify signal below static upper bound")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{2713}")),

    // ═══════════════════════════════════════════════════════════════════════
    //  Ports & Subsystems
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("EnablePort", "Ports & Subsystems")
        .with_aliases(&["Enable"])
        .with_description("Add enable port to subsystem")
        .with_ports(IOPorts::None, IOPorts::None)
        .with_icon(icon("EN")),

    SimulinkBlockDefinition::new("ForIterator", "Ports & Subsystems")
        .with_aliases(&["For Iterator"])
        .with_description("Repeat subsystem execution a specified number of times")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("for")),

    SimulinkBlockDefinition::new("ForEach", "Ports & Subsystems")
        .with_aliases(&["For Each"])
        .with_description("Partition input and apply subsystem to each element")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{2200}")),

    SimulinkBlockDefinition::new("TriggerPort", "Ports & Subsystems")
        .with_aliases(&["Trigger"])
        .with_description("Add trigger port to subsystem")
        .with_ports(IOPorts::None, IOPorts::None)
        .with_icon(icon("\u{2191}")),

    SimulinkBlockDefinition::new("PMIOPort", "Ports & Subsystems")
        .with_aliases(&["Connection Port", "Simscape Port"])
        .with_description("Physical modeling connection port")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{21C6}")),

    // ═══════════════════════════════════════════════════════════════════════
    //  Signal Attributes
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("DataTypeConversion", "Signal Attributes")
        .with_aliases(&["Data Type Conversion"])
        .with_description("Convert signal to specified data type")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_block_label(BlockLabelPolicy::Fixed("convert")),

    SimulinkBlockDefinition::new("Width", "Signal Attributes")
        .with_description("Output width (number of elements) of input signal")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("W")),

    SimulinkBlockDefinition::new("SignalConversion", "Signal Routing")
        .with_aliases(&["Signal Conversion"])
        .with_description("Convert between signal types")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("\u{21D2}")),

    // ═══════════════════════════════════════════════════════════════════════
    //  Signal Routing
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("BusAssignment", "Signal Routing")
        .with_aliases(&["Bus Assignment"])
        .with_description("Assign signals to a bus")
        .with_ports(IOPorts::Fixed(2), IOPorts::Fixed(1))
        .with_shape(SimulinkShape::FilledBlack),

    SimulinkBlockDefinition::new("BusToVector", "Signal Routing")
        .with_aliases(&["Bus to Vector"])
        .with_description("Convert bus to a vector signal")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("B\u{2192}V")),

    SimulinkBlockDefinition::new("GotoTagVisibility", "Signal Routing")
        .with_aliases(&["Goto Tag Visibility"])
        .with_description("Define scope of Goto tag visibility")
        .with_ports(IOPorts::None, IOPorts::None)
        .with_icon(icon("[.]")),

    SimulinkBlockDefinition::new("Merge", "Signal Routing")
        .with_description("Merge multiple signals into single output")
        .with_ports(IOPorts::Variable(2), IOPorts::Fixed(1))
        .with_block_label(BlockLabelPolicy::Fixed("merge")),

    SimulinkBlockDefinition::new("MultiPortSwitch", "Signal Routing")
        .with_aliases(&["Multiport Switch"])
        .with_description("Select one of N inputs based on control signal")
        .with_ports(IOPorts::Variable(3), IOPorts::Fixed(1))
        .with_icon(icon("\u{21C5}")),

    SimulinkBlockDefinition::new("Selector", "Signal Routing")
        .with_description("Select input elements from a vector/matrix")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("sel")),

    SimulinkBlockDefinition::new("Switch", "Signal Routing")
        .with_description("Switch between two inputs based on threshold")
        .with_ports(IOPorts::Fixed(3), IOPorts::Fixed(1))
        .with_icon(icon("\u{2277}")),

    // ═══════════════════════════════════════════════════════════════════════
    //  Sinks
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("Record", "Sinks")
        .with_aliases(&["XY Graph", "To Workspace"])
        .with_description("Record signal data")
        .with_ports(IOPorts::Variable(1), IOPorts::None)
        .with_icon(icon("rec")),

    // ═══════════════════════════════════════════════════════════════════════
    //  Sources
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("Clock", "Sources")
        .with_description("Output continuous simulation time")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("clk")),

    SimulinkBlockDefinition::new("DigitalClock", "Sources")
        .with_aliases(&["Digital Clock"])
        .with_description("Output simulation time at specified sample rate")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("12:34")),

    SimulinkBlockDefinition::new("Ground", "Sources")
        .with_description("Output zero-valued signal")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("\u{22A5}")),

    SimulinkBlockDefinition::new("RandomNumber", "Sources")
        .with_aliases(&["Random Number"])
        .with_description("Generate normally distributed random numbers")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("rand")),

    SimulinkBlockDefinition::new("UniformRandomNumber", "Sources")
        .with_aliases(&["Uniform Random Number"])
        .with_description("Generate uniformly distributed random numbers")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("U~")),

    SimulinkBlockDefinition::new("SignalGenerator", "Sources")
        .with_aliases(&["Signal Generator"])
        .with_description("Generate various waveforms (sine, square, sawtooth)")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("\u{223F}")),

    SimulinkBlockDefinition::new("Step", "Sources")
        .with_description("Generate step function signal")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("\u{2310}")),

    SimulinkBlockDefinition::new("Ramp", "Sources")
        .with_description("Generate ramp signal")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("\u{2571}")),

    SimulinkBlockDefinition::new("BandLimitedWhiteNoise", "Sources")
        .with_aliases(&["Band-Limited White Noise"])
        .with_description("White noise with specified bandwidth")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("noise")),

    SimulinkBlockDefinition::new("Chirp", "Sources")
        .with_aliases(&["Chirp Signal"])
        .with_description("Generate frequency-swept sinusoidal signal")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("chirp")),

    SimulinkBlockDefinition::new("Counter", "Sources")
        .with_aliases(&["Counter Free-Running"])
        .with_description("Free-running counter output")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("n++")),

    SimulinkBlockDefinition::new("CounterLimited", "Sources")
        .with_aliases(&["Counter Limited"])
        .with_description("Counter with configurable upper limit")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("n++")),

    SimulinkBlockDefinition::new("Repeating", "Sources")
        .with_aliases(&["Repeating Sequence"])
        .with_description("Generate repeating arbitrary signal")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("\u{21BB}")),

    SimulinkBlockDefinition::new("RepeatingInterp", "Sources")
        .with_aliases(&["Repeating Sequence Interpolated"])
        .with_description("Repeating sequence with interpolation")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("\u{21BB}")),

    SimulinkBlockDefinition::new("RepeatingStair", "Sources")
        .with_aliases(&["Repeating Sequence Stair"])
        .with_description("Generate repeating staircase signal")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("\u{21BB}")),

    SimulinkBlockDefinition::new("WaveformGenerator", "Sources")
        .with_aliases(&["Waveform Generator"])
        .with_description("Generate waveform from stored table")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("\u{223F}")),

    // ═══════════════════════════════════════════════════════════════════════
    //  String Operations
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("ASCIIToString", "String Operations")
        .with_aliases(&["ASCII to String"])
        .with_description("Convert ASCII codes to string")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_block_label(BlockLabelPolicy::Fixed("A\u{2192}str")),

    SimulinkBlockDefinition::new("ToString", "String Operations")
        .with_aliases(&["To String"])
        .with_description("Convert input to string representation")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_block_label(BlockLabelPolicy::Fixed("\u{2192}str")),

    SimulinkBlockDefinition::new("StringConstant", "String Operations")
        .with_aliases(&["String Constant"])
        .with_description("Output a constant string value")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("\"...\"")),

    // ═══════════════════════════════════════════════════════════════════════
    //  User-Defined Functions
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("Fcn", "User-Defined Functions")
        .with_description("Apply user-specified expression: y = f(u)")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(icon("f(u)")),

    SimulinkBlockDefinition::new("S-Function", "User-Defined Functions")
        .with_aliases(&["S-Function Builder", "Level-2 MATLAB S-Function"])
        .with_description("S-Function (system function) block")
        .with_ports(IOPorts::Variable(1), IOPorts::Variable(1))
        .with_icon(icon("S-fn")),

    SimulinkBlockDefinition::new("CustomCallbackButton", "Dashboard")
        .with_aliases(&["Callback Button"])
        .with_description("Dashboard callback button")
        .with_ports(IOPorts::None, IOPorts::None)
        .with_block_label(BlockLabelPolicy::Fixed("Button")),

    // ═══════════════════════════════════════════════════════════════════════
    //  Timing & Scheduling / Advanced
    // ═══════════════════════════════════════════════════════════════════════
    SimulinkBlockDefinition::new("EventListener", "Ports & Subsystems")
        .with_aliases(&["Event Listener"])
        .with_description("Listen for simulation events")
        .with_ports(IOPorts::None, IOPorts::None)
        .with_icon(icon("evt")),

    SimulinkBlockDefinition::new("StateReader", "Ports & Subsystems")
        .with_aliases(&["State Reader"])
        .with_description("Read block state for logging or initialisation")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(icon("SR")),

    SimulinkBlockDefinition::new("StateWriter", "Ports & Subsystems")
        .with_aliases(&["State Writer"])
        .with_description("Write values into block state")
        .with_ports(IOPorts::Fixed(1), IOPorts::None)
        .with_icon(icon("SW")),
];
