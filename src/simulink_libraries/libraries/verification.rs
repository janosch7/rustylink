//! Model verification blocks: assertions and range checks.
//!
//! Part of the block catalog: one file per Simulink library group.

#![cfg(feature = "egui")]

use super::prelude::{check_axes, plot};
use crate::simulink_libraries::types::{IOPorts, PortLabelPolicy, SimulinkBlockDefinition};

#[rustfmt::skip]
pub static BLOCKS: &[SimulinkBlockDefinition] = &[
    // ═══════════════════════════════════════════════════════════════════════
    //  Model Verification / Testing
    // ═══════════════════════════════════════════════════════════════════════
    // Simulink draws each verification block as a miniature plot: the signal
    // under test against the grey band of the region it is checked against,
    // with the bound inputs labelled `max`/`min` and the signal `u`.
    SimulinkBlockDefinition::new("Assertion", "Testing & Verification")
        .with_description("Assert that input is nonzero")
        .with_ports(IOPorts::Fixed(1), IOPorts::None)
        .with_icon(plot(concat!(
            "p 0.02,0.50 0.30,0.50;",
            "c 0.62,0.50,0.30;",
            "p 0.50,0.52 0.59,0.64 0.75,0.34"
        ))),
    SimulinkBlockDefinition::new("CheckDynamicRange", "Testing & Verification")
        .with_aliases(&["Check Dynamic Range"])
        .with_description("Verify signal stays within dynamic range")
        .with_ports(IOPorts::Fixed(3), IOPorts::None)
        .with_port_labels(
            PortLabelPolicy::Fixed(&["max", "min", "u"]),
            PortLabelPolicy::None,
        )
        .with_icon(plot(concat!(
            check_axes!(),
            "pf 0,0,0,46 0.9600,0.2400 0.3508,0.2400 0.3508,0.3333 0.3150,0.3333 0.3150,0.4267 0.1000,0.4267 0.1000,0.1467 0.9600,0.1467;",
            "pf 0,0,0,46 0.9600,0.5200 0.7808,0.5200 0.7808,0.6600 0.5658,0.6600 0.5658,0.4967 0.1000,0.4967 0.1000,0.7533 0.9600,0.7533;",
            "p 0.1000,0.4267 0.3150,0.4267 0.3150,0.3333 0.3508,0.3333 0.3508,0.2400 0.9600,0.2400;",
            "p 0.1000,0.4967 0.5658,0.4967 0.5658,0.6600 0.7808,0.6600 0.7808,0.5200 0.9600,0.5200;",
            "bc 0.1000,0.4580 0.1172,0.4176 0.1344,0.4176 0.1516,0.4580 0.1688,0.4984 0.1860,0.4984 0.2032,0.4580 0.2204,0.4176 0.2376,0.4256 0.2548,0.4580 0.2892,0.5388 0.3236,0.3771 0.3752,0.2963 0.4268,0.2155 0.4784,0.2155 0.5300,0.3367 0.5816,0.4580 0.5988,0.6600 0.6504,0.6600 0.7020,0.6600 0.7192,0.4580 0.7536,0.3367 0.7880,0.2155 0.8224,0.2155 0.8568,0.2963 0.8843,0.3771 0.8912,0.4378 0.9084,0.4580 0.9256,0.4984 0.9428,0.4984 0.9600,0.4580"
        ))),
    SimulinkBlockDefinition::new("CheckStaticGap", "Testing & Verification")
        .with_aliases(&["Check Static Gap"])
        .with_description("Verify no static gap in signal")
        .with_ports(IOPorts::Fixed(1), IOPorts::None)
        .with_icon(plot(concat!(
            check_axes!(),
            "b 0.1000,0.3800 0.9600,0.5200;",
            "p 0.1000,0.3800 0.9600,0.3800;",
            "p 0.1000,0.5200 0.9600,0.5200;",
            "bc 0.1000,0.3800 0.2344,0.0378 0.3956,0.1078 0.5300,0.4500 0.6644,0.7922 0.8256,0.8622 0.9600,0.5200"
        ))),
    SimulinkBlockDefinition::new("CheckStaticRange", "Testing & Verification")
        .with_aliases(&["Check Static Range"])
        .with_description("Verify signal stays within static range")
        .with_ports(IOPorts::Fixed(1), IOPorts::None)
        .with_icon(plot(concat!(
            check_axes!(),
            "b 0.1000,0.1467 0.9600,0.3333;",
            "b 0.1000,0.5667 0.9600,0.7533;",
            "p 0.1000,0.3333 0.9600,0.3333;",
            "p 0.1000,0.5667 0.9600,0.5667;",
            "bc 0.1000,0.4500 0.1537,0.2944 0.1896,0.2944 0.2433,0.4500 0.2971,0.6056 0.3329,0.6056 0.3867,0.4500 0.4404,0.2944 0.4762,0.2944 0.5300,0.4500 0.5837,0.6056 0.6196,0.6056 0.6733,0.4500 0.7271,0.2944 0.7629,0.2944 0.8167,0.4500 0.8704,0.6056 0.9062,0.6056 0.9600,0.4500"
        ))),
    SimulinkBlockDefinition::new("CheckDynamicGap", "Testing & Verification")
        .with_aliases(&["Check Dynamic Gap"])
        .with_description("Verify no dynamic gap in signal")
        .with_ports(IOPorts::Fixed(3), IOPorts::None)
        .with_port_labels(
            PortLabelPolicy::Fixed(&["max", "min", "u"]),
            PortLabelPolicy::None,
        )
        .with_icon(plot(concat!(
            check_axes!(),
            "pf 0,0,0,46 0.1000,0.3147 0.4225,0.3147 0.4225,0.3520 0.6375,0.3520 0.6375,0.3800 0.9600,0.3800 0.9600,0.4733 0.6375,0.4733 0.6375,0.5013 0.4225,0.5013 0.4225,0.5387 0.1000,0.5387;",
            "p 0.1000,0.3147 0.4225,0.3147 0.4225,0.3520 0.6375,0.3520 0.6375,0.3800 0.9600,0.3800;",
            "p 0.1000,0.5387 0.4225,0.5387 0.4225,0.5013 0.6375,0.5013 0.6375,0.4733 0.9600,0.4733;",
            "bc 0.9600,0.6410 0.9374,0.5535 0.8921,0.4733 0.8355,0.4733 0.7789,0.4733 0.7337,0.5535 0.6771,0.6119 0.6205,0.6702 0.5753,0.7067 0.5187,0.7067 0.4621,0.7067 0.4168,0.6702 0.3603,0.6119 0.3037,0.5535 0.2584,0.5171 0.2018,0.5535 0.1453,0.5900 0.1226,0.6556 0.1000,0.6994;",
            "bc 0.9600,0.2123 0.9374,0.2998 0.8921,0.3800 0.8355,0.3800 0.7789,0.3800 0.7337,0.2998 0.6771,0.2415 0.6205,0.1831 0.5753,0.1467 0.5187,0.1467 0.4621,0.1467 0.4168,0.1831 0.3603,0.2415 0.3037,0.2998 0.2584,0.3362 0.2018,0.2998 0.1453,0.2633 0.1226,0.1977 0.1000,0.1540"
        ))),
    SimulinkBlockDefinition::new("CheckDiscreteGradient", "Testing & Verification")
        .with_aliases(&["Check Discrete Gradient"])
        .with_description("Verify discrete gradient within bounds")
        .with_ports(IOPorts::Fixed(1), IOPorts::None)
        .with_icon(plot(concat!(
            check_axes!(),
            "pf 0,0,0,46 0.4583,0.6600 0.6017,0.6600 0.6017,0.3070;",
            "bc 0.1000,0.4733 0.1717,0.2867 0.2433,0.2867 0.3150,0.4733 0.3867,0.6600 0.4583,0.6600 0.5300,0.4733 0.6017,0.2867 0.6733,0.2867 0.7450,0.4733 0.8167,0.6600 0.8883,0.6600 0.9600,0.4733;",
            "p 0.4133,0.7642 0.6467,0.1933"
        ))),
    SimulinkBlockDefinition::new("CheckDynamicLowerBound", "Testing & Verification")
        .with_aliases(&["Check Dynamic Lower Bound"])
        .with_description("Verify signal above dynamic lower bound")
        .with_ports(IOPorts::Fixed(2), IOPorts::None)
        .with_port_labels(PortLabelPolicy::Fixed(&["min", "u"]), PortLabelPolicy::None)
        .with_icon(plot(concat!(
            check_axes!(),
            "pf 0,0,0,46 0.1000,0.6133 0.3867,0.6133 0.3867,0.5667 0.6017,0.5667 0.6017,0.5200 0.9600,0.5200 0.9600,0.7533 0.1000,0.7533;",
            "p 0.1000,0.6133 0.3867,0.6133 0.3867,0.5667 0.6017,0.5667 0.6017,0.5200 0.9600,0.5200;",
            "bc 0.1000,0.3626 0.1217,0.4934 0.1651,0.6133 0.2193,0.6133 0.2735,0.6133 0.3169,0.4934 0.3711,0.4062 0.4253,0.3190 0.4687,0.2645 0.5229,0.2645 0.5771,0.2645 0.6205,0.3190 0.6747,0.4062 0.7290,0.4934 0.7723,0.5479 0.8266,0.4934 0.8808,0.4389 0.9025,0.3408 0.9242,0.2754"
        ))),
    SimulinkBlockDefinition::new("CheckDynamicUpperBound", "Testing & Verification")
        .with_aliases(&["Check Dynamic Upper Bound"])
        .with_description("Verify signal below dynamic upper bound")
        .with_ports(IOPorts::Fixed(2), IOPorts::None)
        .with_port_labels(PortLabelPolicy::Fixed(&["max", "u"]), PortLabelPolicy::None)
        .with_icon(plot(concat!(
            check_axes!(),
            "pf 0,0,0,46 0.1000,0.2867 0.3867,0.2867 0.3867,0.3333 0.6017,0.3333 0.6017,0.3800 0.9600,0.3800 0.9600,0.1467 0.1000,0.1467;",
            "p 0.1000,0.2867 0.3867,0.2867 0.3867,0.3333 0.6017,0.3333 0.6017,0.3800 0.9600,0.3800;",
            "bc 0.1000,0.5374 0.1217,0.4066 0.1651,0.2867 0.2193,0.2867 0.2735,0.2867 0.3169,0.4066 0.3711,0.4938 0.4253,0.5810 0.4687,0.6355 0.5229,0.6355 0.5771,0.6355 0.6205,0.5810 0.6747,0.4938 0.7290,0.4066 0.7723,0.3521 0.8266,0.4066 0.8808,0.4611 0.9025,0.5592 0.9242,0.6246"
        ))),
    SimulinkBlockDefinition::new("CheckInputResolution", "Testing & Verification")
        .with_aliases(&["Check Input Resolution"])
        .with_description("Verify signal resolution meets requirement")
        .with_ports(IOPorts::Fixed(1), IOPorts::None)
        .with_icon(plot(concat!(
            check_axes!(),
            "p 0.1009,0.7054 0.2433,0.7067 0.2433,0.6133 0.3867,0.6133 0.3867,0.5200 0.5300,0.5200 0.5300,0.4267 0.6733,0.4267 0.6733,0.3333 0.8167,0.3333 0.8167,0.4267 0.9600,0.4267"
        ))),
    SimulinkBlockDefinition::new("CheckStaticLowerBound", "Testing & Verification")
        .with_aliases(&["Check Static Lower Bound"])
        .with_description("Verify signal above static lower bound")
        .with_ports(IOPorts::Fixed(1), IOPorts::None)
        .with_icon(plot(concat!(
            check_axes!(),
            "b 0.1000,0.4733 0.9600,0.7533;",
            "p 0.1000,0.4733 0.9600,0.4733;",
            "bc 0.1000,0.3683 0.1537,0.2283 0.1896,0.2283 0.2433,0.3683 0.2971,0.5083 0.3329,0.5083 0.3867,0.3683 0.4404,0.2283 0.4762,0.2283 0.5300,0.3683 0.5837,0.5083 0.6196,0.5083 0.6733,0.3683 0.7271,0.2283 0.7629,0.2283 0.8167,0.3683 0.8704,0.5083 0.9062,0.5083 0.9600,0.3683"
        ))),
    SimulinkBlockDefinition::new("CheckStaticUpperBound", "Testing & Verification")
        .with_aliases(&["Check Static Upper Bound"])
        .with_description("Verify signal below static upper bound")
        .with_ports(IOPorts::Fixed(1), IOPorts::None)
        .with_icon(plot(concat!(
            check_axes!(),
            "b 0.1000,0.1467 0.9600,0.4267;",
            "p 0.1000,0.4267 0.9600,0.4267;",
            "bc 0.1000,0.5317 0.1537,0.3917 0.1896,0.3917 0.2433,0.5317 0.2971,0.6717 0.3329,0.6717 0.3867,0.5317 0.4404,0.3917 0.4762,0.3917 0.5300,0.5317 0.5837,0.6717 0.6196,0.6717 0.6733,0.5317 0.7271,0.3917 0.7629,0.3917 0.8167,0.5317 0.8704,0.6717 0.9062,0.6717 0.9600,0.5317"
        ))),
];
